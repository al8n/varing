use super::*;

macro_rules! fuzzy {
  (@varing ($($ty:ty $( => $suffix:ident)? ), +$(,)?)) => {
    paste::paste! {
      $(
        #[quickcheck_macros::quickcheck]
        fn [< fuzzy_ $ty:snake >](value: $ty) -> bool {
          let encoded = [< encode_ $ty:snake $(_$suffix)? >](value);
          if encoded.len() != [< encoded_ $ty:snake $(_$suffix)?_len >] (value) || !(encoded.len() <= <$ty>::MAX_ENCODED_LEN) {
            return false;
          }

          let Ok(consumed) = $crate::consume_varint(&encoded) else {
            return false;
          };
          if consumed != encoded.len() {
            return false;
          }

          if let Ok((bytes_read, decoded)) = [< decode_ $ty:snake $(_$suffix)? >](&encoded) {
            value == decoded && encoded.len() == bytes_read
          } else {
            false
          }
        }
      )*
    }
  };
  (@const_varint_into ($($ty:ident($target:ty) $( => $suffix:ident)? ), +$(,)?)) => {
    paste::paste! {
      $(
        #[quickcheck_macros::quickcheck]
        fn [< fuzzy_ $ty:snake >](value: $ty) -> bool {
          let value = ::core::convert::Into::into(value);
          let encoded = [< encode_ $ty:snake $(_$suffix)? >](value);
          if encoded.len() != [< encoded_ $ty:snake $(_$suffix)?_len >] (value) || !(encoded.len() <= <$target>::MAX_ENCODED_LEN) {
            return false;
          }

          let Ok(consumed) = $crate::consume_varint(&encoded) else {
            return false;
          };
          if consumed != encoded.len() {
            return false;
          }

          if let Ok((bytes_read, decoded)) = [< decode_ $ty:snake $(_$suffix)? >](&encoded) {
            value == decoded && encoded.len() == bytes_read
          } else {
            false
          }
        }
      )*
    }
  };
  (@varing_ref ($($ty:ty$( => $suffix:ident)?), +$(,)?)) => {
    paste::paste! {
      $(
        #[quickcheck_macros::quickcheck]
        fn [< fuzzy_ $ty:snake >](value: $ty) -> bool {
          let encoded = [< encode_ $ty:snake $(_$suffix)? >](&value);
          if encoded.len() != [< encoded_ $ty:snake $(_$suffix)?_len >] (&value) || !(encoded.len() <= <$ty>::MAX_ENCODED_LEN) {
            return false;
          }

          let Ok(consumed) = $crate::consume_varint(&encoded) else {
            return false;
          };
          if consumed != encoded.len() {
            return false;
          }

          if let Ok((bytes_read, decoded)) = [< decode_ $ty:snake $(_$suffix)? >](&encoded) {
            value == decoded && encoded.len() == bytes_read
          } else {
            false
          }
        }
      )*
    }
  };
  (@varint($($ty:ty), +$(,)?)) => {
    $(
      paste::paste! {
        #[quickcheck_macros::quickcheck]
        fn [< fuzzy_ $ty:snake _varint>](value: $ty) -> bool {
          let mut buf = [0; <$ty>::MAX_ENCODED_LEN];
          let Ok(encoded_len) = value.encode(&mut buf) else { return false; };
          if encoded_len != value.encoded_len() || !(value.encoded_len() <= <$ty>::MAX_ENCODED_LEN) {
            return false;
          }

          let Ok(consumed) = $crate::consume_varint(&buf) else {
            return false;
          };
          if consumed != encoded_len {
            return false;
          }

          if let Ok((bytes_read, decoded)) = <$ty>::decode(&buf) {
            value == decoded && encoded_len == bytes_read
          } else {
            false
          }
        }
      }
    )*
  };
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
            value == decoded && encoded_len == bytes_read
          } else {
            false
          }
        }
      }
    )*
  };
  (@sequence ($($ty:ty), +$(,)?)) => {
    $(
      paste::paste! {
        #[quickcheck_macros::quickcheck]
        fn [< fuzzy_ $ty:snake _sequence>](value: std::vec::Vec<$ty>) -> bool {
          let encoded_len = [< encoded_ $ty _sequence_len>](&value);
          let mut buf = std::vec![0; encoded_len];
          let Ok(written) = [< encode_ $ty _sequence_to>](&value, &mut buf) else { return false; };
          if encoded_len != written {
            return false;
          }

          let (readed, decoded) = <$ty>::decode_sequence::<std::vec::Vec<_>>(&buf).unwrap();
          if encoded_len != readed {
            return false;
          }

          if decoded != value {
            return false;
          }

          true
        }
      }
    )*
  };
}

fuzzy!(@varing(u8 => varint, u16 => varint, u32 => varint, u64 => varint, u128 => varint, i8 => varint, i16 => varint, i32 => varint, i64 => varint, i128 => varint));
fuzzy!(@varint(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, bool));

#[cfg(feature = "std")]
mod with_std {
  use super::*;

  extern crate std;

  use std::{vec, vec::Vec};

  fuzzy!(@sequence (u8, u16, u32, u64, u128, i8, i16, i32, i64, i128));

  #[quickcheck_macros::quickcheck]
  fn fuzzy_buffer_underflow(value: u64, short_len: usize) -> bool {
    let short_len = short_len % 9; // Keep length under max varint size
    if short_len >= value.encoded_len() {
      return true; // Skip test if buffer is actually large enough
    }
    let mut short_buffer = vec![0u8; short_len];
    matches!(
      value.encode(&mut short_buffer),
      Err(EncodeError::Underflow { .. })
    )
  }

  #[quickcheck_macros::quickcheck]
  fn fuzzy_invalid_sequences(bytes: Vec<u8>) -> bool {
    if bytes.is_empty() {
      return matches!(decode_u64_varint(&bytes), Err(DecodeError::Underflow));
    }

    // Only test sequences up to max varint length
    if bytes.len() > 10 {
      return true;
    }

    // If all bytes have continuation bit set, should get Underflow
    if bytes.iter().all(|b| b & 0x80 != 0) {
      return matches!(decode_u64_varint(&bytes), Err(DecodeError::Underflow));
    }

    // For other cases, we should get either a valid decode or an error
    match decode_u64_varint(&bytes) {
      Ok(_) => true,
      Err(_) => true,
    }
  }
}
