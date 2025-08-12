use super::*;

use ::bnum_0_13::*;

fuzzy!(@varint_into(
  RatioI128(Ratio<i128>),
  RatioU128(Ratio<u128>),
));

#[quickcheck_macros::quickcheck]
fn fuzzy_ratio_u128(value: ArbitraryRatio<u128>) -> bool {
  let value = ::core::convert::Into::into(value);
  let mut buf = [0; <Ratio<u128>>::MAX_ENCODED_LEN];
  let encoded = encode_ratio_u128_to(&value, &mut buf).unwrap();
  if encoded != encoded_ratio_u128_len(&value) || !(encoded <= <Ratio<u128>>::MAX_ENCODED_LEN) {
    return false;
  }

  let Some(consumed) = crate::consume_varint_checked(&buf) else {
    return false;
  };
  if consumed != encoded {
    return false;
  }

  if let Ok((bytes_read, decoded)) = decode_ratio_u128(&buf) {
    value == decoded && encoded == bytes_read
  } else {
    false
  }
}

#[quickcheck_macros::quickcheck]
fn fuzzy_ratio_i128(value: ArbitraryRatio<i128>) -> bool {
  let value = ::core::convert::Into::into(value);
  let mut buf = [0; <Ratio<i128>>::MAX_ENCODED_LEN];
  let encoded = encode_ratio_i128_to(&value, &mut buf).unwrap();
  if encoded != encoded_ratio_i128_len(&value) || !(encoded <= <Ratio<i128>>::MAX_ENCODED_LEN) {
    return false;
  }

  let Some(consumed) = crate::consume_varint_checked(&buf) else {
    return false;
  };
  if consumed != encoded {
    return false;
  }

  if let Ok((bytes_read, decoded)) = decode_ratio_i128(&buf) {
    value == decoded && encoded == bytes_read
  } else {
    false
  }
}

macro_rules! ratio_bnum_fuzzy {
  (@varint_into ($($ty:ident($target:ty)), +$(,)?)) => {
    $(
      paste::paste! {
        #[quickcheck_macros::quickcheck]
        fn [< fuzzy_ $ty:snake _varint>](value: $ty) -> bool {
          let value: $target = ::core::convert::Into::into(value);
          let mut buf = [0; <$target>::MAX_ENCODED_LEN];
          let Ok(encoded_len) = value.encode(&mut buf) else { return false; };
          if encoded_len != value.encoded_len() || !(value.encoded_len() <= <$target>::MAX_ENCODED_LEN) {
            return false;
          }

          let Some(consumed) = $crate::consume_varint_checked(&buf) else {
            return false;
          };
          if consumed != encoded_len {
            return false;
          }

          if let Ok((bytes_read, decoded)) = <$target>::decode(&buf) {
            value.numer() == decoded.numer() && value.denom() == decoded.denom() && encoded_len == bytes_read
          } else {
            false
          }
        }
      }
    )*
  };
}

macro_rules! type_aliases {
  ($sign:ident::$storage:literal::$base:ident($($bits:literal), + $(,)?)) => {
    paste::paste! {
      $(
        type [<$sign $bits>] = $base<{ $bits / 8 }>;
      )*
    }
  };
}

macro_rules! test_mod {
  ($($sign:ident::$base:ident::$storage:literal),+$(,)?) => {
    paste::paste! {
      $(
        mod [<$base:snake>] {
          use super::*;
          use ::bnum_0_13::$base;

          type_aliases!(U::$storage::$base(8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192));

          impl_arbitrary_ratio!(@bnum (U8, U16, U32, U64, U128, U256, U512, U1024, U2048));

          ratio_bnum_fuzzy!(@varint_into (
            BnumRatioU8(Ratio<U8>),
            BnumRatioU16(Ratio<U16>),
            BnumRatioU32(Ratio<U32>),
            BnumRatioU64(Ratio<U64>),
            BnumRatioU128(Ratio<U128>),
            BnumRatioU256(Ratio<U256>),
            BnumRatioU512(Ratio<U512>),
            BnumRatioU1024(Ratio<U1024>),
            BnumRatioU2048(Ratio<U2048>),
          ));
        }
      )*
    }
  };
}

test_mod!(
  U::BUintD8::8,
  U::BUintD16::16,
  U::BUintD32::32,
  U::BUint::64,
  I::BIntD8::8,
  I::BIntD16::16,
  I::BIntD32::32,
  I::BInt::64,
);
