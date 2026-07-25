//! Trait definitions.

use crate::Array;
use core::{
    fmt::Debug,
};
use typenum::Unsigned;

/// Trait which associates a [`usize`] size and `ArrayType` with a
/// `typenum`-provided [`Unsigned`] integer.
///
/// # Safety
///
/// `ArrayType` MUST be an array with a number of elements exactly equal to
/// [`Unsigned::USIZE`]. Breaking this requirement will cause undefined behavior.
///
/// NOTE: This trait is effectively sealed and can not be implemented by third-party crates.
/// It is implemented only for a number of types defined in [`typenum::consts`].
#[diagnostic::on_unimplemented(note = "size may not be supported (see RustCrypto/hybrid-array#66)")]
pub unsafe trait ArraySize: Unsigned + Debug {
    /// Array type which corresponds to this size.
    ///
    /// This is always defined to be `[T; N]` where `N` is the same as
    /// [`ArraySize::USIZE`][`typenum::Unsigned::USIZE`].
    type ArrayType<T>: AssocArraySize<Size = Self>
        + AsRef<[T]>
        + AsMut<[T]>
        + IntoIterator<Item = T>
        // removing these is 0.2 seconds faster on my machine
        + core::borrow::Borrow<[T]>
        + core::borrow::BorrowMut<[T]>
        + From<Array<T, Self>>
        + core::ops::Index<usize>
        + core::ops::Index<core::ops::Range<usize>>
        + core::ops::IndexMut<usize>
        + core::ops::IndexMut<core::ops::Range<usize>>
        + Into<Array<T, Self>>
        ;
}

pub trait AssocArraySize: Sized {
    /// Size of an array type, expressed as a [`typenum`]-based [`ArraySize`].
    type Size: ArraySize;
}

pub trait AsArrayRef<T>: AssocArraySize {
    /// Converts this type into an immutable [`Array`] reference.
    fn as_array_ref(&self) -> &Array<T, Self::Size>;
}
