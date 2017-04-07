#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "nightly", feature(core_intrinsics, try_from))]

#[cfg(not(feature = "std"))]
pub mod std {
    pub use core::*;
}

use std::any::Any as StdAny;
use std::any::TypeId;
#[cfg(feature = "nightly")]
use std::convert::TryFrom;
#[cfg(feature = "std")]
use std::error::Error;
#[cfg(feature = "nightly")]
use std::intrinsics;
use std::fmt::{self, Debug, Display};
use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};

// ++++++++++++++++++++ Any ++++++++++++++++++++

#[cfg(feature = "nightly")]
fn type_name<T: StdAny + ?Sized>() -> &'static str { unsafe { intrinsics::type_name::<T>() } }
#[cfg(not(feature = "nightly"))]
fn type_name<T: StdAny + ?Sized>() -> &'static str { "[ONLY ON NIGHTLY]" }

/// FIXME(https://github.com/rust-lang/rust/issues/27745) remove this
pub trait Any: StdAny {
    fn type_id(&self) -> TypeId { TypeId::of::<Self>() }
    #[doc(hidden)]
    fn type_name(&self) -> &'static str { type_name::<Self>() }
}

impl<T> Any for T where T: StdAny + ?Sized {}

// ++++++++++++++++++++ TypeMismatch ++++++++++++++++++++

#[derive(Debug, Clone, Copy)]
pub struct TypeMismatch {
    expected: &'static str,
    found: &'static str,
}

impl TypeMismatch {
    pub fn new<T, O>(found_obj: &O) -> Self
        where T: Any + ?Sized, O: Any + ?Sized
    {
        TypeMismatch {
            expected: type_name::<T>(),
            found: found_obj.type_name(),
        }
    }
}

impl Display for TypeMismatch {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "Type mismatch: Expected '{}', found '{}'!", self.expected, self.found)
    }
}

#[cfg(feature = "std")]
impl Error for TypeMismatch {
    fn description(&self) -> &str { "Type mismatch" }
}

// ++++++++++++++++++++ DowncastError ++++++++++++++++++++

pub struct DowncastError<O> {
    mismatch: TypeMismatch,
    object: O,
}

impl<O> DowncastError<O> {
    pub fn new(mismatch: TypeMismatch, object: O) -> Self {
        Self {
            mismatch: mismatch,
            object: object,
        }
    }
    pub fn type_mismatch(&self) -> TypeMismatch { self.mismatch }
    pub fn into_object(self) -> O { self.object }
}

impl<O> Debug for DowncastError<O> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("DowncastError")
            .field("mismatch", &self.mismatch)
            .finish()
    }
}

impl<O> Display for DowncastError<O> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.mismatch, fmt)
    }
}

#[cfg(feature = "std")]
impl<O> Error for DowncastError<O> {
    fn description(&self) -> &str { self.mismatch.description() }
}

// ++++++++++++++++++++ Downcast ++++++++++++++++++++

pub trait Downcast<T>: Any
    where T: Any
{
    fn is_type(&self) -> bool;

    unsafe fn downcast_ref_unchecked(&self) -> &T;

    fn downcast_ref(&self) -> Result<&T, DowncastError<&Self>> {
        if self.is_type() {
            Ok(unsafe { self.downcast_ref_unchecked() })
        } else {
            let mismatch = TypeMismatch::new::<T, Self>(self);
            Err(DowncastError::new(mismatch, self))
        }
    }

    unsafe fn downcast_mut_unchecked(&mut self) -> &mut T;

    fn downcast_mut(&mut self) -> Result<&mut T, DowncastError<&mut Self>> {
        if self.is_type() {
            Ok(unsafe { self.downcast_mut_unchecked() })
        } else {
            let mismatch = TypeMismatch::new::<T, Self>(self);
            Err(DowncastError::new(mismatch, self))
        }
    }

    #[cfg(feature = "std")]
    unsafe fn downcast_unchecked(self: Box<Self>) -> Box<T>;

    #[cfg(feature = "std")]
    fn downcast(self: Box<Self>) -> Result<Box<T>, DowncastError<Box<Self>>> {
        if self.is_type() {
            Ok(unsafe { self.downcast_unchecked() })
        } else {
            let mismatch = TypeMismatch::new::<T, Self>(&*self);
            Err(DowncastError::new(mismatch, self))
        }
    }
}


// ++++++++++++++++++++ Downcasted ++++++++++++++++++++

pub struct Downcasted<T, O> {
    inner: O,
    _phantom: PhantomData<fn(T)>,
}

#[cfg(feature = "nightly")]
impl<T, O> TryFrom<O> for Downcasted<T, O>
    where T: Any, O: Deref, O::Target: Downcast<T>
{
    type Error = DowncastError<O>;
    fn try_from(inner: O) -> Result<Self, Self::Error> {
        if inner.is_type() {
            Ok(Self {
                   inner: inner,
                   _phantom: PhantomData,
               })
        } else {
            let mismatch = TypeMismatch::new::<T, O::Target>(&*inner);
            Err(DowncastError::new(mismatch, inner))
        }
    }
}

impl<T, O> From<O> for Downcasted<T, O>
    where T: Any, O: Deref, O::Target: Downcast<T>
{
    fn from(inner: O) -> Self {
        /* FIXME(try_from) use try_from().unwrap() */

        inner.downcast_ref().unwrap();
        Self {
            inner: inner,
            _phantom: PhantomData,
        }
    }
}

impl<T, O> Clone for Downcasted<T, O>
    where T: Any, O: Deref + Clone, O::Target: Downcast<T>
{
    fn clone(&self) -> Self { Self::from(self.inner.clone()) }
}

impl<T, O> Downcasted<T, O>
    where T: Any, O: Deref, O::Target: Downcast<T>
{
    pub fn into_inner(self) -> O { self.inner }
}

impl<T, O> Deref for Downcasted<T, O>
    where T: Any, O: Deref, O::Target: Downcast<T>
{
    type Target = T;
    fn deref(&self) -> &Self::Target { unsafe { self.inner.downcast_ref_unchecked() } }
}

impl<T, O> DerefMut for Downcasted<T, O>
    where T: Any, O: DerefMut, O::Target: Downcast<T>
{
    fn deref_mut(&mut self) -> &mut T { unsafe { self.inner.downcast_mut_unchecked() } }
}

// ++++++++++++++++++++ Downcasted2 ++++++++++++++++++++

pub struct Downcasted2<T, O> {
    inner: O,
    _phantom: PhantomData<fn(T)>,
}

#[cfg(feature = "nightly")]
impl<T, O> TryFrom<O> for Downcasted2<T, O>
    where T: Any, O: Deref, O::Target: Deref, <O::Target as Deref>::Target: Downcast<T>
{
    type Error = DowncastError<O>;
    fn try_from(inner: O) -> Result<Self, Self::Error> {
        if inner.is_type() {
            Ok(Self {
                   inner: inner,
                   _phantom: PhantomData,
               })
        } else {
            let mismatch = TypeMismatch::new::<T, <O::Target as Deref>::Target>(&**inner);
            Err(DowncastError::new(mismatch, inner))
        }
    }
}

impl<T, O> From<O> for Downcasted2<T, O>
    where T: Any, O: Deref, O::Target: Deref, <O::Target as Deref>::Target: Downcast<T>
{
    fn from(inner: O) -> Self {
        /* FIXME(try_from) use try_from().unwrap() */

        inner.downcast_ref().unwrap();
        Self {
            inner: inner,
            _phantom: PhantomData,
        }
    }
}

impl<T, O> Clone for Downcasted2<T, O>
    where T: Any, O: Deref + Clone, O::Target: Deref, <O::Target as Deref>::Target: Downcast<T>
{
    fn clone(&self) -> Self { Self::from(self.inner.clone()) }
}

impl<T, O> Downcasted2<T, O>
    where T: Any, O: Deref, O::Target: Deref, <O::Target as Deref>::Target: Downcast<T>
{
    pub fn into_inner(self) -> O { self.inner }
}

impl<T, O> Deref for Downcasted2<T, O>
    where T: Any, O: Deref, O::Target: Deref, <O::Target as Deref>::Target: Downcast<T>
{
    type Target = T;
    fn deref(&self) -> &Self::Target { unsafe { self.inner.downcast_ref_unchecked() } }
}

impl<T, O> DerefMut for Downcasted2<T, O>
    where T: Any, O: DerefMut, O::Target: DerefMut, <O::Target as Deref>::Target: Downcast<T>
{
    fn deref_mut(&mut self) -> &mut T { unsafe { self.inner.downcast_mut_unchecked() } }
}
// ++++++++++++++++++++ macros ++++++++++++++++++++

#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct TraitObject {
    pub data: *mut (),
    pub vtable: *mut (),
}

#[doc(hidden)]
#[inline]
pub fn to_trait_object<T: ?Sized>(obj: &T) -> TraitObject {
    assert_eq!(mem::size_of::<&T>(), mem::size_of::<TraitObject>());
    unsafe { *((&obj) as *const &T as *const TraitObject) }
}

#[doc(hidden)]
pub mod _std {
    pub use std::*;
}

/// Implements `Downcast<T: $base>` for your trait-object-type `$base`.
///
/// TODO Get rid of @core, automatically detect whether std is enabled
#[macro_export]
macro_rules! impl_downcast {
    (@core @items $t:ty) => {
        fn is_type(&self) -> bool {
            use $crate::_std::any::TypeId;

            $crate::Any::type_id(self) == TypeId::of::<$t>()
        }
        unsafe fn downcast_ref_unchecked(&self) -> &$t {
            &*($crate::to_trait_object(self).data as *mut $t)
        }
        unsafe fn downcast_mut_unchecked(&mut self) -> &mut $t {
            &mut*($crate::to_trait_object(self).data as *mut $t)
        }
    };
    (@items $t:ty) => {
        impl_downcast!(@core @items $t);

        unsafe fn downcast_unchecked(self: Box<Self>) -> Box<$t> {
            use $crate::_std::mem;

            let ret: Box<$t> = Box::from_raw($crate::to_trait_object(&*self).data as *mut $t);
            mem::forget(self);
            ret
        }
    };
    (@core <$($params:ident),+ $(,)*> $base:ty $(where $($bounds:tt)+)*) => {
        impl<_T: $crate::Any, $($params),+> $crate::Downcast<_T> for $base
            $(where $($bounds)+)*
        {
            impl_downcast!(@core @items _T);
        }
    };
    (<$($params:ident),+ $(,)*> $base:ty $(where $($bounds:tt)+)*) => {
        impl<_T: $crate::Any, $($params),+> $crate::Downcast<_T> for $base
            $(where $($bounds)+)*
        {
            impl_downcast!(@items _T);
        }
    };
    (@core $base:ty) => {
        impl<_T: $crate::Any> $crate::Downcast<_T> for $base {
            impl_downcast!(@core @items _T);
        }
    };
    ($base:ty) => {
        impl<_T: $crate::Any> $crate::Downcast<_T> for $base {
            impl_downcast!(@items _T);
        }
    };
}

/// Implement `downcast`-methods on your trait-object-type (these don't require
/// `Downcast` to
/// be imported to be used).
///
/// Generated methods:
///
/// ```
/// pub fn is<T>(&self) -> bool
///     where T: Any, Self: Downcast<T>;
///
/// pub unsafe fn downcast_ref_unchecked<T>(&self) -> &T
///     where T: Any, Self: Downcast<T>;
///
/// pub fn downcast_ref<T>(&self) -> Result<&T, DowncastError<&T>>
///     where T: Any, Self: Downcast<T>;
///
/// pub unsafe fn downcast_mut_unchecked<T>(&mut self) -> &mut T
///     where T: Any, Self: Downcast<T>;
///
/// pub fn downcast_mut<T>(&mut self) -> Result<&mut T, DowncastError<&mut T>>
///     where T: Any, Self: Downcast<T>;
///
/// pub unsafe fn downcast_unchecked<T>(self: Box<Self>) -> Box<T>
///     where T: Any, Self: Downcast<T>;
///
/// pub fn downcast<T>(self: Box<Self>) -> Result<Box<T>, DowncastError<Box<T>>>
///     where T: Any, Self: Downcast<T>;
/// ```
///
/// TODO Get rid of @core, automatically detect whether std is enabled
#[macro_export]
macro_rules! downcast_methods {
    (@core @items) => {
        #[allow(unused)]
        pub fn is<_T>(&self) -> bool
            where _T: $crate::Any, Self: $crate::Downcast<_T>
        {
            $crate::Downcast::<_T>::is_type(self)
        }

        #[allow(unused)]
        pub unsafe fn downcast_ref_unchecked<_T>(&self) -> &_T
            where _T: $crate::Any, Self: $crate::Downcast<_T>
        {
            $crate::Downcast::<_T>::downcast_ref_unchecked(self)
        }

        #[allow(unused)]
        pub fn downcast_ref<_T>(&self) -> Result<&_T, $crate::DowncastError<&Self>>
            where _T: $crate::Any, Self: $crate::Downcast<_T>
        {
            $crate::Downcast::<_T>::downcast_ref(self)
        }

        #[allow(unused)]
        pub unsafe fn downcast_mut_unchecked<_T>(&mut self) -> &mut _T
            where _T: $crate::Any, Self: $crate::Downcast<_T>
        {
            $crate::Downcast::<_T>::downcast_mut_unchecked(self)
        }

        #[allow(unused)]
        pub fn downcast_mut<_T>(&mut self) -> Result<&mut _T, $crate::DowncastError<&mut Self>>
            where _T: $crate::Any, Self: $crate::Downcast<_T>
        {
            $crate::Downcast::<_T>::downcast_mut(self)
        }
    };
    (@items) => {
        downcast_methods!(@core @items);

        #[allow(unused)]
        pub unsafe fn downcast_unchecked<_T>(self: Box<Self>) -> Box<_T>
            where _T: $crate::Any, Self: $crate::Downcast<_T>
        {
            $crate::Downcast::<_T>::downcast_unchecked(self)
        }

        #[allow(unused)]
        pub fn downcast<_T>(self: Box<Self>) ->  Result<Box<_T>, $crate::DowncastError<Box<Self>>>
            where _T: $crate::Any, Self: $crate::Downcast<_T>
        {
            $crate::Downcast::<_T>::downcast(self)
        }
    };
    (@core <$($params:ident),+ $(,)*> $base:ty $(where $($bounds:tt)+)*) => {
        impl<$($params),+> $base
            $(where $($bounds)+)*
        {
            downcast_methods!(@core @items);
        }
    };
    (<$($params:ident),+ $(,)*> $base:ty $(where $($bounds:tt)+)*) => {
        impl<$($params),+> $base
            $(where $($bounds)+)*
        {
            downcast_methods!(@items);
        }
    };
    (@core $base:ty) => {
        impl $base {
            downcast_methods!(@core @items);
        }
    };
    ($base:ty) => {
        impl $base {
            downcast_methods!(@items);
        }
    };
}

/// `impl_downcast!(...)` + `downcast_methods!(...)`
#[macro_export]
macro_rules! downcast {
    ($($tt:tt)+) => {
        impl_downcast!($($tt)+);
        downcast_methods!($($tt)+);
    }
}

// NOTE: We only implement the trait, because implementing the methods won't
// be possible when we replace downcast::Any by std::any::Any.
#[cfg(feature = "std")]
mod any_impls {
    use super::Any;

    impl_downcast!(Any);
    impl_downcast!((Any + Send));
    impl_downcast!((Any + Sync));
    impl_downcast!((Any + Send + Sync));
}
#[cfg(not(feature = "std"))]
mod any_impls {
    use super::Any;

    impl_downcast!(@core Any);
    impl_downcast!(@core (Any + Send));
    impl_downcast!(@core (Any + Sync));
    impl_downcast!(@core (Any + Send + Sync));
}
