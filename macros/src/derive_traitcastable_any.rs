use cargo_manifest_proc_macros::CargoManifest;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
  DeriveInput, Error, Token, TypePath,
  parse::{self, Parse, ParseStream},
  parse_macro_input,
  punctuated::Punctuated,
};

/// Helper struct to parse attribute arguments like `#[traitcast_targets(Ident, Ident, ...)]`.
struct TraitCastTargets {
  targets: Vec<TypePath>,
}

impl Parse for TraitCastTargets {
  fn parse(input: ParseStream<'_>) -> parse::Result<Self> {
    let targets: Vec<TypePath> = Punctuated::<TypePath, Token![,]>::parse_terminated(input)?
      .into_iter()
      .collect();
    Ok(Self { targets })
  }
}

impl quote::ToTokens for TraitCastTargets {
  fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
    let vars = &self.targets;
    tokens.extend(quote!(#(#vars),*));
  }
}

pub fn derive_traitcastable_any(input: TokenStream) -> TokenStream {
  let crate_path = CargoManifest::shared().resolve_crate_path("trait-cast", &[]);

  let derive_input = parse_macro_input!(input as DeriveInput);
  let source_ident = &derive_input.ident;

  let traitcast_targets_attr = derive_input
    .attrs
    .iter()
    .find(|attr| attr.path().is_ident("traitcast_targets"));

  let trait_cast_targets = if let Some(attr) = traitcast_targets_attr {
    match attr.parse_args::<TraitCastTargets>() {
      Ok(targets) => targets,
      Err(err) => {
        return Error::new_spanned(
          attr,
          format!("Failed to parse traitcast_targets attribute: {err}"),
        )
        .to_compile_error()
        .into();
      },
    }
  } else {
    return Error::new_spanned(
      derive_input.ident,
      "Missing required attribute 'traitcast_targets', e.g. #[traitcast_targets(TargetTrait1, TargetTrait2)]",
    )
    .to_compile_error()
    .into();
  };

  TokenStream::from(quote!(
    #crate_path::make_trait_castable_decl! {
    #source_ident => (#trait_cast_targets)
  }))
}
