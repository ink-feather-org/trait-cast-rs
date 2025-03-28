# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.7] - 2025-03-26

- Bump dependencies.

## [0.3.6] - 2025-01-22

- Bump dependencies.
- Minor cleanup and better CI.

## [0.3.5] - 2024-12-23

- Bump dependencies.
- Make the proc macro path resolver more robust.

## [0.3.4] - 2024-12-19

- Improve remapped dependency support in the proc macro.

## [0.3.3] - 2024-12-18

- Support remapping the `trait-cast` dependency in most cases when using the proc macro.

## [0.3.2] - 2024-12-16

- Support older nightly versions because the one that build the docs is currently frozen.

## [0.3.1] - 2024-12-14

- Restructure the project.
- Rebrand as `trait-cast`.

## [0.3.0] - 2024-12-12

- Move from `venial` to `syn` in `trait_cast_impl_rs`.
- Fix lots of clippy lints.
- Use [const Ordering for TypeId](https://github.com/rust-lang/rust/pull/101698) to make a lot of code cleanups.
- Removed the `const_sort` feature because the `const trait impls` in the standard library have been removed.
- Fix the doctest in the README when the `min_specialization` feature is enabled.

## [0.2.4] - 2022-11-10

### Fixes
- Updated dependencies.
- Fixed clippy lints

## [0.2.3] - 2022-09-25

Moved to [ink-feather-org](https://github.com/ink-feather-org/trait-cast-rs).
Updated dependencies.

### Fixes
- Fixed clippy lints

## [0.2.2] - 2022-09-22

### Fixes
- Fixed `const_trait_impl` breakage in dependency.

## [0.2.1] - 2022-09-21

### Added
  - Added `const_sort` feature.

## [0.2.0] - 2022-08-27

Initial release.

[Unreleased]: https://github.com/ink-feather-org/trait-cast-rs/compare/v0.3.7...HEAD
[0.3.7]: https://github.com/ink-feather-org/trait-cast-rs/compare/v0.3.6...v0.3.7
[0.3.6]: https://github.com/ink-feather-org/trait-cast-rs/compare/v0.3.5...v0.3.6
[0.3.5]: https://github.com/ink-feather-org/trait-cast-rs/compare/v0.3.4...v0.3.5
[0.3.4]: https://github.com/ink-feather-org/trait-cast-rs/compare/v0.3.3...v0.3.4
[0.3.3]: https://github.com/ink-feather-org/trait-cast-rs/compare/v0.3.2...v0.3.3
[0.3.2]: https://github.com/ink-feather-org/trait-cast-rs/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/ink-feather-org/trait-cast-rs/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/ink-feather-org/trait-cast-rs/compare/v0.2.4...v0.3.0
[0.2.4]: https://github.com/ink-feather-org/trait-cast-rs/compare/v0.2.3...v0.2.4
[0.2.3]: https://github.com/ink-feather-org/trait-cast-rs/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/ink-feather-org/trait-cast-rs/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/ink-feather-org/trait-cast-rs/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/ink-feather-org/trait-cast-rs/releases/tag/v0.2.0
