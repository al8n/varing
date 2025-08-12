use super::*;
use ::ruint_1::aliases::*;

impl_arbitrary_ratio!(@ruint (U64, U128, U192, U256, U384, U448, U512, U768, U1024, U2048, U4096,));

fuzzy!(@varint_into (
  RatioU128(Ratio<u128>),
  RatioI128(Ratio<i128>),
));

macro_rules! ratio_ruint_fuzzy {
  (@varint_into ($($ty:ident($target:ty)), +$(,)?)) => {
    $(
      paste::paste! {
        #[quickcheck_macros::quickcheck]
        fn [< fuzzy_ $ty:snake _varint>](value: $ty) -> bool {
          let value: $target = ::core::convert::Into::into(value);
          let mut buf = [0; <$target>::MAX_ENCODED_LEN.get()];
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

ratio_ruint_fuzzy!(@varint_into (
  RUintRatioU64(Ratio<U64>),
  RUintRatioU128(Ratio<U128>),
  RUintRatioU192(Ratio<U192>),
  RUintRatioU256(Ratio<U256>),
  RUintRatioU384(Ratio<U384>),
  RUintRatioU448(Ratio<U448>),
  RUintRatioU512(Ratio<U512>),
  RUintRatioU768(Ratio<U768>),
  RUintRatioU1024(Ratio<U1024>),
  RUintRatioU2048(Ratio<U2048>),
  RUintRatioU4096(Ratio<U4096>),
));
