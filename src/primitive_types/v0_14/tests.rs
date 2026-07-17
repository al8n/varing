use crate::Varint;
use quickcheck::Arbitrary;

use super::*;

#[derive(Debug, Clone, Copy)]
struct ArbitraryType<T>(T);

impl<T> Arbitrary for ArbitraryType<T>
where
  T: Arbitrary,
{
  fn arbitrary(g: &mut quickcheck::Gen) -> Self {
    Self(T::arbitrary(g))
  }
}

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

macro_rules! fuzzy_test {
  ($($ty:literal), +$(,)?) => {
    paste::paste! {
      $(
        #[quickcheck_macros::quickcheck]
        fn [< fuzzy_ $ty:snake _varint >](value: [< AU $ty >]) -> bool {
          let value: [< U $ty >] = ::core::convert::Into::into(value);
          let mut buf = [0; <[< U $ty >]>::MAX_ENCODED_LEN.get()];
          let Ok(encoded_len) = value.encode(&mut buf) else { return false; };
          if encoded_len != value.encoded_len() || !(value.encoded_len() <= <[< U $ty >]>::MAX_ENCODED_LEN) {
            return false;
          }

          let Some(consumed) = $crate::consume_varint_checked(&buf) else {
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
          let mut buf = [0; <[< U $ty >]>::MAX_ENCODED_LEN.get()];
          let Ok(encoded) = [< encode_u $ty:snake _to>](&value, &mut buf) else { return false; };
          if encoded != [< encoded_u $ty:snake _len >] (&value) || !(encoded <= <[< U $ty >]>::MAX_ENCODED_LEN) {
            return false;
          }

          let Some(consumed) = $crate::consume_varint_checked(&buf) else {
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

fuzzy_test!(128, 256, 512);
