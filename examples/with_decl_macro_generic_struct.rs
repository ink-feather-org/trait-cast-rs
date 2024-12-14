//! This example demonstrates how to implement the `TraitcastableAny` trait for a generic struct `HybridPet` using the `make_trait_castable_decl` macro.
#![cfg_attr(feature = "min_specialization", feature(min_specialization))]
#![feature(ptr_metadata)]

use core::{any::type_name, fmt::Display};

use trait_cast::{make_trait_castable_decl, TraitcastableAny, TraitcastableAnyInfra};

struct HybridPet<T: Display> {
  name: T,
}
impl<T: Display> HybridPet<T> {
  fn greet(&self) {
    println!("{}: Hi {}", self.name, type_name::<T>());
  }
}

impl<T: Display> Dog for HybridPet<T> {
  fn bark(&self) {
    println!("{}: Woof!", self.name);
  }
}
impl<V: Display + ?Sized, T: Display> Cat<V> for HybridPet<T> {
  fn meow(&self, speak: &V) {
    println!("{}: Meow! {speak}", self.name);
  }
}

trait Dog {
  fn bark(&self);
}

/// Note: The `+ ?Sized` trait bound is not generally required but used to allow `str`.
trait Cat<T: Display + ?Sized> {
  fn meow(&self, speak: &T);
}
// With the decl_macro you can't (yet) be generic over a T, so we only make `HybridPet<String>` && `HybridPet<u8>` traitcastable.
make_trait_castable_decl! {
  HybridPet<String> => (Dog, Cat<str>),
  HybridPet<u8> => (Dog, Cat<u128>),
}
#[cfg_attr(test, test)]
fn main() {
  // The box is technically not needed but kept for added realism
  let pet = Box::new(HybridPet {
    name: "Kokusnuss".to_string(),
  });
  pet.greet();

  let castable_pet: Box<dyn TraitcastableAny> = pet;

  let as_dog: &dyn Dog = castable_pet.downcast_ref().unwrap();
  as_dog.bark();

  let as_cat: &dyn Cat<str> = castable_pet.downcast_ref().unwrap();
  as_cat.meow("Text");

  let cast_back: &HybridPet<String> = castable_pet.downcast_ref().unwrap();
  cast_back.greet();
}
