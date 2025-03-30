use super::*;

use ::bnum_0_13::*;

fuzzy!(@varint_into(
  ComplexI128(Complex<i128>),
  ComplexU128(Complex<u128>),
));

macro_rules! complex_bnum_fuzzy {
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

          let Ok(consumed) = $crate::consume_varint(&buf) else {
            return false;
          };
          if consumed != encoded_len {
            return false;
          }

          if let Ok((bytes_read, decoded)) = <$target>::decode(&buf) {
            value.re == decoded.re && value.im == decoded.im && encoded_len == bytes_read
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

          type_aliases!(U::$storage::$base(8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768));

          impl_arbitrary_complex!(@bnum (U8, U16, U32, U64, U128, U256, U512, U1024, U2048, U4096,));

          complex_bnum_fuzzy!(@varint_into (
            BnumComplexU8(Complex<U8>),
            BnumComplexU16(Complex<U16>),
            BnumComplexU32(Complex<U32>),
            BnumComplexU64(Complex<U64>),
            BnumComplexU128(Complex<U128>),
            BnumComplexU256(Complex<U256>),
            BnumComplexU512(Complex<U512>),
            BnumComplexU1024(Complex<U1024>),
            BnumComplexU2048(Complex<U2048>),
            BnumComplexU4096(Complex<U4096>),
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
