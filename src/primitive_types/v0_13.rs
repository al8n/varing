use ::primitive_types_0_13::{U128, U256, U512};
use ruint_1::aliases;

macro_rules! impl_varint_for {
  ($($ty:ident), +$(,)?) => {
    $(
      impl $crate::Varint for $ty {
        const MIN_ENCODED_LEN: usize = aliases::$ty::MIN_ENCODED_LEN;

        const MAX_ENCODED_LEN: usize = aliases::$ty::MAX_ENCODED_LEN;

        fn encoded_len(&self) -> usize {
          aliases::$ty::from_limbs(self.0).encoded_len()
        }

        fn encode(&self, buf: &mut [u8]) -> Result<usize, $crate::EncodeError> {
          aliases::$ty::from_limbs(self.0).encode(buf)
        }

        fn decode(buf: &[u8]) -> Result<(usize, Self), $crate::DecodeError>
          where
            Self: Sized {
          aliases::$ty::decode(buf).map(|(len, value)| (len, $ty(value.into_limbs())))
        }
      }
    )*
  };
}

impl_varint_for!(U128, U256, U512,);
