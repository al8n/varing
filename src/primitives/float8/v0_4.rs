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
mod tests {
  use super::*;

  macro_rules! fuzzy_quickcheck {
    ($($ty:ident),+$(,)?) => {
      paste::paste! {
        $(
          #[derive(Debug, Clone, Copy)]
          struct [< Fuzzy $ty >]($ty);

          impl quickcheck::Arbitrary for [< Fuzzy $ty >] {
            fn arbitrary(g: &mut quickcheck::Gen) -> Self {
              loop {
                let val = <$ty>::from_bits(u8::arbitrary(g));
                if !val.is_nan() {
                  break Self(val);
                }
              }
            }
          }

          quickcheck::quickcheck! {
            fn [< fuzzy_ $ty:lower _varint >](value: [< Fuzzy $ty >]) -> bool {
              let value = value.0;
              let mut buf = [0u8; { <$ty>::MAX_ENCODED_LEN.get() + 1 }];
              let len = value.encoded_len();
              let len2 = value.encode(&mut buf).unwrap();
              assert_eq!(len, len2);
              let (read, value2) = [< decode_ $ty:lower _varint >](&buf[..len.get()]).unwrap();
              assert_eq!(len, read);
              assert_eq!(value, value2);

              [< encode_ $ty:lower _varint >](value).as_slice() == &buf[..len.get()]
            }
          }
        )*
      }
    };
  }

  fuzzy_quickcheck!(F8E4M3, F8E5M2);

  #[cfg(feature = "std")]
  mod with_std {
    use super::*;

    macro_rules! fuzzy_sequence {
      ($($ty:ident), +$(,)?) => {
        paste::paste! {
          $(
            quickcheck::quickcheck! {
              fn [< fuzzy_ $ty:lower _sequence >](value: std::vec::Vec<[< Fuzzy $ty >]>) -> bool {
                let value = value.into_iter().map(|v| v.0).collect::<std::vec::Vec<_>>();
                let encoded_len = [< encoded_ $ty:lower _sequence_len >](&value);
                let mut buf = std::vec![0; encoded_len];
                let Ok(written) = [< encode_ $ty:lower _sequence_to >](&value, &mut buf) else { return false; };
                if encoded_len != written {
                  return false;
                }

                let (readed, decoded) = crate::decode_sequence::<$ty, std::vec::Vec<_>>(&buf).unwrap();
                if encoded_len != readed {
                  return false;
                }

                assert_eq!(decoded.len(), value.len());

                for (a, b) in decoded.iter().zip(value.iter()) {
                  if a.to_bits() != b.to_bits() {
                    return false;
                  }
                }

                true
              }
            }
          )*
        }
      };
    }

    fuzzy_sequence!(F8E4M3, F8E5M2);
  }
}
