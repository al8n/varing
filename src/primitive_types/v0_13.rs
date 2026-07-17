use ::primitive_types_0_13::{U128, U256, U512};
use bnum_0_13::types;

macro_rules! impl_varint_for {
  ($($ty:ident), +$(,)?) => {
    paste::paste! {
      $(
        #[doc = "Returns the encoded length of the `" $ty "` value."]
        #[inline]
        pub const fn [<encoded_ $ty:snake _len>](val: &$ty) -> ::core::num::NonZeroUsize {
          $crate::bnum::encoded_uint_d64_len(&types::$ty::from_digits(val.0))
        }

        #[doc = "Encodes the `" $ty "` value."]
        #[inline]
        pub const fn [<encode_ $ty:snake _to>](
          val: &$ty,
          buf: &mut [u8],
        ) -> Result<::core::num::NonZeroUsize, $crate::ConstEncodeError> {
          $crate::bnum::encode_uint_d64_to(types::$ty::from_digits(val.0), buf)
        }

        #[doc = "Decodes the `" $ty "` from the given buffer"]
        ///
        /// Returns the bytes read and the value.
        #[inline]
        pub const fn [<decode_ $ty:snake>](buf: &[u8]) -> Result<(::core::num::NonZeroUsize, $ty), $crate::ConstDecodeError> {
          match $crate::bnum::decode_uint_d64(buf) {
            Ok((read, val)) => Ok((read, $ty(*val.digits()))),
            Err(e) => Err(e),
          }
        }

        impl $crate::Varint for $ty {
          const MIN_ENCODED_LEN: ::core::num::NonZeroUsize = types::$ty::MIN_ENCODED_LEN;

          const MAX_ENCODED_LEN: ::core::num::NonZeroUsize = types::$ty::MAX_ENCODED_LEN;

          fn encoded_len(&self) -> ::core::num::NonZeroUsize {
            [<encoded_ $ty:snake _len>](self)
          }

          fn encode(&self, buf: &mut [u8]) -> Result<::core::num::NonZeroUsize, $crate::EncodeError> {
            [<encode_ $ty:snake _to>](self, buf).map_err(Into::into)
          }

          fn decode(buf: &[u8]) -> Result<(::core::num::NonZeroUsize, Self), $crate::DecodeError>
            where
              Self: Sized {
            types::$ty::decode(buf).map(|(len, value)| (len, $ty(value.into()))).map_err(Into::into)
          }
        }
      )*
    }
  };
}

impl_varint_for!(U128, U256, U512,);

#[cfg(test)]
mod tests;
