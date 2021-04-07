//! Provides a type which allows for temporarily deinitialising references.
//! 
//! ```
//! use reinit::*;
//! 
//! let mut n = 42;
//! let init = Initialised::new(&mut n);
//! let (v, uninit,) = init.take();
//! assert_eq!(v, 42);
//! let init = uninit.init(24);
//! assert_eq!(n, 24);
//! ```
//! 
//! Author --- DMorgan  
//! Last Moddified --- 2021-04-07

#![no_std]
#![deny(missing_docs,)]
#![feature(
  const_ptr_read, const_maybe_uninit_as_ptr, const_refs_to_cell, const_mut_refs,
  const_ptr_write, const_raw_ptr_deref, const_panic, const_fn_transmute,
)]

use core::{
  ops::{Deref, DerefMut,},
  marker::PhantomData,
};

/// A reference to initialised memory.
#[repr(transparent,)]
pub struct Initialised<'a, T: 'a,> {
  /// The reference.
  slot: &'a mut T,
}

impl<'a, T,> Initialised<'a, T,> {
  /// Constructs a new `Initialised` from `slot`.
  #[inline]
  pub const fn new(slot: &'a mut T,) -> Self { Self { slot, } }
  /// Returns the inner value.
  #[inline]
  pub const fn into_inner(self,) -> &'a mut T { self.slot }
  /// Moves the value behind the reference and leaves the reference uninitialised.
  #[inline]
  pub const fn take(self,) -> (T, Uninitialised<'a, T,>,) {
    use core::ptr;

    unsafe {
      (
        ptr::read(self.slot,),
        Uninitialised { slot: self.slot, _phantom: PhantomData, },
      )
    }
  }
}

impl<T,> Deref for Initialised<'_, T,>
  where T: Deref, {
  type Target = T::Target;

  #[inline]
  fn deref(&self,) -> &Self::Target { self.slot.deref() }
}

impl<T,> DerefMut for Initialised<'_, T,>
  where T: DerefMut, {
  #[inline]
  fn deref_mut(&mut self,) -> &mut Self::Target { self.slot.deref_mut() }
}

impl<'a, T,> From<&'a mut T> for Initialised<'a, T,> {
  #[inline]
  fn from(from: &'a mut T,) -> Self { Initialised::new(from,) }
}

/// A reference to uninitialised memory.
/// 
/// Dropping this value will panic as the referenced memory is left uninitialised.
/// 
/// ```no_run
/// use reinit::*;
/// 
/// let mut num = 42;
/// let init = Initialised::new(&mut num);
/// let (v, uninit) = init.take();
/// // This will panic when `uninit` is dropped because the value in `num` was moved.
/// ```
#[repr(transparent,)]
#[must_use]
pub struct Uninitialised<'a, T: 'a,> {
  /// The reference.
  #[allow(dead_code,)]
  slot: *mut T,
  _phantom: PhantomData<&'a ()>,
}

impl<'a, T,> Uninitialised<'a, T,> {
  /// Reinitialises the reference.
  #[inline]
  pub const fn init(self, value: T,) -> Initialised<'a, T,> {
    use core::{ptr, mem::{transmute, MaybeUninit,},};

    unsafe {
      let slot = transmute::<_, *mut T,>(MaybeUninit::new(self,),);
      ptr::write(slot, value,);
      Initialised::new(&mut *slot,)
    }
  }
}

impl<'a, T,> Drop for Uninitialised<'a, T,> {
  #[track_caller]
  #[inline]
  fn drop(&mut self,) { panic!(concat!("Dropped an `", stringify!(Uninitialised),"` value",),) }
}

#[cfg(test,)]
mod tests {
  use super::*;

  #[test]
  fn test_sizes() {
    assert_eq!(core::mem::size_of::<Initialised<bool,>>(), core::mem::size_of::<*mut bool>());
    assert_eq!(core::mem::size_of::<Initialised<i32,>>(), core::mem::size_of::<*mut i32>());
    assert_eq!(core::mem::size_of::<Uninitialised<bool,>>(), core::mem::size_of::<*mut bool>());
    assert_eq!(core::mem::size_of::<Uninitialised<i32,>>(), core::mem::size_of::<*mut i32>());
  }
  #[test]
  #[should_panic]
  #[allow(unused_must_use,)]
  fn test_dropping_uninit() {
    let mut b = true;
    let init = Initialised::new(&mut b,);
    init.take();
  }
  #[test]
  fn test_reinit() {
    let mut b = 42;
    let init = Initialised::new(&mut b,);
    let (v, uninit,) = init.take();
    assert_eq!(v, 42, "Got incorrect value",);
    let v = *uninit.init(10,).into_inner();
    assert_eq!(v, b, "Ptr does not line up",);
    assert_eq!(b, 10, "Set incorrect value",)
  }
}
