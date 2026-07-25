#![no_std]

pub mod sizes;

pub struct Array<T, U: ArraySize>(pub U::ArrayType<T>);

pub unsafe trait ArraySize: typenum::Unsigned {
    type ArrayType<T>:
    MyFrom<crate::Array<T, Self>>
    // these don't make a compile time difference
    // + MyFrom2<crate::Array<T, Self>>
    // + MyFrom3<crate::Array<T, Self>>
    // + MyFrom4<crate::Array<T, Self>>
    ;
}

pub trait MyFrom<T> {}

impl<T, U, const N: usize> MyFrom<Array<T, U>> for [T; N]
where
    U: ArraySize<ArrayType<T> = [T; N]>,
{
}

// these don't make a compile time difference
// pub trait MyFrom2<T> {}

// impl<T, U, const N: usize> MyFrom2<Array<T, U>> for [T; N]
// where
//     U: ArraySize<ArrayType<T> = [T; N]>,
// {
// }
// pub trait MyFrom3<T> {}

// impl<T, U, const N: usize> MyFrom3<Array<T, U>> for [T; N]
// where
//     U: ArraySize<ArrayType<T> = [T; N]>,
// {
// }
// pub trait MyFrom4<T> {}

// impl<T, U, const N: usize> MyFrom4<Array<T, U>> for [T; N]
// where
//     U: ArraySize<ArrayType<T> = [T; N]>,
// {
// }
