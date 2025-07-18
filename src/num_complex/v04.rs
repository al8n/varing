use num_complex_0_4::Complex;

use crate::{DecodeError, EncodeError, Varint};

macro_rules! impl_varint_for_complex {
  (@inner $($sign:ident::$bits:literal($merged_ty:ident)),+$(,)?) => {
    paste::paste! {
      $(
        impl Varint for Complex<[< $sign $bits >]> {
          const MIN_ENCODED_LEN: usize = $merged_ty::MAX_ENCODED_LEN;

          const MAX_ENCODED_LEN: usize = $merged_ty::MAX_ENCODED_LEN;

          fn encoded_len(&self) -> usize {
            $crate::utils::[< pack_ $sign $bits >](self.re, self.im).encoded_len()
          }

          fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
            $crate::utils::[< pack_ $sign $bits >](self.re, self.im).encode(buf)
          }

          fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError>
          where
            Self: Sized,
          {
            let (bytes_read, merged) = $merged_ty::decode(buf)?;
            let (re, im) = $crate::utils::[< unpack_ $sign $bits >](merged);
            Ok((bytes_read, Complex { re, im }))
          }
        }
      )*
    }
  };
  ($($bits:literal($merged_ty:ident)),+$(,)?) => {
    impl_varint_for_complex!(@inner $(u::$bits($merged_ty)),+);
    impl_varint_for_complex!(@inner $(i::$bits($merged_ty)),+);
  };
  (@const_inner $($sign:ident::$bits:literal($merged_ty:ident)),+$(,)?) => {
    paste::paste! {
      $(
        #[doc = "Returns the encoded length of the `Complex<" $sign $bits ">` value."]
        #[inline]
        pub const fn [< encoded_complex_ $sign $bits _len>](val: Complex<[< $sign $bits >]>) -> usize {
          $crate::[< encoded_ $merged_ty _varint_len >] ($crate::utils::[< pack_ $sign $bits >](val.re, val.im))
        }

        #[doc = "Encodes the `Complex<" $sign $bits ">` value."]
        #[inline]
        pub const fn [< encode_complex_ $sign $bits >](val: Complex<[< $sign $bits >]>) -> $crate::utils::Buffer<{ $merged_ty::MAX_ENCODED_LEN + 1 }> {
          $crate::[< encode_ $merged_ty _varint>] ($crate::utils::[< pack_ $sign $bits >](val.re, val.im))
        }

        #[doc = "Encodes the `Complex<" $sign $bits ">` value into the provided buffer."]
        #[inline]
        pub const fn [< encode_complex_ $sign $bits _to >](val: Complex<[< $sign $bits >]>, buf: &mut [u8]) -> Result<usize, EncodeError> {
          $crate::[< encode_ $merged_ty _varint_to>] ($crate::utils::[< pack_ $sign $bits >](val.re, val.im), buf)
        }

        #[doc = "Decodes the `Complex<" $sign $bits ">` value from the provided buffer."]
        #[inline]
        pub const fn [< decode_complex_ $sign $bits >](buf: &[u8]) -> Result<(usize, Complex<[< $sign $bits >]>), DecodeError> {
          match $crate::[< decode_ $merged_ty _varint >](buf) {
            Ok((bytes_read, merged)) => {
              let (re, im) = $crate::utils::[< unpack_ $sign $bits >](merged);
              Ok((bytes_read, Complex { re, im }))
            }
            Err(e) => Err(e),
          }
        }
      )*
    }
  };
  (@const $($bits:literal($merged_ty:ident)),+$(,)?) => {
    impl_varint_for_complex!(@const_inner $(u::$bits($merged_ty)),+);
    impl_varint_for_complex!(@const_inner $(i::$bits($merged_ty)),+);
  }
}

impl_varint_for_complex!(8(u16), 16(u32), 32(u64), 64(u128));
impl_varint_for_complex!(@const 8(u16), 16(u32), 32(u64), 64(u128));

macro_rules! impl_varint_for_complex_floats {
  ($($bits:literal:$target:ty: $ty:ty), +$(,)?) => {
    $(
      impl Varint for Complex<$ty> {
        const MAX_ENCODED_LEN: usize = <Complex<$target> as Varint>::MAX_ENCODED_LEN;
        const MIN_ENCODED_LEN: usize = <Complex<$target> as Varint>::MIN_ENCODED_LEN;

        fn encoded_len(&self) -> usize {
          Complex {
            re: self.re.to_bits(),
            im: self.im.to_bits(),
          }
          .encoded_len()
        }

        fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
          Complex {
            re: self.re.to_bits(),
            im: self.im.to_bits(),
          }
          .encode(buf)
        }

        fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError>
        where
          Self: Sized,
        {
          let (bytes_read, Complex { re, im }) = Complex::<$target>::decode(buf)?;
          Ok((bytes_read, Complex {
            re: <$ty>::from_bits(re),
            im: <$ty>::from_bits(im),
          }))
        }
      }

      paste::paste! {
        #[doc = "Returns the encoded length of the `Complex<f" $bits ">` value."]
        #[inline]
        pub const fn [< encoded_complex_ $ty _len>](val: Complex<$ty>) -> usize {
          $crate::num_complex::v04::[< encoded_complex_ $target _len >] (Complex {
            re: val.re.to_bits(),
            im: val.im.to_bits(),
          })
        }

        #[doc = "Encodes the `Complex<f" $bits ">` value."]
        #[inline]
        pub const fn [< encode_complex_f $bits >](val: Complex<$ty>) -> $crate::utils::Buffer<{ Complex::<$target>::MAX_ENCODED_LEN + 1 }> {
          $crate::num_complex::v04::[< encode_complex_ $target>] (Complex {
            re: val.re.to_bits(),
            im: val.im.to_bits(),
          })
        }

        #[doc = "Encodes the `Complex<f" $bits ">` value into the provided buffer."]
        #[inline]
        pub const fn [< encode_complex_ $ty _to >](val: Complex<$ty>, buf: &mut [u8]) -> Result<usize, EncodeError> {
          $crate::num_complex::v04::[< encode_complex_ $target _to>] (Complex {
            re: val.re.to_bits(),
            im: val.im.to_bits(),
          }, buf)
        }

        #[doc = "Decodes the `Complex<f" $bits ">` value from the provided buffer."]
        #[inline]
        pub const fn [< decode_complex_ $ty >](buf: &[u8]) -> Result<(usize, Complex<$ty>), DecodeError> {
          match $crate::num_complex::v04::[< decode_complex_ $target >](buf) {
            Ok((bytes_read, Complex {re, im})) => {
              Ok((bytes_read, Complex {
                re: <$ty>::from_bits(re),
                im: <$ty>::from_bits(im),
              }))
            }
            Err(e) => Err(e),
          }
        }
      }
    )*
  };
}

impl_varint_for_complex_floats!(
  32:u32:f32,
  64:u64:f64,
);

#[cfg(feature = "half_2")]
use half_2::f16;
#[cfg(feature = "half_2")]
impl_varint_for_complex_floats!(
  16:u16:f16,
);

#[cfg(feature = "bnum_0_13")]
mod bnum_0_13;
#[cfg(feature = "bnum_0_13")]
#[cfg_attr(docsrs, doc(cfg(feature = "bnum_0_13")))]
pub use bnum_0_13::*;

#[cfg(feature = "ruint_1")]
mod ruint_1;

#[cfg(test)]
mod tests;
