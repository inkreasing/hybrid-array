#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg"
)]

pub mod sizes;

mod from_fn;
mod traits;

pub use crate::traits::*;

use core::{
    array::TryFromSliceError,
    borrow::{Borrow, BorrowMut},
    cmp::Ordering,
    fmt::{self, Debug},
    hash::{Hash, Hasher},
    mem::{self, MaybeUninit},
    ops::{Deref, DerefMut, Index, IndexMut},
    ptr,
    slice::{self},
};

#[repr(transparent)]
pub struct Array<T, U: ArraySize>(pub U::ArrayType<T>);

impl<T, U> Array<T, U>
where
    U: ArraySize,
{
    /// Returns a slice containing the entire array. Equivalent to `&s[..]`.
    #[inline]
    pub const fn as_slice(&self) -> &[T] {
        // SAFETY: `[T]` is layout-identical to `Array<T, U>`, which is a `repr(transparent)`
        // newtype for `[T; N]`.
        unsafe { slice::from_raw_parts(self.as_ptr(), U::USIZE) }
    }

    /// Returns a mutable slice containing the entire array. Equivalent to `&mut s[..]`.
    #[inline]
    pub const fn as_mut_slice(&mut self) -> &mut [T] {
        // SAFETY: `[T]` is layout-identical to `Array<T, U>`, which is a `repr(transparent)`
        // newtype for `[T; N]`.
        unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), U::USIZE) }
    }

    /// Returns a pointer to the start of the array.
    pub const fn as_ptr(&self) -> *const T {
        ptr::from_ref::<Self>(self).cast::<T>()
    }

    pub const fn as_mut_ptr(&mut self) -> *mut T {
        ptr::from_mut::<Self>(self).cast::<T>()
    }

    pub const fn slice_as_array(slice: &[T]) -> Option<&Self> {
        if slice.len() == U::USIZE {
            // SAFETY: `Self` is ensured to be layout-identical to `[T; U::USIZE]`, and immediately
            // above we validated that `slice` is also layout-identical to `[T; U::USIZE]`,
            // therefore the cast is valid.
            unsafe { Some(&*slice.as_ptr().cast()) }
        } else {
            None
        }
    }

    pub const fn slice_as_mut_array(slice: &mut [T]) -> Option<&mut Self> {
        if slice.len() == U::USIZE {
            // SAFETY: `Self` is ensured to be layout-identical to `[T; U::USIZE]`, and immediately
            // above we validated that `slice` is also layout-identical to `[T; U::USIZE]`,
            // therefore the cast is valid.
            unsafe { Some(&mut *slice.as_mut_ptr().cast()) }
        } else {
            None
        }
    }

    pub const fn slice_as_chunks(buf: &[T]) -> (&[Self], &[T]) {
        assert!(U::USIZE != 0, "chunk size must be non-zero");
        // Arithmetic safety: we have checked that `N::USIZE` is not zero, thus
        // division always returns correct result. `tail_pos` can not be bigger than `buf.len()`,
        // thus overflow on multiplication and underflow on substraction are impossible.
        let chunks_len = buf.len() / U::USIZE;
        let tail_pos = U::USIZE * chunks_len;
        let tail_len = buf.len() - tail_pos;
        unsafe {
            let ptr = buf.as_ptr();
            let chunks = slice::from_raw_parts(ptr.cast(), chunks_len);
            let tail = slice::from_raw_parts(ptr.add(tail_pos), tail_len);
            (chunks, tail)
        }
    }

    /// Splits the exclusive slice into a slice of `U`-element arrays, starting at the beginning
    /// of the slice, and a remainder slice with length strictly less than `U`.
    ///
    /// # Panics
    /// Panics if `U` is 0.
    #[allow(clippy::arithmetic_side_effects)]
    #[inline]
    pub const fn slice_as_chunks_mut(buf: &mut [T]) -> (&mut [Self], &mut [T]) {
        assert!(U::USIZE != 0, "chunk size must be non-zero");
        // Arithmetic safety: we have checked that `N::USIZE` is not zero, thus
        // division always returns correct result. `tail_pos` can not be bigger than `buf.len()`,
        // thus overflow on multiplication and underflow on substraction are impossible.
        let chunks_len = buf.len() / U::USIZE;
        let tail_pos = U::USIZE * chunks_len;
        let tail_len = buf.len() - tail_pos;
        unsafe {
            let ptr = buf.as_mut_ptr();
            let chunks = slice::from_raw_parts_mut(ptr.cast(), chunks_len);
            let tail = slice::from_raw_parts_mut(ptr.add(tail_pos), tail_len);
            (chunks, tail)
        }
    }
}
// Impls which depend on the inner array type being `[T; N]`.
impl<T, U, const N: usize> Array<T, U>
where
    U: ArraySize<ArrayType<T> = [T; N]>,
{
    /// Cast a reference to a core array to an [`Array`] reference.
    #[inline]
    pub const fn cast_from_core(array_ref: &[T; N]) -> &Self {
        // SAFETY: `Self` is a `repr(transparent)` newtype for `[T; N]`
        unsafe { &*array_ref.as_ptr().cast() }
    }

    /// Cast a mutable reference to a core array to an [`Array`] reference.
    #[inline]
    pub const fn cast_from_core_mut(array_ref: &mut [T; N]) -> &mut Self {
        // SAFETY: `Self` is a `repr(transparent)` newtype for `[T; 1]`
        unsafe { &mut *array_ref.as_mut_ptr().cast() }
    }

    /// Transform slice to slice of core array type.
    #[inline]
    pub const fn cast_slice_from_core(slice: &[[T; N]]) -> &[Self] {
        // SAFETY: `Self` is a `repr(transparent)` newtype for `[T; N]`
        unsafe { slice::from_raw_parts(slice.as_ptr().cast(), slice.len()) }
    }

    /// Transform mutable slice to mutable slice of core array type.
    #[inline]
    pub const fn cast_slice_from_core_mut(slice: &mut [[T; N]]) -> &mut [Self] {
        // SAFETY: `Self` is a `repr(transparent)` newtype for `[T; N]`
        unsafe { slice::from_raw_parts_mut(slice.as_mut_ptr().cast(), slice.len()) }
    }

    /// Transform slice to slice of core array type.
    #[inline]
    pub const fn cast_slice_to_core(slice: &[Self]) -> &[[T; N]] {
        // SAFETY: `Self` is a `repr(transparent)` newtype for `[T; N]`
        unsafe { slice::from_raw_parts(slice.as_ptr().cast(), slice.len()) }
    }

    /// Transform mutable slice to mutable slice of core array type.
    #[inline]
    pub const fn cast_slice_to_core_mut(slice: &mut [Self]) -> &mut [[T; N]] {
        // SAFETY: `Self` is a `repr(transparent)` newtype for `[T; N]`
        unsafe { slice::from_raw_parts_mut(slice.as_mut_ptr().cast(), slice.len()) }
    }
}

impl<T, U> Array<MaybeUninit<T>, U>
where
    U: ArraySize,
{
    /// Create an uninitialized array of [`MaybeUninit`]s for the given type.
    #[must_use]
    pub const fn uninit() -> Array<MaybeUninit<T>, U> {
        // SAFETY: `Array` is a `repr(transparent)` newtype for `[MaybeUninit<T>, N]`, i.e. an
        // array of uninitialized memory mediated via the `MaybeUninit` interface, where the inner
        // type is constrained by `ArraySize` impls which can only be added by this crate.
        //
        // Calling `uninit().assume_init()` triggers the `clippy::uninit_assumed_init` lint, but
        // as just mentioned the inner type we're "assuming init" for is `[MaybeUninit<T>, N]`,
        // i.e. an array of uninitialized memory, which is always valid because definitionally no
        // initialization is required of uninitialized memory.
        #[allow(clippy::uninit_assumed_init)]
        Self(unsafe { MaybeUninit::uninit().assume_init() })
    }

    /// Extract the values from an array of `MaybeUninit` containers.
    ///
    /// # Safety
    ///
    /// It is up to the caller to guarantee that all elements of the array are in an initialized
    /// state.
    #[inline]
    pub unsafe fn assume_init(self) -> Array<T, U> {
        unsafe {
            // `Array` is a `repr(transparent)` newtype for a generic inner type which is constrained to
            // be `[T; N]` by the `ArraySize` impls in this crate.
            //
            // Since we're working with a type-erased inner type and ultimately trying to convert
            // `[MaybeUninit<T>; N]` to `[T; N]`, we can't use simpler approaches like a pointer cast
            // or `transmute`, since the compiler can't prove to itself that the size will be the same.
            //
            // We've taken unique ownership of `self`, which is a `MaybeUninit` array, and as such we
            // don't need to worry about `Drop` impls because `MaybeUninit` does not impl `Drop`.
            // Since we have unique ownership of `self`, it's okay to make a copy because we're throwing
            // the original away (and this should all get optimized to a noop by the compiler, anyway).
            mem::transmute_copy(&self)
        }
    }
}
impl<T, U> AsRef<Array<T, U>> for Array<T, U>
where
    U: ArraySize,
{
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<T, U> AsRef<[T]> for Array<T, U>
where
    U: ArraySize,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.0.as_ref()
    }
}

impl<T, U, const N: usize> AsRef<[T; N]> for Array<T, U>
where
    U: ArraySize<ArrayType<T> = [T; N]>,
{
    #[inline]
    fn as_ref(&self) -> &[T; N] {
        &self.0
    }
}

impl<T, U> AsMut<Array<T, U>> for Array<T, U>
where
    U: ArraySize,
{
    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<T, U> AsMut<[T]> for Array<T, U>
where
    U: ArraySize,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self.0.as_mut()
    }
}

impl<T, U, const N: usize> AsMut<[T; N]> for Array<T, U>
where
    U: ArraySize<ArrayType<T> = [T; N]>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [T; N] {
        &mut self.0
    }
}

impl<T, U> Borrow<[T]> for Array<T, U>
where
    U: ArraySize,
{
    #[inline]
    fn borrow(&self) -> &[T] {
        self.0.as_ref()
    }
}

impl<T, U, const N: usize> Borrow<[T; N]> for Array<T, U>
where
    U: ArraySize<ArrayType<T> = [T; N]>,
{
    #[inline]
    fn borrow(&self) -> &[T; N] {
        &self.0
    }
}

impl<T, U> BorrowMut<[T]> for Array<T, U>
where
    U: ArraySize,
{
    #[inline]
    fn borrow_mut(&mut self) -> &mut [T] {
        self.0.as_mut()
    }
}

impl<T, U, const N: usize> BorrowMut<[T; N]> for Array<T, U>
where
    U: ArraySize<ArrayType<T> = [T; N]>,
{
    #[inline]
    fn borrow_mut(&mut self) -> &mut [T; N] {
        &mut self.0
    }
}

impl<T, U> Clone for Array<T, U>
where
    T: Clone,
    U: ArraySize,
{
    #[inline]
    fn clone(&self) -> Self {
        Self::from_fn(|n| self.0.as_ref()[n].clone())
    }
}

impl<T, U> Copy for Array<T, U>
where
    T: Copy,
    U: ArraySize,
    U::ArrayType<T>: Copy,
{
}

impl<T, U> Debug for Array<T, U>
where
    T: Debug,
    U: ArraySize,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Array").field(&self.0.as_ref()).finish()
    }
}

impl<T, U> Default for Array<T, U>
where
    T: Default,
    U: ArraySize,
{
    #[inline]
    fn default() -> Self {
        Self::from_fn(|_| Default::default())
    }
}

impl<T, U> Deref for Array<T, U>
where
    U: ArraySize,
{
    type Target = [T];

    #[inline]
    fn deref(&self) -> &[T] {
        self.0.as_ref()
    }
}

impl<T, U> DerefMut for Array<T, U>
where
    U: ArraySize,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut [T] {
        self.0.as_mut()
    }
}

impl<T, U> Eq for Array<T, U>
where
    T: Eq,
    U: ArraySize,
{
}

impl<T, U, const N: usize> From<[T; N]> for Array<T, U>
where
    U: ArraySize<ArrayType<T> = [T; N]>,
{
    #[inline]
    fn from(arr: [T; N]) -> Array<T, U> {
        Array(arr)
    }
}

impl<T, U, const N: usize> From<Array<T, U>> for [T; N]
where
    U: ArraySize<ArrayType<T> = [T; N]>,
{
    #[inline]
    fn from(arr: Array<T, U>) -> [T; N] {
        arr.0
    }
}

impl<'a, T, U, const N: usize> From<&'a [T; N]> for &'a Array<T, U>
where
    U: ArraySize<ArrayType<T> = [T; N]>,
{
    #[inline]
    fn from(array_ref: &'a [T; N]) -> &'a Array<T, U> {
        Array::cast_from_core(array_ref)
    }
}

impl<'a, T, U, const N: usize> From<&'a Array<T, U>> for &'a [T; N]
where
    U: ArraySize<ArrayType<T> = [T; N]>,
{
    #[inline]
    fn from(array_ref: &'a Array<T, U>) -> &'a [T; N] {
        array_ref.as_ref()
    }
}

impl<'a, T, U, const N: usize> From<&'a mut [T; N]> for &'a mut Array<T, U>
where
    U: ArraySize<ArrayType<T> = [T; N]>,
{
    #[inline]
    fn from(array_ref: &'a mut [T; N]) -> &'a mut Array<T, U> {
        Array::cast_from_core_mut(array_ref)
    }
}

impl<'a, T, U, const N: usize> From<&'a mut Array<T, U>> for &'a mut [T; N]
where
    U: ArraySize<ArrayType<T> = [T; N]>,
{
    #[inline]
    fn from(array_ref: &'a mut Array<T, U>) -> &'a mut [T; N] {
        array_ref.as_mut()
    }
}
impl<T, U> Hash for Array<T, U>
where
    T: Hash,
    U: ArraySize,
{
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.as_ref().hash(state);
    }
}

impl<T, I, U> Index<I> for Array<T, U>
where
    [T]: Index<I>,
    U: ArraySize,
{
    type Output = <[T] as Index<I>>::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        Index::index(self.as_slice(), index)
    }
}

impl<T, I, U> IndexMut<I> for Array<T, U>
where
    [T]: IndexMut<I>,
    U: ArraySize,
{
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(self.as_mut_slice(), index)
    }
}

impl<T, U> PartialEq for Array<T, U>
where
    T: PartialEq,
    U: ArraySize,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ref().eq(other.0.as_ref())
    }
}

impl<T, U, const N: usize> PartialEq<[T; N]> for Array<T, U>
where
    T: PartialEq,
    U: ArraySize<ArrayType<T> = [T; N]>,
{
    #[inline]
    fn eq(&self, other: &[T; N]) -> bool {
        self.0.eq(other)
    }
}

impl<T, U, const N: usize> PartialEq<Array<T, U>> for [T; N]
where
    T: PartialEq,
    U: ArraySize<ArrayType<T> = [T; N]>,
{
    #[inline]
    fn eq(&self, other: &Array<T, U>) -> bool {
        self.eq(&other.0)
    }
}

impl<T, U> PartialOrd for Array<T, U>
where
    T: PartialOrd,
    U: ArraySize,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.as_ref().partial_cmp(other.0.as_ref())
    }
}

impl<T, U> Ord for Array<T, U>
where
    T: Ord,
    U: ArraySize,
{
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.as_ref().cmp(other.0.as_ref())
    }
}

/// SAFETY: `Array` is a `repr(transparent)` newtype for `[T; N]`, so as long as `T: Send` it should
/// also be `Send`.
unsafe impl<T, U: ArraySize> Send for Array<T, U> where T: Send {}

/// SAFETY: `Array` is a `repr(transparent)` newtype for `[T; N]`, so as long as `T: Sync` it should
/// also be `Sync`.
unsafe impl<T, U: ArraySize> Sync for Array<T, U> where T: Sync {}

impl<'a, T, U> TryFrom<&'a [T]> for &'a Array<T, U>
where
    U: ArraySize,
{
    type Error = TryFromSliceError;

    #[inline]
    fn try_from(slice: &'a [T]) -> Result<Self, TryFromSliceError> {
        check_slice_length::<T, U>(slice)?;

        // SAFETY: `Array<T, U>` is a `repr(transparent)` newtype for a core
        // array with length checked above.
        Ok(unsafe { &*slice.as_ptr().cast() })
    }
}

impl<'a, T, U> TryFrom<&'a mut [T]> for &'a mut Array<T, U>
where
    U: ArraySize,
{
    type Error = TryFromSliceError;

    #[inline]
    fn try_from(slice: &'a mut [T]) -> Result<Self, TryFromSliceError> {
        check_slice_length::<T, U>(slice)?;

        // SAFETY: `Array<T, U>` is a `repr(transparent)` newtype for a core
        // array with length checked above.
        Ok(unsafe { &mut *slice.as_mut_ptr().cast() })
    }
}

impl<'a, T, U> TryFrom<&'a [T]> for Array<T, U>
where
    Self: Clone,
    U: ArraySize,
{
    type Error = TryFromSliceError;

    #[inline]
    fn try_from(slice: &'a [T]) -> Result<Array<T, U>, TryFromSliceError> {
        <&'a Self>::try_from(slice).cloned()
    }
}

// Deprecated legacy methods to ease migrations from `generic-array`
impl<T, U> Array<T, U>
where
    U: ArraySize,
{
    /// Convert the given slice into a reference to a hybrid array.
    ///
    /// # Panics
    ///
    /// Panics if the slice's length doesn't match the array type.
    #[deprecated(since = "0.2.0", note = "use `TryFrom` instead")]
    #[inline]
    pub fn from_slice(slice: &[T]) -> &Self {
        slice.try_into().expect("slice length mismatch")
    }

    /// Convert the given mutable slice to a mutable reference to a hybrid array.
    ///
    /// # Panics
    ///
    /// Panics if the slice's length doesn't match the array type.
    #[deprecated(since = "0.2.0", note = "use `TryFrom` instead")]
    #[inline]
    pub fn from_mut_slice(slice: &mut [T]) -> &mut Self {
        slice.try_into().expect("slice length mismatch")
    }

    /// Clone the contents of the slice as a new hybrid array.
    ///
    /// # Panics
    ///
    /// Panics if the slice's length doesn't match the array type.
    #[deprecated(since = "0.2.0", note = "use `TryFrom` instead")]
    #[inline]
    pub fn clone_from_slice(slice: &[T]) -> Self
    where
        Self: Clone,
    {
        slice.try_into().expect("slice length mismatch")
    }
}

/// Generate a [`TryFromSliceError`] if the slice doesn't match the given length.
fn check_slice_length<T, U: ArraySize>(slice: &[T]) -> Result<(), TryFromSliceError> {
    debug_assert_eq!(Array::<(), U>::default().len(), U::USIZE);

    if slice.len() == U::USIZE {
        Ok(())
    } else {
        // Hack: `TryFromSliceError` lacks a public constructor, so this fakes one
        <&[T; 1]>::try_from([].as_slice()).map(|_| ())
    }
}
