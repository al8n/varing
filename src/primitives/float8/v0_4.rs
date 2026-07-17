use core::num::NonZeroUsize;

use float8_0_4::{F8E4M3, F8E5M2};

use crate::Varint;

macro_rules! generate {
  ($($ty:ident),+$(,)?) => {
    paste::paste! {
      $(
        impl Varint for $ty {
          const MIN_ENCODED_LEN: NonZeroUsize = u8::MIN_ENCODED_LEN;

          const MAX_ENCODED_LEN: NonZeroUsize = u8::MAX_ENCODED_LEN;

          #[inline]
          fn encoded_len(&self) -> NonZeroUsize {
            [< encoded_ $ty:lower _varint_len >](*self)
          }

          #[inline]
          fn encode(&self, buf: &mut [u8]) -> Result<NonZeroUsize, crate::EncodeError> {
            [< encode_ $ty:lower _varint_to >](*self, buf).map_err(Into::into)
          }

          #[inline]
          fn decode(buf: &[u8]) -> Result<(NonZeroUsize, Self), crate::DecodeError>
          where
            Self: Sized,
          {
            [< decode_ $ty:lower _varint >](buf).map_err(Into::into)
          }
        }

        #[doc = "Returns the encoded length of the value in LEB128 variable length format. The returned value will be in range of [`" $ty "::ENCODED_LEN_RANGE`]"]
        #[inline]
        pub const fn [< encoded_ $ty:lower _varint_len >](value: $ty) -> NonZeroUsize {
          crate::encoded_u8_varint_len(value.to_bits())
        }

        #[doc = "Encodes a `" $ty "` value into LEB128 variable length format, and writes it to the buffer."]
        #[inline]
        pub const fn [< encode_ $ty:lower _varint >](
          value: $ty,
        ) -> crate::utils::Buffer<{ <$ty>::MAX_ENCODED_LEN.get() + 1 }> {
          crate::encode_u8_varint(value.to_bits())
        }

        #[doc = "Encodes a `" $ty "` value into LEB128 variable length format, and writes it to the buffer."]
        #[inline]
        pub const fn [< encode_ $ty:lower _varint_to >](
          value: $ty,
          buf: &mut [u8],
        ) -> Result<NonZeroUsize, crate::ConstEncodeError> {
          crate::encode_u8_varint_to(value.to_bits(), buf)
        }

        #[doc = "Decodes an `" $ty "` in LEB128 encoded format from the buffer."]
        ///
        /// Returns the bytes readed and the decoded value if successful.
        #[inline]
        pub const fn [< decode_ $ty:lower _varint >](buf: &[u8]) -> Result<(NonZeroUsize, $ty), crate::ConstDecodeError> {
          match crate::decode_u8_varint(buf) {
            Ok((len, bits)) => Ok((len, <$ty>::from_bits(bits))),
            Err(e) => Err(e),
          }
        }

        #[doc = "Returns the encoded length of a sequence of `" $ty "` values."]
        #[inline]
        pub const fn [< encoded_ $ty:lower _sequence_len >](sequence: &[$ty]) -> usize {
          encode!(@sequence_encoded_len_impl sequence, [< encoded_ $ty:lower _varint_len >])
        }

        #[doc = "Encodes a sequence of `" $ty "` to the buffer."]
        #[inline]
        pub const fn [< encode_ $ty:lower _sequence_to >](
          sequence: &[$ty],
          buf: &mut [u8],
        ) -> Result<usize, crate::ConstEncodeError> {
          encode!(@sequence_encode_to_impl buf, sequence, [< encode_ $ty:lower _varint_to >], [< encoded_ $ty:lower _sequence_len >])
        }
      )*
    }
  };
}

generate!(F8E4M3, F8E5M2);

#[cfg(test)]
mod tests;
