use arbitrary_int_1::*;

use crate::*;

macro_rules! generate_fns {
  ($($underlying:ident($($inner:ident), +$(,)?)),+$(,)?) => {
    $(
      $(
        paste::paste! {
          #[doc = "A buffer for storing LEB128 encoded " $inner " values."]
          #[derive(Copy, Clone, Eq)]
          pub struct [< $inner:camel VarintBuffer >]([u8; $inner::MAX_ENCODED_LEN + 1]);

          impl PartialEq for [< $inner:camel VarintBuffer >] {
            fn eq(&self, other: &Self) -> bool {
              self.as_bytes().eq(other.as_bytes())
            }
          }

          impl core::hash::Hash for [< $inner:camel VarintBuffer >] {
            fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
              self.as_bytes().hash(state)
            }
          }

          impl core::fmt::Debug for [< $inner:camel VarintBuffer >] {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
              self.0[..self.len()].fmt(f)
            }
          }

          impl [< $inner:camel VarintBuffer >] {
            const LAST_INDEX: usize = $inner::MAX_ENCODED_LEN;

            #[allow(dead_code)]
            #[inline]
            const fn new(val: $inner) -> Self {
              let mut buf = [0; $inner::MAX_ENCODED_LEN + 1];
              let len = match [< encode_ $inner _varint_to >](val, &mut buf) {
                Ok(len) => len,
                Err(_) => panic!("buffer should be large enough"),
              };
              buf[Self::LAST_INDEX] = len as u8;
              Self(buf)
            }

            /// Returns the number of bytes in the buffer.
            #[inline]
            #[allow(clippy::len_without_is_empty)]
            pub const fn len(&self) -> usize {
              self.0[Self::LAST_INDEX] as usize
            }

            /// Extracts a slice from the buffer.
            #[inline]
            pub const fn as_bytes(&self) -> &[u8] {
              self.0.split_at(self.len()).0
            }
          }

          impl core::ops::Deref for [< $inner:camel VarintBuffer >] {
            type Target = [u8];

            fn deref(&self) -> &Self::Target {
              &self.0[..self.len()]
            }
          }

          impl core::borrow::Borrow<[u8]> for [< $inner:camel VarintBuffer >] {
            fn borrow(&self) -> &[u8] {
              self
            }
          }

          impl AsRef<[u8]> for [< $inner:camel VarintBuffer >] {
            fn as_ref(&self) -> &[u8] {
              self
            }
          }

          impl Varint for $inner {
            const MIN_ENCODED_LEN: usize = [< encoded_ $inner _varint_len >]($inner::MIN);
            const MAX_ENCODED_LEN: usize = [< encoded_ $inner _varint_len >]($inner::MAX);

            #[inline]
            fn encoded_len(&self) -> usize {
              [< encoded_ $inner _varint_len >](*self)
            }

            fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
              [< encode_ $inner _varint_to >](*self, buf)
            }

            #[inline]
            fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError> {
              [< decode_ $inner _varint >](buf)
            }
          }

          /// Returns the encoded length of the value in LEB128 variable length format.
          #[doc = "The returned value will be in range of [`" $inner "::ENCODED_LEN_RANGE`]."]
          #[inline]
          pub const fn [< encoded_ $inner _varint_len >](value: $inner) -> usize {
            [<encoded_ $underlying _varint_len>](value.value())
          }

          #[doc = "Encodes an `" $inner "` value into LEB128 variable length format, and writes it to the buffer."]
          #[inline]
          pub const fn [< encode_ $inner _varint >](x: $inner) -> [< $inner:camel VarintBuffer >] {
            [< $inner:camel VarintBuffer >]::new(x)
          }

          #[doc = "Encodes an `" $inner "` value into LEB128 variable length format, and writes it to the buffer."]
          #[inline]
          pub const fn [< encode_ $inner _varint_to >](value: $inner, buf: &mut [u8]) -> Result<usize, EncodeError> {
            [<encode_ $underlying _varint_to>](value.value(), buf)
          }

          #[doc = "Decodes an `" $inner "` in LEB128 encoded format from the buffer."]
          ///
          /// Returns the bytes readed and the decoded value if successful.
          #[inline]
          pub const fn [< decode_ $inner _varint >](buf: &[u8]) -> Result<(usize, $inner), DecodeError> {
            match [<decode_ $underlying _varint>](buf) {
              Ok((readed, val)) => {
                match $inner::try_new(val) {
                  Ok(val) => Ok((readed, val)),
                  Err(_) => Err(DecodeError::Overflow),
                }
              },
              Err(err) => Err(err),
            }
          }

          #[test]
          fn [< test_ $inner _min_max_varint >]() {
            let min = $inner::MIN;
            let max = $inner::MAX;
            let min_encoded_len = [< encoded_ $inner _varint_len >](min);
            let max_encoded_len = [< encoded_ $inner _varint_len >](max);

            assert_eq!(min_encoded_len, $inner::MIN_ENCODED_LEN);
            assert_eq!(max_encoded_len, $inner::MAX_ENCODED_LEN);

            let mut buf = [0; $inner::MAX_ENCODED_LEN];
            let len = [< encode_ $inner _varint_to >](min, &mut buf).unwrap();
            assert_eq!(len, min_encoded_len);
            let buffer = [< encode_ $inner _varint >](min);
            assert_eq!(buffer.len(), min_encoded_len);
            assert_eq!(buffer.as_bytes(), &buf[..min_encoded_len]);

            let (readed, val) = [< decode_ $inner _varint >](&buf).unwrap();
            assert_eq!(readed, len);
            assert_eq!(val, min);

            let len = [< encode_ $inner _varint_to >](max, &mut buf).unwrap();
            assert_eq!(len, max_encoded_len);
            let buffer = [< encode_ $inner _varint >](max);
            assert_eq!(buffer.len(), max_encoded_len);
            assert_eq!(buffer.as_bytes(), &buf[..max_encoded_len]);

            let (readed, val) = [< decode_ $inner _varint >](&buf).unwrap();
            assert_eq!(readed, len);
            assert_eq!(val, max);
          }
        }
      )*
    )*
  };
}

generate_fns!(
  u8(u1, u2, u3, u4, u5, u6, u7),
  u16(u9, u10, u11, u12, u13, u14, u15),
);

seq_macro::seq!(N in 17..=31 {
  generate_fns!(u32(
    #(
      u~N,
    )*
  ));
});

seq_macro::seq!(N in 33..=63 {
  generate_fns!(u64(
    #(
      u~N,
    )*
  ));
});

seq_macro::seq!(N in 65..=127 {
  generate_fns!(u128(
    #(
      u~N,
    )*
  ));
});
