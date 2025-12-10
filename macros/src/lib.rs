//! Proc-macro automating the implementation of `trait_cast::TraitcastableAny`.
//!
//! See `derive_traitcastable_any` for more details.
//!

mod derive_traitcastable_any;

use proc_macro::TokenStream;

/// Derive macro implementing `TraitcastableAny` for a struct, enum or union.
///
/// Use the arguments to specify all possible target Traits for which trait objects are
/// supposed to be downcastable from a dyn `TraitcastableAny`.
///
/// Example:
/// ```ignore
/// extern crate trait_cast;
///
/// use trait_cast::TraitcastableAny;
///
///
/// #[derive(TraitcastableAny)]
/// #[traitcast_targets(Print)]
/// struct Source(i32);
///
/// trait Print {
///   fn print(&self);
/// }
/// impl Print for Source {
///   fn print(&self) {
///     println!("{}", self.0)
///   }
/// }
///
/// fn main() {
///   let source = Box::new(Source(5));
///   let castable: Box<dyn TraitcastableAny> = source;
///   let x: &dyn Print = castable.downcast_ref().unwrap();
///   x.print();
/// }
/// ```
#[proc_macro_derive(TraitcastableAny, attributes(traitcast_targets))]
pub fn derive_traitcastable_any(input: TokenStream) -> TokenStream {
  derive_traitcastable_any::derive_traitcastable_any(input)
}
