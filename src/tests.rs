extern crate std;

use super::*;

fn check(value: u64, encoded: &[u8]) {
  let a = encode_u64_varint(value);
  assert_eq!(a.as_ref(), encoded);
  assert_eq!(a.len(), encoded.len());
  assert_eq!(a.len(), encoded_u64_varint_len(value));

  let (read, decoded) = decode_u64_varint(&a).unwrap();
  assert_eq!(decoded, value);
  assert_eq!(read, encoded.len());
  assert_eq!(a.len(), encoded_u64_varint_len(value));
}

#[test]
fn roundtrip_u64() {
  check(2u64.pow(0) - 1, &[0x00]);
  check(2u64.pow(0), &[0x01]);

  check(2u64.pow(7) - 1, &[0x7F]);
  check(2u64.pow(7), &[0x80, 0x01]);
  check(300u64, &[0xAC, 0x02]);

  check(2u64.pow(14) - 1, &[0xFF, 0x7F]);
  check(2u64.pow(14), &[0x80, 0x80, 0x01]);

  check(2u64.pow(21) - 1, &[0xFF, 0xFF, 0x7F]);
  check(2u64.pow(21), &[0x80, 0x80, 0x80, 0x01]);

  check(2u64.pow(28) - 1, &[0xFF, 0xFF, 0xFF, 0x7F]);
  check(2u64.pow(28), &[0x80, 0x80, 0x80, 0x80, 0x01]);

  check(2u64.pow(35) - 1, &[0xFF, 0xFF, 0xFF, 0xFF, 0x7F]);
  check(2u64.pow(35), &[0x80, 0x80, 0x80, 0x80, 0x80, 0x01]);

  check(2u64.pow(42) - 1, &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F]);
  check(2u64.pow(42), &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01]);

  check(
    2u64.pow(49) - 1,
    &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F],
  );
  check(
    2u64.pow(49),
    &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
  );

  check(
    2u64.pow(56) - 1,
    &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F],
  );
  check(
    2u64.pow(56),
    &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
  );

  check(
    2u64.pow(63) - 1,
    &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F],
  );
  check(
    2u64.pow(63),
    &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
  );

  check(
    u64::MAX,
    &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01],
  );
}

#[test]
fn test_large_number_encode_decode() {
  let original = 30000u64;
  let encoded = encode_u64_varint(original);
  let (bytes_read, decoded) = decode_u64_varint(&encoded).unwrap();
  assert_eq!(original, decoded);
  assert_eq!(bytes_read, encoded.len());
}

#[test]
fn test_decode_overflow_error() {
  let buffer = [0x80u8; 11]; // More than 10 bytes
  match decode_u64_varint(&buffer) {
    Err(DecodeError::Overflow) => (),
    _ => panic!("Expected Overflow error"),
  }

  let buffer = [0x80u8; 6]; // More than 5 bytes
  match decode_u32_varint(&buffer) {
    Err(DecodeError::Overflow) => (),
    _ => panic!("Expected Overflow error"),
  }

  let buffer = [0x80u8; 4]; // More than 3 bytes
  match decode_u16_varint(&buffer) {
    Err(DecodeError::Overflow) => (),
    _ => panic!("Expected Overflow error"),
  }
}

// Helper function for zig-zag encoding and decoding
fn test_zigzag_encode_decode<T>(value: T)
where
  T: Copy
    + PartialEq
    + core::fmt::Debug
    + core::ops::Shl<Output = T>
    + core::ops::Shr<Output = T>
    + Into<i64>
    + core::convert::TryInto<usize>
    + core::convert::TryFrom<usize>,
{
  let encoded = encode_i64_varint(value.into());
  let bytes_written = encoded.len();

  // Decode
  let decode_result = decode_i64_varint(&encoded);
  assert!(decode_result.is_ok(), "Decoding failed");
  let (decoded_bytes, decoded_value) = decode_result.unwrap();

  assert_eq!(
    decoded_bytes, bytes_written,
    "Incorrect number of bytes decoded"
  );
  assert_eq!(
    decoded_value,
    value.into(),
    "Decoded value does not match original"
  );
}

#[test]
fn test_zigzag_encode_decode_i8() {
  let values = [-1, 0, 1, -100, 100, i8::MIN, i8::MAX];
  for &value in &values {
    test_zigzag_encode_decode(value);
  }
}

#[test]
fn test_zigzag_encode_decode_i16() {
  let values = [-1, 0, 1, -100, 100, i16::MIN, i16::MAX];
  for &value in &values {
    test_zigzag_encode_decode(value);
  }
}

#[test]
fn test_zigzag_encode_decode_i32() {
  let values = [-1, 0, 1, -10000, 10000, i32::MIN, i32::MAX];
  for &value in &values {
    test_zigzag_encode_decode(value);
  }
}

#[test]
fn test_zigzag_encode_decode_i64() {
  let values = [-1, 0, 1, -1000000000, 1000000000, i64::MIN, i64::MAX];
  for &value in &values {
    test_zigzag_encode_decode(value);
  }
}

#[test]
fn test_encode_error_update() {
  let ent = EncodeError::underflow(1, 0).update(4, 0);
  assert!(matches!(
    ent,
    EncodeError::Underflow {
      required: 4,
      remaining: 0
    }
  ));

  let ent = EncodeError::custom("test").update(4, 0);
  assert!(matches!(ent, EncodeError::Custom(_)));
}
