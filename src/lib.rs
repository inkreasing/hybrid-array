#![no_std]

pub mod sizes;

pub struct Array<T, U: ArraySize>(pub U::ArrayType<T>);

impl<T, U, const N: usize> From<[T; N]> for Array<T, U>
where
    U: ArraySize<ArrayType<T> = [T; N]>,
{
    fn from(arr: [T; N]) -> Array<T, U> {
        Array(arr)
    }
}

impl<T, U, const N: usize> From<Array<T, U>> for [T; N]
where
    U: ArraySize<ArrayType<T> = [T; N]>,
{
    fn from(arr: Array<T, U>) -> [T; N] {
        arr.0
    }
}

pub unsafe trait ArraySize: typenum::Unsigned {
    type ArrayType<T>:
        // commenting these out makes the new solver a lot faster
        From<crate::Array<T, Self>>
        + Into<crate::Array<T, Self>>
        ;
}
