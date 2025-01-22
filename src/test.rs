#![allow(dead_code)]
use crate::make_trait_castable_decl;

const fn _test_empty_trait_cast_targets() {
  struct Woof {}

  make_trait_castable_decl! {
    Woof => (),
  }
}

make_trait_castable_decl! {
    Source => (Print)
}

struct Source(i32);
trait Print {
  fn print(&self) -> i32;
}
impl Print for Source {
  fn print(&self) -> i32 {
    self.0
  }
}

#[cfg(feature = "alloc")]
#[test]
fn test_trait_castable() {
  use crate::{TraitcastableAny, TraitcastableAnyInfra};
  use alloc::boxed::Box;

  let source = Box::new(Source(5));
  let castable: Box<dyn TraitcastableAny> = source;
  let x: &dyn Print = castable.downcast_ref().unwrap();
  x.print();
}
