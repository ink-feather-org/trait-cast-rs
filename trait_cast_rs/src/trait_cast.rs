use core::{
  any::{type_name, Any, TypeId},
  fmt::{self, Debug, Formatter},
  mem,
};

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

/// A struct representing the transformation from `dyn Traitcastable` to another `dyn Trait`.
///
/// This should generally not be manually used, but generated by the `make_trait_castable` attribute macro.
pub struct TraitcastTarget {
  target_type_id: TypeId,
  target_type_name: &'static str,
  to_dyn_ref: fn(&dyn Traitcastable) -> *const (),
  to_dyn_mut: fn(&mut dyn Traitcastable) -> *mut (),
}
impl TraitcastTarget {
  /// Creates a `TraitcastTarget` from two function pointers.
  ///
  /// This is Safe since we know that the function Pointers have the correct Signature.
  ///
  /// As a side effect of this the user also doesn't have to use the `const_type_id` feature flag.
  pub const fn create<Target: 'static + ?Sized>(
    to_dyn_ref: fn(&dyn Traitcastable) -> Option<&Target>,
    to_dyn_mut: fn(&mut dyn Traitcastable) -> Option<&mut Target>,
  ) -> Self {
    Self {
      target_type_id: TypeId::of::<Target>(),
      target_type_name: type_name::<Target>(),
      // SAFETY:
      // We will transmute the return type back to the original type before we call the function.
      to_dyn_ref: unsafe { mem::transmute(to_dyn_ref) },
      // SAFETY:
      // We will transmute the return type back to the original type before we call the function.
      to_dyn_mut: unsafe { mem::transmute(to_dyn_mut) },
    }
  }
}

/// A Trait marking a Type as being able to traitcast to from `dyn Traitcastable` to another `dyn Trait`.
///
/// This should generally not be manually implemented, but generated by the `make_trait_castable` attribute macro.
pub trait Traitcastable: Any {
  /// This returns a list of all the `TraitcastTarget`'s to which a trait object can be cast, this is then used by the implementations of `TraitcastTo` to accomplish the Traitcast process.
  ///
  /// This should generally not be manually implemented, but generated by the `make_trait_castable` attribute macro.
  fn traitcast_targets(&self) -> &'static [TraitcastTarget];

  /// Returns the `TypeId` of the concrete type
  fn type_id(&self) -> TypeId {
    Any::type_id(self)
  }
}

/// Mimics the API of `Any` but additionally allows downcasts to select trait Objects.
pub trait TraitcastTo<Target: ?Sized> {
  /// Returns true if `Target` is the exact same type as Self
  fn is(&self) -> bool;

  /// Returns true if Self can be converted to a Target
  fn can_be(&self) -> bool;

  /// Returns some reference to the inner value if it is downcastable to type T, or None if it isn’t.
  ///
  /// If T is Sized this is forwarded to `Any::downcast_ref`,
  ///   otherwise `Traitcastable::traitcast_targets` is used to determine if a traitcast is possible.
  ///
  /// Return none if the concrete type of self is not Target and a traitcast is not possible.
  fn downcast_ref(&self) -> Option<&Target>;

  /// Unchecked variant of `downcast_ref`
  #[cfg(feature = "downcast_unchecked")]
  #[doc(cfg(feature = "downcast_unchecked"))]
  unsafe fn downcast_ref_unchecked(&self) -> &Target;

  /// Returns some mutable reference to the inner value if it is downcastable to type T, or None if it isn’t.
  ///
  /// If T is Sized this is forwarded to `Any::downcast_ref`,
  ///   otherwise `Traitcastable::traitcast_targets` is used to determine if a traitcast is possible.
  ///
  /// Return none if the concrete type of self is not Target and a traitcast is not possible.
  fn downcast_mut(&mut self) -> Option<&mut Target>;

  /// Unchecked variant of `downcast_ref`
  #[cfg(feature = "downcast_unchecked")]
  #[doc(cfg(feature = "downcast_unchecked"))]
  unsafe fn downcast_mut_unchecked(&mut self) -> &mut Target;

  /// Same as `downcast_ref` and `downcast_mut`, except that is downcasts a Box in place.
  ///
  /// Return none if the concrete type of self is not Target and a traitcast is not possible.
  /// # Errors
  /// In case of the cast being impossible the input is passed back.
  /// Otherwise the box would be dropped.
  #[cfg(feature = "alloc")]
  #[doc(cfg(feature = "alloc"))]
  fn downcast(self: Box<Self>) -> Result<Box<Target>, Box<Self>>;

  /// Unchecked variant of `downcast`
  #[cfg(all(feature = "alloc", feature = "downcast_unchecked"))]
  #[doc(cfg(all(feature = "alloc", feature = "downcast_unchecked")))]
  unsafe fn downcast_unchecked(self: Box<Self>) -> Box<Target>;
}

#[cfg(feature = "min_specialization")]
impl<T: 'static> Traitcastable for T {
  default fn traitcast_targets(&self) -> &'static [TraitcastTarget] {
    &[]
  }
}

macro_rules! implement_with_markers {
  ($($(+)? $traits:ident)*) => {
    impl Debug for dyn Traitcastable $(+ $traits)* {
      fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Traitcastable to {{")?;
        for (i, target) in self.traitcast_targets().iter().enumerate() {
          if i != 0 {
            write!(f, ", ")?;
          }
          write!(f, "{}", target.target_type_name)?;
        }
        write!(f, "}}")
      }
    }
    impl dyn Traitcastable $(+ $traits)* {
      fn get_trait_cast_target<Target: ?Sized + 'static + $($traits +)*>(&self) -> Option<&'static TraitcastTarget> {
        self
        .traitcast_targets()
        .iter()
        .find(|possible| possible.target_type_id == TypeId::of::<Target>())
      }
    }
    impl<Target: ?Sized + 'static + $($traits +)*> TraitcastTo<Target> for dyn Traitcastable $(+ $traits)* {
      default fn is(&self) -> bool {
        false
      }
      default fn can_be(&self) -> bool {
        self.get_trait_cast_target::<Target>().is_some()
      }
      default fn downcast_ref(&self) -> Option<&Target> {
        self.get_trait_cast_target::<Target>()
          .and_then(|target| {
            let fn_ptr: fn(&dyn Traitcastable) -> Option<&Target> =
            // SAFETY:
            // The actual type of the function was always:
            //  `fn(&dyn Traitcastable) -> Option<&Target>`
            // We had just previously thrown away the type of the return value.
              unsafe { mem::transmute(target.to_dyn_ref) };
            fn_ptr(self)
          })
      }
      #[cfg(feature = "downcast_unchecked")]
      default unsafe fn downcast_ref_unchecked(&self) -> &Target {
        self.downcast_ref().unwrap_unchecked()
      }

      default fn downcast_mut(&mut self) -> Option<&mut Target> {
        self.get_trait_cast_target::<Target>()
          .and_then(|target| {
            let fn_ptr: fn(&mut dyn Traitcastable) -> Option<&mut Target> =
            // SAFETY:
            // The actual type of the function was always:
            //  `fn(&mut dyn Traitcastable) -> Option<&mut Target>`
            // We had just previously thrown away the type of the return value.
              unsafe { mem::transmute(target.to_dyn_mut) };
            fn_ptr(self)
          })
      }
      #[cfg(feature = "downcast_unchecked")]
      default unsafe fn downcast_mut_unchecked(&mut self) -> &mut Target {
        self.downcast_mut().unwrap_unchecked()
      }

      #[cfg(feature = "alloc")]
      default fn downcast(self: Box<Self>) -> Result<Box<Target>, Box<Self>> {
        let raw : *mut Self = Box::into_raw(self);
          // SAFETY:
          // We can cast the *mut to a &mut since we never use the pointer directly in the success case
          //  and the reference isn't passed to the failure case.
        if let Some(to_ref) = unsafe {&mut *raw}.downcast_mut() {
          // SAFETY:
          // The pointer originates from a `Box` with the same dynamic type,
          //  since we only changed the pointer metadata.
          Ok(unsafe { Box::from_raw(to_ref) })
        } else {
          // SAFETY:
          // We reconstruct the previously destructed `Box`.
          Err( unsafe { Box::from_raw(raw)})
        }
      }

      #[cfg(all(feature = "alloc", feature = "downcast_unchecked"))]
      default unsafe fn downcast_unchecked(self: Box<Self>) -> Box<Target> {
        self.downcast().unwrap_unchecked()
      }
    }
    impl<Target: Sized + 'static + $($traits +)*> TraitcastTo<Target> for dyn Traitcastable $(+ $traits)* {
      fn is(&self) -> bool {
        <dyn Any>::is::<Target>(self)
      }
      fn can_be(&self) -> bool {
        <dyn Traitcastable as TraitcastTo<Target>>::is(self)
      }
      fn downcast_ref(&self) -> Option<&Target> {
        <dyn Any>::downcast_ref::<Target>(self)
      }
      #[cfg(feature = "downcast_unchecked")]
      unsafe fn downcast_ref_unchecked(&self) -> &Target {
        <dyn Any>::downcast_ref_unchecked::<Target>(self)
      }

      fn downcast_mut(&mut self) -> Option<&mut Target> {
        <dyn Any>::downcast_mut::<Target>(self)
      }
      #[cfg(feature = "downcast_unchecked")]
      unsafe fn downcast_mut_unchecked(&mut self) -> &mut Target {
        <dyn Any>::downcast_mut_unchecked::<Target>(self)
      }

      #[cfg(feature = "alloc")]
      fn downcast(self: Box<Self>) -> Result<Box<Target>, Box<Self>> { // TODO Extension Trait for Rc, Arc and dyn Error (probably also move Box)
        #[cfg(feature = "downcast_unchecked")]
        if TraitcastTo::<Target>::is(self.as_ref()) {
          // SAFETY:
          // We checked for dynamic type equality `is` in the previous if.
          unsafe { Ok(<Box<dyn Any>>::downcast_unchecked(self)) }
        } else { Err(self) }
        #[cfg(not(feature = "downcast_unchecked"))]
        if TraitcastTo::<Target>::is(self.as_ref()) { Ok(<Box<dyn Any>>::downcast(self).unwrap()) } else { Err(self) }
      }

      #[cfg(all(feature = "alloc", feature = "downcast_unchecked"))]
      unsafe fn downcast_unchecked(self: Box<Self>) -> Box<Target> {
        <Box<dyn Any>>::downcast_unchecked::<Target>(self)
      }
    }
  };
}

implement_with_markers!();
implement_with_markers!(Send);
implement_with_markers!(Send + Sync);
