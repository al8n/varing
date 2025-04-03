use arbitrary_int_1::*;

use crate::*;

macro_rules! generate {
  ($($storage:ident($start:literal..=$end:literal)), +$(,)?) => {
    $(
      seq_macro::seq!(N in $start..=$end {
        generate!($storage(
          #(
            u~N,
          )*
        ));
      });
    )*
  };
  ($($underlying:ident($($inner:ident), +$(,)?)),+$(,)?) => {
    $(
      $(
        paste::paste! {
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
          pub const fn [< encode_ $inner _varint >](x: $inner) -> $crate::utils::Buffer<{ $inner::MAX_ENCODED_LEN + 1 }> {
            let mut buf = [0; $inner::MAX_ENCODED_LEN + 1];
            let len = match [< encode_ $inner _varint_to >](x, &mut buf) {
              Ok(len) => len,
              Err(_) => panic!("buffer should be large enough"),
            };
            buf[$crate::utils::Buffer::<{ $inner::MAX_ENCODED_LEN + 1 }>::CAPACITY] = len as u8;
            $crate::utils::Buffer::new(buf)
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

          #[cfg(test)]
          #[derive(Debug, Clone, Copy, PartialEq, Eq)]
          struct [< Fuzzy $inner:camel >]($inner);

          #[cfg(test)]
          const _: () = {
            use quickcheck::{Arbitrary, Gen};

            impl Arbitrary for [< Fuzzy $inner:camel >] {
              fn arbitrary(g: &mut Gen) -> Self {
                let val = loop {
                  let val = $underlying::arbitrary(g);
                  if val >= $inner::MIN.[<as_ $underlying>]() && val <= $inner::MAX.[<as_ $underlying>]() {
                    break val;
                  }
                };
                Self($inner::try_new(val).unwrap())
              }
            }
          };

          #[cfg(test)]
          quickcheck::quickcheck! {
            fn [< fuzzy_ $inner _varint >](x: [< Fuzzy $inner:camel >]) -> bool {
              let x = x.0;
              let mut buf = [0; $inner::MAX_ENCODED_LEN];
              let len = [< encode_ $inner _varint_to >](x, &mut buf).unwrap();
              let buffer = [< encode_ $inner _varint >](x);
              assert_eq!(buffer.len(), len);
              assert_eq!(buffer.as_slice(), &buf[..len]);

              let (readed, val) = [< decode_ $inner _varint >](&buf).unwrap();
              assert_eq!(readed, len);
              assert_eq!(val, x);

              true
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
            assert_eq!(buffer.as_slice(), &buf[..min_encoded_len]);

            let (readed, val) = [< decode_ $inner _varint >](&buf).unwrap();
            assert_eq!(readed, len);
            assert_eq!(val, min);

            let len = [< encode_ $inner _varint_to >](max, &mut buf).unwrap();
            assert_eq!(len, max_encoded_len);
            let buffer = [< encode_ $inner _varint >](max);
            assert_eq!(buffer.len(), max_encoded_len);
            assert_eq!(buffer.as_slice(), &buf[..max_encoded_len]);

            let (readed, val) = [< decode_ $inner _varint >](&buf).unwrap();
            assert_eq!(readed, len);
            assert_eq!(val, max);
          }
        }
      )*
    )*
  };
}

generate!(
  u8(1..=7),
  u16(9..=15),
  u32(17..=31),
  u64(33..=63),
  u128(65..=127),
);
