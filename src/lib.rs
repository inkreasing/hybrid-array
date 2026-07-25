#![no_std]

pub mod sizes;

#[repr(transparent)]
pub struct Array<T, U: ArraySize>(pub U::ArrayType<T>);

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

pub unsafe trait ArraySize: typenum::Unsigned {
    type ArrayType<T>: AssocArraySize<Size = Self>
        // removing these is 0.2 seconds faster on my machine
        + AsRef<[T]>
        + AsMut<[T]>
        + IntoIterator<Item = T>
        + core::borrow::Borrow<[T]>
        + core::borrow::BorrowMut<[T]>
        + From<crate::Array<T, Self>>
        + core::ops::Index<usize>
        + core::ops::Index<core::ops::Range<usize>>
        + core::ops::IndexMut<usize>
        + core::ops::IndexMut<core::ops::Range<usize>>
        + Into<crate::Array<T, Self>>
        ;
}

pub trait AssocArraySize: Sized {
    type Size: ArraySize;
}
