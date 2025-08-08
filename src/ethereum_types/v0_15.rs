use ::ethereum_types_0_15::U64;

type BU64 = bnum_0_13::BUint<1>;

macro_rules! impl_varint_for {
  ($($ty:ident <=> $target:ident), +$(,)?) => {
    paste::paste! {
      $(
        #[doc = "Returns the encoded length of the `" $ty "` value."]
        #[inline]
        pub const fn [<encoded_ $ty:snake _len>](val: &$ty) -> usize {
          $crate::bnum::encoded_uint_d64_len(&$target::from_digits(val.0))
        }

        #[doc = "Encodes the `" $ty "` value."]
        #[inline]
        pub const fn [<encode_ $ty:snake _to>](
          val: &$ty,
          buf: &mut [u8],
        ) -> Result<usize, $crate::ConstEncodeError> {
          $crate::bnum::encode_uint_d64_to($target::from_digits(val.0), buf)
        }

        #[doc = "Decodes the `" $ty "` from the given buffer"]
        ///
        /// Returns the bytes read and the value.
        #[inline]
        pub const fn [<decode_ $ty:snake>](buf: &[u8]) -> Result<(usize, $ty), $crate::ConstDecodeError> {
          match $crate::bnum::decode_uint_d64(buf) {
            Ok((read, val)) => Ok((read, $ty(*val.digits()))),
            Err(e) => Err(e),
          }
        }

        impl $crate::Varint for $ty {
          const MIN_ENCODED_LEN: usize = $target::MIN_ENCODED_LEN;

          const MAX_ENCODED_LEN: usize = $target::MAX_ENCODED_LEN;

          fn encoded_len(&self) -> usize {
            [<encoded_ $ty:snake _len>](self)
          }

          fn encode(&self, buf: &mut [u8]) -> Result<usize, $crate::EncodeError> {
            [<encode_ $ty:snake _to>](self, buf).map_err(Into::into)
          }

          fn decode(buf: &[u8]) -> Result<(usize, Self), $crate::DecodeError>
            where
              Self: Sized {
            $target::decode(buf).map(|(len, value)| (len, $ty(value.into()))).map_err(Into::into)
          }
        }
      )*
    }
  };
}

impl_varint_for!(U64 <=> BU64);

#[cfg(not(feature = "primitive-types_0_13"))]
use ::ethereum_types_0_15::{U128, U256, U512};
#[cfg(not(feature = "primitive-types_0_13"))]
use bnum_0_13::types::{U128 as BU128, U256 as BU256, U512 as BU512};
#[cfg(not(feature = "primitive-types_0_13"))]
impl_varint_for!(U128 <=> BU128, U256 <=> BU256, U512 <=> BU512,);

#[cfg(test)]
#[derive(Debug, Clone, Copy)]
struct ArbitraryType<T>(T);

#[cfg(test)]
const _: () = {
  use quickcheck::Arbitrary;

  impl<T> Arbitrary for ArbitraryType<T>
  where
    T: Arbitrary,
  {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
      Self(T::arbitrary(g))
    }
  }
};

#[cfg(test)]
macro_rules! fuzzy_test {
  ($($ty:literal), +$(,)?) => {
    paste::paste! {
      $(
        #[quickcheck_macros::quickcheck]
        fn [< fuzzy_ $ty:snake _varint >](value: [< AU $ty >]) -> bool {
          let value: [< U $ty >] = ::core::convert::Into::into(value);
          let mut buf = [0; <[< U $ty >]>::MAX_ENCODED_LEN];
          let Ok(encoded_len) = value.encode(&mut buf) else { return false; };
          if encoded_len != value.encoded_len() || !(value.encoded_len() <= <[< U $ty >]>::MAX_ENCODED_LEN) {
            return false;
          }

          let Ok(consumed) = $crate::consume_varint(&buf) else {
            return false;
          };
          if consumed != encoded_len {
            return false;
          }

          if let Ok((bytes_read, decoded)) = <[< U $ty >]>::decode(&buf) {
            value == decoded && encoded_len == bytes_read
          } else {
            false
          }
        }

        #[quickcheck_macros::quickcheck]
        fn [< fuzzy_ $ty:snake >](value: [< AU $ty >]) -> bool {
          let value: [< U $ty >] = ::core::convert::Into::into(value);
          let mut buf = [0; <[< U $ty >]>::MAX_ENCODED_LEN];
          let Ok(encoded) = [< encode_u $ty:snake _to>](&value, &mut buf) else { return false; };
          if encoded != [< encoded_u $ty:snake _len >] (&value) || !(encoded <= <[< U $ty >]>::MAX_ENCODED_LEN) {
            return false;
          }

          let Ok(consumed) = $crate::consume_varint(&buf) else {
            return false;
          };
          if consumed != encoded {
            return false;
          }

          if let Ok((bytes_read, decoded)) = [< decode_u $ty:snake >](&buf) {
            value == decoded && encoded == bytes_read
          } else {
            false
          }
        }
      )*
    }
  };
}

#[cfg(test)]
mod t {
  use super::*;
  use crate::Varint;

  type AU64 = ArbitraryType<BU64>;

  impl From<AU64> for U64 {
    fn from(arbitrary: AU64) -> Self {
      Self(arbitrary.0.into())
    }
  }

  fuzzy_test!(64);
}

#[cfg(test)]
#[cfg(not(feature = "primitive-types_0_13"))]
mod tests {
  use crate::Varint;

  use super::*;

  type AU128 = ArbitraryType<BU128>;
  type AU256 = ArbitraryType<BU256>;
  type AU512 = ArbitraryType<BU512>;

  type BU128 = bnum_0_13::types::U128;
  type BU256 = bnum_0_13::types::U256;
  type BU512 = bnum_0_13::types::U512;

  impl From<AU128> for U128 {
    fn from(arbitrary: AU128) -> Self {
      Self(arbitrary.0.into())
    }
  }

  impl From<AU256> for U256 {
    fn from(arbitrary: AU256) -> Self {
      Self(arbitrary.0.into())
    }
  }

  impl From<AU512> for U512 {
    fn from(arbitrary: AU512) -> Self {
      Self(arbitrary.0.into())
    }
  }

  fuzzy_test!(128, 256, 512);
}
