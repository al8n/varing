#![allow(warnings)]

use super::*;

use quickcheck::{Arbitrary, Gen};

#[derive(Clone, Copy, Debug)]
struct ArbitraryRatio<T> {
  numer: T,
  denom: T,
}

macro_rules! impl_arbitrary_ratio {
    ($($ty:ty),+$(,)?) => {
      paste::paste! {
        $(
          impl Arbitrary for ArbitraryRatio<$ty> {
            fn arbitrary(g: &mut Gen) -> Self {
              let numer = <$ty>::arbitrary(g);
              let denom = loop {
                let denom = <$ty>::arbitrary(g);
                if denom != 0 {
                  break denom;
                }
              };
              Self { numer, denom }
            }
          }

          impl From<ArbitraryRatio<$ty>> for Ratio<$ty> {
            fn from(arbitrary: ArbitraryRatio<$ty>) -> Self {
              Ratio::new_raw(arbitrary.numer, arbitrary.denom)
            }
          }

          type [< Ratio $ty:camel >] = ArbitraryRatio<$ty>;
        )*
      }
    };
    (@ruint ($($ty:ty),+$(,)?)) => {
      paste::paste! {
        $(
          impl Arbitrary for ArbitraryRatio<$ty> {
            fn arbitrary(g: &mut Gen) -> Self {
              let numer = <$ty>::arbitrary(g);
              let denom = loop {
                let denom = <$ty>::arbitrary(g);
                if !denom.is_zero() {
                  break denom;
                }
              };
              Self { numer, denom }
            }
          }

          impl From<ArbitraryRatio<$ty>> for Ratio<$ty> {
            fn from(arbitrary: ArbitraryRatio<$ty>) -> Self {
              Ratio::new_raw(arbitrary.numer, arbitrary.denom)
            }
          }

          type [< RUintRatio $ty:camel >] = ArbitraryRatio<$ty>;
        )*
      }
    };
    (@bnum ($($ty:ty),+$(,)?)) => {
      paste::paste! {
        $(
          impl Arbitrary for ArbitraryRatio<$ty> {
            fn arbitrary(g: &mut Gen) -> Self {
              let numer = <$ty>::arbitrary(g);
              let denom = loop {
                let denom = <$ty>::arbitrary(g);
                if !denom.is_zero() {
                  break denom;
                }
              };
              Self { numer, denom }
            }
          }

          impl From<ArbitraryRatio<$ty>> for Ratio<$ty> {
            fn from(arbitrary: ArbitraryRatio<$ty>) -> Self {
              Ratio::new_raw(arbitrary.numer, arbitrary.denom)
            }
          }

          type [< BnumRatio $ty:camel >] = ArbitraryRatio<$ty>;
        )*
      }
    };
  }

impl_arbitrary_ratio!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128,);

fuzzy!(@const_varint_into (RatioU8(Ratio<u8>), RatioU16(Ratio<u16>), RatioU32(Ratio<u32>), RatioU64(Ratio<u64>), RatioI8(Ratio<i8>), RatioI16(Ratio<i16>), RatioI32(Ratio<i32>), RatioI64(Ratio<i64>),));
fuzzy!(@varint_into (
  RatioU8(Ratio<u8>),
  RatioU16(Ratio<u16>),
  RatioU32(Ratio<u32>),
  RatioU64(Ratio<u64>),
  RatioI8(Ratio<i8>),
  RatioI16(Ratio<i16>),
  RatioI32(Ratio<i32>),
  RatioI64(Ratio<i64>),
));

#[cfg(feature = "bnum_0_13")]
mod bnum_0_13;

#[cfg(feature = "ruint_1")]
mod ruint_1;
