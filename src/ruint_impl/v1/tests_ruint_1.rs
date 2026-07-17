use super::*;

use ruint_1::aliases::{
  U0, U1, U16, U32, U64, U128, U256, U320, U384, U448, U512, U768, U1024, U2048, U4096,
};

use quickcheck_macros::quickcheck;

macro_rules! fuzzy {
  ($($ty:ident), +$(,)?) => {
    $(
      paste::paste! {
        #[quickcheck]
        fn [< fuzzy_ $ty:snake >](value: $ty) -> bool {
          let mut buf = [0; <$ty>::MAX_ENCODED_LEN.get()];
          let Ok(encoded_len) = value.encode(&mut buf) else { return false; };
          if encoded_len != value.encoded_len() || !(value.encoded_len() <= <$ty>::MAX_ENCODED_LEN) {
            return false;
          }

          let Some(consumed) = crate::consume_varint_checked(&buf) else {
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
}

fuzzy!(
  U0, U1, U16, U32, U64, U128, U256, U320, U384, U448, U512, U768, U1024, U2048, U4096
);

macro_rules! max_encoded_len {
  ($($ty:ident), +$(,)?) => {
    $(
      paste::paste! {
        #[test]
        fn [< test_ $ty:snake _min_max_encoded_len>]() {
          let max = $ty::MAX;
          let min = $ty::MIN;
          assert_eq!(max.encoded_len(), $ty::MAX_ENCODED_LEN);
          assert_eq!(min.encoded_len(), $ty::MIN_ENCODED_LEN);
        }
      }
    )*
  };
}

max_encoded_len!(
  U0, U1, U16, U32, U64, U128, U256, U320, U384, U448, U512, U768, U1024, U2048, U4096
);

#[test]
fn final_byte_excess_bits_are_rejected() {
  // BITS = 8 is not a multiple of 7: the second byte's payload carries bits
  // above the 8-bit width. Previously these were silently truncated and the
  // decode wrongly returned `Ok((2, 127))`; now it must be rejected.
  assert!(matches!(
    <Uint<8, 1>>::decode(&[0xff, 0x02]),
    Err(DecodeError::Overflow)
  ));

  // Valid maxima still decode: Uint<8> max = 255 = [0xff, 0x01].
  let (read, value) = <Uint<8, 1>>::decode(&[0xff, 0x01]).unwrap();
  assert_eq!(read.get(), 2);
  assert_eq!(value, Uint::<8, 1>::MAX);

  // A wider, non-multiple-of-7 width (U256) round-trips its true maximum.
  let max = U256::MAX;
  let mut buf = [0u8; <U256>::MAX_ENCODED_LEN.get()];
  let written = max.encode(&mut buf).unwrap();
  let (read, value) = U256::decode(&buf).unwrap();
  assert_eq!(read, written);
  assert_eq!(value, max);

  // U256: 256 = 7 * 36 + 4, so the 37th byte holds only 4 payload bits.
  // A terminal byte with a bit above that width (0x10) must be rejected.
  let mut overflow = [0x80u8; 37];
  overflow[36] = 0x10;
  assert!(matches!(
    U256::decode(&overflow),
    Err(DecodeError::Overflow)
  ));
}

#[cfg(feature = "std")]
mod with_std {
  use super::*;

  use std::{vec, vec::Vec};

  // Helper type to generate fixed size arrays
  #[derive(Debug, Clone)]
  struct ByteArray<const N: usize>([u8; N]);

  impl<const N: usize> quickcheck::Arbitrary for ByteArray<N> {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
      let mut arr = [0u8; N];
      for b in arr.iter_mut() {
        *b = u8::arbitrary(g);
      }
      ByteArray(arr)
    }
  }

  // Underflow tests for different sizes
  #[quickcheck]
  fn fuzzy_u256_buffer_underflow(bytes: ByteArray<32>, short_len: usize) -> bool {
    let uint = Uint::<256, 4>::from_be_bytes(bytes.0);
    let short_len = short_len % (Uint::<256, 4>::MAX_ENCODED_LEN.get() - 1);
    if short_len >= uint.encoded_len().get() {
      return true;
    }
    let mut short_buffer = vec![0u8; short_len];
    matches!(
      uint.encode(&mut short_buffer),
      Err(EncodeError::InsufficientSpace { .. })
    )
  }

  #[quickcheck]
  fn fuzzy_u512_buffer_underflow(bytes: ByteArray<64>, short_len: usize) -> bool {
    let uint = Uint::<512, 8>::from_be_bytes(bytes.0);
    let short_len = short_len % (Uint::<512, 8>::MAX_ENCODED_LEN.get() - 1);
    if short_len >= uint.encoded_len().get() {
      return true;
    }
    let mut short_buffer = vec![0u8; short_len];
    matches!(
      uint.encode(&mut short_buffer),
      Err(EncodeError::InsufficientSpace { .. })
    )
  }

  #[quickcheck]
  fn fuzzy_invalid_sequences(bytes: Vec<u8>) -> bool {
    if bytes.is_empty() {
      return matches!(U256::decode(&bytes), Err(DecodeError::InsufficientData(_)));
    }

    // Only test sequences up to max varint length
    if bytes.len() > 10 {
      return true;
    }

    // If all bytes have continuation bit set, should get Underflow
    if bytes.iter().all(|b| b & 0x80 != 0) {
      return matches!(U256::decode(&bytes), Err(DecodeError::InsufficientData(_)));
    }

    // For other cases, we should get either a valid decode or an error
    match U256::decode(&bytes) {
      Ok(_) => true,
      Err(_) => true,
    }
  }
}
