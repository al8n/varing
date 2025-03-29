use num_rational_0_4::Ratio;

use crate::{DecodeError, EncodeError, Varint};

macro_rules! impl_varint_for_ratio {
  (@inner $($sign:ident::$bits:literal($merged_ty:ident)),+$(,)?) => {
    paste::paste! {
      $(
        impl Varint for Ratio<[< $sign $bits >]> {
          const MIN_ENCODED_LEN: usize = $merged_ty::MAX_ENCODED_LEN;

          const MAX_ENCODED_LEN: usize = $merged_ty::MAX_ENCODED_LEN;

          fn encoded_len(&self) -> usize {
            $crate::utils::[< pack_ $sign $bits >](*self.numer(), *self.denom()).encoded_len()
          }

          fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
            $crate::utils::[< pack_ $sign $bits >](*self.numer(), *self.denom()).encode(buf)
          }

          fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError>
          where
            Self: Sized,
          {
            let (bytes_read, merged) = $merged_ty::decode(buf)?;
            let (numer, denom) = $crate::utils::[< unpack_ $sign $bits >](merged);
            if denom == 0 {
              return Err(DecodeError::custom("denominator cannot be zero"));
            }
            Ok((bytes_read, Ratio::new_raw(numer, denom)))
          }
        }
      )*
    }
  };
  ($($bits:literal($merged_ty:ident)),+$(,)?) => {
    impl_varint_for_ratio!(@inner $(u::$bits($merged_ty)),+);
    impl_varint_for_ratio!(@inner $(i::$bits($merged_ty)),+);
  };
  (@const_inner $($sign:ident::$bits:literal($merged_ty:ident)),+$(,)?) => {
    paste::paste! {
      $(
        #[doc = "Returns the encoded length of the `Ratio<" $sign $bits ">` value."]
        #[inline]
        pub const fn [< encoded_ratio_ $sign $bits _len>](val: Ratio<[< $sign $bits >]>) -> usize {
          $crate::[< encoded_ $merged_ty _varint_len >] ($crate::utils::[< pack_ $sign $bits >](*val.numer(), *val.denom()))
        }

        #[doc = "Encodes the `Ratio<" $sign $bits ">` value."]
        #[inline]
        pub const fn [< encode_ratio_ $sign $bits >](val: Ratio<[< $sign $bits >]>) -> $crate::[< $merged_ty:camel VarintBuffer>] {
          $crate::[< encode_ $merged_ty _varint>] ($crate::utils::[< pack_ $sign $bits >](*val.numer(), *val.denom()))
        }

        #[doc = "Encodes the `Ratio<" $sign $bits ">` value into the provided buffer."]
        #[inline]
        pub const fn [< encode_ratio_ $sign $bits _to >](val: Ratio<[< $sign $bits >]>, buf: &mut [u8]) -> Result<usize, EncodeError> {
          $crate::[< encode_ $merged_ty _varint_to>] ($crate::utils::[< pack_ $sign $bits >](*val.numer(), *val.denom()), buf)
        }

        #[doc = "Decodes the `Ratio<" $sign $bits ">` value from the provided buffer."]
        #[inline]
        pub const fn [< decode_ratio_ $sign $bits >](buf: &[u8]) -> Result<(usize, Ratio<[< $sign $bits >]>), DecodeError> {
          match $crate::[< decode_ $merged_ty _varint >](buf) {
            Ok((bytes_read, merged)) => {
              let (numer, denom) = $crate::utils::[< unpack_ $sign $bits >](merged);
              if denom == 0 {
                Err(DecodeError::custom("denominator cannot be zero"))
              } else {
                Ok((bytes_read, Ratio::new_raw(numer, denom)))
              }
            }
            Err(e) => Err(e),
          }
        }
      )*
    }
  };
  (@const $($bits:literal($merged_ty:ident)),+$(,)?) => {
    impl_varint_for_ratio!(@const_inner $(u::$bits($merged_ty)),+);
    impl_varint_for_ratio!(@const_inner $(i::$bits($merged_ty)),+);
  }
}

impl_varint_for_ratio!(8(u16), 16(u32), 32(u64), 64(u128));
impl_varint_for_ratio!(@const 8(u16), 16(u32), 32(u64), 64(u128));

#[cfg(feature = "ruint_1")]
mod ruint_1;

#[cfg(feature = "bnum_0_13")]
mod bnum_0_13;

#[cfg(test)]
mod tests;
