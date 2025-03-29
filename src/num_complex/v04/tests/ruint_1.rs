use super::*;

use ::ruint_1::aliases::*;

fuzzy!(@varint_into(
  ComplexI128(Complex<i128>),
  ComplexU128(Complex<u128>),
));

impl_arbitrary_complex!(@ruint (U64, U128, U192, U256, U384, U448, U512, U768, U1024, U2048, U4096,));

macro_rules! complex_ruint_fuzzy {
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

complex_ruint_fuzzy!(@varint_into (
  RUintComplexU64(Complex<U64>),
  RUintComplexU128(Complex<U128>),
  RUintComplexU192(Complex<U192>),
  RUintComplexU256(Complex<U256>),
  RUintComplexU384(Complex<U384>),
  RUintComplexU448(Complex<U448>),
  RUintComplexU512(Complex<U512>),
  RUintComplexU768(Complex<U768>),
  RUintComplexU1024(Complex<U1024>),
  RUintComplexU2048(Complex<U2048>),
  RUintComplexU4096(Complex<U4096>),
));
