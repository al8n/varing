#![allow(warnings)]

use super::*;

use quickcheck::{Arbitrary, Gen};

#[derive(Clone, Copy, Debug)]
struct ArbitraryComplex<T> {
  re: T,
  im: T,
}

macro_rules! impl_arbitrary_complex {
    ($($ty:ty),+$(,)?) => {
      paste::paste! {
        $(
          impl Arbitrary for ArbitraryComplex<$ty> {
            fn arbitrary(g: &mut Gen) -> Self {
              let re = <$ty>::arbitrary(g);
              let im = <$ty>::arbitrary(g);
              Self { re, im }
            }
          }

          impl From<ArbitraryComplex<$ty>> for Complex<$ty> {
            fn from(arbitrary: ArbitraryComplex<$ty>) -> Self {
              Complex { re: arbitrary.re, im: arbitrary.im }
            }
          }

          type [< Complex $ty:camel >] = ArbitraryComplex<$ty>;
        )*
      }
    };
    (@ruint ($($ty:ty),+$(,)?)) => {
      paste::paste! {
        $(
          impl Arbitrary for ArbitraryComplex<$ty> {
            fn arbitrary(g: &mut Gen) -> Self {
              let re = <$ty>::arbitrary(g);
              let im = <$ty>::arbitrary(g);
              Self { re, im }
            }
          }

          impl From<ArbitraryComplex<$ty>> for Complex<$ty> {
            fn from(arbitrary: ArbitraryComplex<$ty>) -> Self {
              Complex { re: arbitrary.re, im: arbitrary.im }
            }
          }

          type [< RUintComplex $ty:camel >] = ArbitraryComplex<$ty>;
        )*
      }
    };
    (@bnum ($($ty:ty),+$(,)?)) => {
      paste::paste! {
        $(
          impl Arbitrary for ArbitraryComplex<$ty> {
            fn arbitrary(g: &mut Gen) -> Self {
              let re = <$ty>::arbitrary(g);
              let im = <$ty>::arbitrary(g);
              Self { re, im }
            }
          }

          impl From<ArbitraryComplex<$ty>> for Complex<$ty> {
            fn from(arbitrary: ArbitraryComplex<$ty>) -> Self {
              Complex { re: arbitrary.re, im: arbitrary.im }
            }
          }

          type [< BnumComplex $ty:camel >] = ArbitraryComplex<$ty>;
        )*
      }
    };
  }

impl_arbitrary_complex!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128,);

fuzzy!(@const_varint_into (ComplexU8(Complex<u8>), ComplexU16(Complex<u16>), ComplexU32(Complex<u32>), ComplexU64(Complex<u64>), ComplexI8(Complex<i8>), ComplexI16(Complex<i16>), ComplexI32(Complex<i32>), ComplexI64(Complex<i64>),));

fuzzy!(@varint_into (
  ComplexU8(Complex<u8>),
  ComplexU16(Complex<u16>),
  ComplexU32(Complex<u32>),
  ComplexU64(Complex<u64>),
  ComplexI8(Complex<i8>),
  ComplexI16(Complex<i16>),
  ComplexI32(Complex<i32>),
  ComplexI64(Complex<i64>),
));

#[cfg(feature = "ruint_1")]
mod ruint_1;

#[cfg(feature = "bnum_0_13")]
mod bnum_0_13;
