use std::{collections::HashMap, env, path::PathBuf, sync::LazyLock};
use thiserror::Error;
use toml_edit::{DocumentMut, Item};
use tracing::{info, trace};

pub fn pretty_format_syn_path(path: syn::Path) -> String {
  let mut path_str = String::new();
  let has_leading_colon = path.leading_colon.is_some();
  if has_leading_colon {
    path_str.push_str("::");
  }
  for segment in &path.segments {
    path_str.push_str(&segment.ident.to_string());
    path_str.push_str("::");
  }
  path_str.truncate(path_str.len() - 2);
  path_str
}

pub type PathPiece = syn::punctuated::Punctuated<syn::PathSegment, syn::Token![::]>;

pub trait CrateReExportingPolicy {
  fn get_re_exported_crate_path(&self, crate_name: &str) -> Option<PathPiece>;
}

pub struct KnownReExportingCrate<'a> {
  re_exporting_crate_name: &'a str,
  crate_re_exporting_policy: &'a dyn CrateReExportingPolicy,
}

#[derive(Error, Clone, Debug, PartialEq, Eq)]
pub enum TryResolveCratePathError {
  #[error("Ambiguous crate dependency {0}.")]
  AmbiguousDependency(String),
  #[error("Could not find crate path for {0}.")]
  CratePathNotFound(String),
}

/// The key is the crate name.
/// The value is the absolute module path of the crate.
#[derive(Default)]
struct ResolvedDependenciesMap(HashMap<String, Result<String, TryResolveCratePathError>>);

/// The cargo manifest (`Cargo.toml`) of the crate that is being built.
/// This can be the user's crate that either directly or indirectly depends on your crate.
/// If there are uses of the proc macro in your own crate, it may also point to the manifest of your own crate.
///
/// The [`CargoManifest`] is used to resolve a crate name to an absolute module path.
pub struct CargoManifest {
  user_crate_name: String,

  resolved_dependencies: ResolvedDependenciesMap,
  resolved_dependencies_dev: ResolvedDependenciesMap,
  resolved_dependencies_build: ResolvedDependenciesMap,
}

pub fn crate_name_to_path(crate_name: &str) -> syn::Path {
  let crate_name = crate_name.replace('-', "_");
  syn::parse_str::<syn::Path>(crate_name.as_str()).expect("Failed to parse crate name as path")
}

fn resolve_dependencies(
  dependencies_section: &Item,
  resolved_dependencies: &mut ResolvedDependenciesMap,
) {
  let dependencies_table = dependencies_section
    .as_table()
    .expect("The dependencies section in the Cargo.toml is not a table");

  for (dependency_key, dependency_item) in dependencies_table {
    // Get the actual dependency name whether it is remapped or not
    let dependency_crate_name = *dependency_item
      .get("package")
      .map(|package_field| {
        package_field
          .as_str()
          .expect("The package name in the Cargo.toml is not a string")
      })
      .get_or_insert(dependency_key);

    // Try to insert the crate into the [`ResolvedDependencyMap`].
    let insert_result = resolved_dependencies.0.try_insert(
      dependency_crate_name.to_string(),
      Ok(dependency_key.to_string()),
    );
    if let Err(_) = insert_result {
      // If the crate name is occupied.
      // We have an ambiguous dependency whose path can't be resolved.
      resolved_dependencies.0.insert(
        dependency_crate_name.to_string(),
        Err(TryResolveCratePathError::AmbiguousDependency(
          dependency_crate_name.to_string(),
        )),
      );
    }
  }
}

impl CargoManifest {
  /// Returns a global shared instance of the [`CargoManifest`] struct.
  pub fn shared() -> &'static LazyLock<Self> {
    static LAZY_MANIFEST: LazyLock<CargoManifest> = LazyLock::new(|| {
      let cargo_manifest_dir = env::var_os("CARGO_MANIFEST_DIR");
      info!("CARGO_MANIFEST_DIR: {:?}", cargo_manifest_dir);

      let cargo_manifest_path = cargo_manifest_dir
        .map(PathBuf::from)
        .map(|mut path| {
          path.push("Cargo.toml");
          if !path.exists() {
            panic!(
              "No Cargo manifest found for crate. Expected: {}",
              path.display()
            );
          }
          path
        })
        .expect("CARGO_MANIFEST_DIR is not defined.");

      let cargo_manifest_string = std::fs::read_to_string(cargo_manifest_path.clone())
        .unwrap_or_else(|err| {
          panic!(
            "Unable to read cargo manifest: {} - {err}",
            cargo_manifest_path.display()
          )
        });

      let cargo_manifest = cargo_manifest_string
        .parse::<DocumentMut>()
        .unwrap_or_else(|err| {
          panic!(
            "Failed to parse cargo manifest: {} - {err}",
            cargo_manifest_path.display()
          )
        });

      // parse user crate name
      let user_crate_name = cargo_manifest
        .get("package")
        .and_then(|package_section| package_section.get("name"))
        .and_then(|name_field| name_field.as_str())
        .expect("The package name in the Cargo.toml is not a string")
        .to_string();

      let resolve_dependencies_section = |dependencies_section| {
        let mut resolved_dependencies = ResolvedDependenciesMap::default();
        resolve_dependencies(dependencies_section, &mut resolved_dependencies);
        resolved_dependencies
      };

      let resolved_dependencies = cargo_manifest
        .get("dependencies")
        .map(resolve_dependencies_section)
        .unwrap_or_default();
      let resolved_dependencies_dev = cargo_manifest
        .get("dev-dependencies")
        .map(resolve_dependencies_section)
        .unwrap_or_default();
      let resolved_dependencies_build = cargo_manifest
        .get("build-dependencies")
        .map(resolve_dependencies_section)
        .unwrap_or_default();

      CargoManifest {
        user_crate_name,
        resolved_dependencies,
        resolved_dependencies_dev,
        resolved_dependencies_build,
      }
    });
    &LAZY_MANIFEST
  }

  /// Gets the absolute module path for a crate from a supplied dependencies section.
  ///
  /// Crates that had their module path remapped are also supported.
  ///
  /// For the normal crate case:
  ///
  /// ```toml
  /// [dependencies]
  /// original-crate-name = "0.1"
  /// ```
  ///
  /// The function would return `Some("original-crate-name")` for the `Item` above.
  ///
  /// For the remapped crate case:
  ///
  /// ```toml
  /// [dependencies]
  /// renamed-crate-name = { version = "0.1", package = "original-crate-name" }
  /// ```
  ///
  /// The function would return `Some("renamed-crate-name")` for the `Item` above.
  fn try_resolve_crate_path_for_dependency_map(
    &self,
    crate_name: &str,
    known_re_exporting_crates: &[&KnownReExportingCrate<'_>],
    resolved_dependencies: &ResolvedDependenciesMap,
  ) -> Result<syn::Path, TryResolveCratePathError> {
    // Check if the user crate is our own crate.
    if crate_name == self.user_crate_name {
      return Ok(crate_name_to_path(crate_name));
    }

    // Check if we have a direct dependency.
    let directly_mapped_crate_name = resolved_dependencies.0.get(crate_name);
    if let Some(directly_mapped_crate_name) = directly_mapped_crate_name {
      let directly_mapped_crate_name = match directly_mapped_crate_name {
        Ok(crate_name) => crate_name,
        Err(err) => {
          return Err(err.clone());
        },
      };

      // We have a direct dependency.
      trace!("Found direct dependency: {}", directly_mapped_crate_name);
      return Ok(crate_name_to_path(directly_mapped_crate_name));
    }

    for known_re_exporting_crate in known_re_exporting_crates {
      // Check if we have a known re-exporting crate.
      let indirect_mapped_exporting_crate_name = resolved_dependencies
        .0
        .get(known_re_exporting_crate.re_exporting_crate_name);
      if let Some(indirect_mapped_exporting_crate_name) = indirect_mapped_exporting_crate_name {
        let indirect_mapped_exporting_crate_name = match indirect_mapped_exporting_crate_name {
          Ok(crate_name) => crate_name,
          Err(err) => {
            return Err(err.clone()); // TODO: collect all errors
          },
        };

        // We have a known re-exporting crate.
        let re_exported_crate_path = known_re_exporting_crate
          .crate_re_exporting_policy
          .get_re_exported_crate_path(crate_name);
        if let Some(re_exported_crate_path) = re_exported_crate_path {
          trace!(
            "Found re-exporting crate: {} -> {}",
            known_re_exporting_crate.re_exporting_crate_name, crate_name
          );
          let mut path = crate_name_to_path(indirect_mapped_exporting_crate_name);
          path.segments.extend(re_exported_crate_path);
          return Ok(path);
        }
      }
    }

    Err(TryResolveCratePathError::CratePathNotFound(
      crate_name.to_string(),
    ))
  }

  /// Attempt to retrieve the absolute module path of a crate named [possible_crate_names](str) as an absolute [`syn::Path`].
  /// Remapped crate names are also supported.
  ///
  ///  # Arguments
  ///
  /// * `crate_name` - The name of the crate to get the path for.
  ///
  /// * `known_re_exporting_crates` - A list of known crates that re-export the proc macro.
  /// This is useful for monorepos like bevy where the user typically only depends on the main bevy crate but uses
  /// proc macros from subcrates like `bevy_ecs`.
  /// If a direct dependency exists, it is preferred over a re-exporting crate.
  pub fn try_resolve_crate_path(
    &self,
    crate_name: &str,
    known_re_exporting_crates: &[&KnownReExportingCrate<'_>],
  ) -> Result<syn::Path, TryResolveCratePathError> {
    info!("Trying to get the path for: {}", crate_name);

    let ret = self
      .try_resolve_crate_path_for_dependency_map(
        crate_name,
        known_re_exporting_crates,
        &self.resolved_dependencies,
      )
      .or_else(|_| {
        self.try_resolve_crate_path_for_dependency_map(
          crate_name,
          known_re_exporting_crates,
          &self.resolved_dependencies_dev,
        )
      })
      .or_else(|_| {
        self.try_resolve_crate_path_for_dependency_map(
          crate_name,
          known_re_exporting_crates,
          &self.resolved_dependencies_build,
        )
      });

    info!(
      "Computed path: {:?} for {}",
      ret.clone().map(pretty_format_syn_path),
      crate_name
    );
    ret
  }

  /// Returns the path for the crate with the given name.
  pub fn resolve_crate_path(
    &self,
    crate_name: &str,
    known_re_exporting_crates: &[&KnownReExportingCrate<'_>],
  ) -> syn::Path {
    self
      .try_resolve_crate_path(crate_name, &known_re_exporting_crates)
      .unwrap_or_else(|_| crate_name_to_path(crate_name))
  }
}
