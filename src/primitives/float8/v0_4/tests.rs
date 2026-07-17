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
