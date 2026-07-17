use super::*;

use quickcheck_macros::quickcheck;

#[quickcheck]
fn encode_decode_duration(value: Duration) -> bool {
  let encoded = encode_duration(&value);
  if encoded.len() != encoded_duration_len(&value).get()
    || (encoded.len() > <Duration>::MAX_ENCODED_LEN.get())
  {
    return false;
  }

  if let Ok((bytes_read, decoded)) = decode_duration(&encoded) {
    value == decoded && encoded.len() == bytes_read.get()
  } else {
    false
  }
}

#[quickcheck]
fn encode_decode_duration_varint(value: Duration) -> bool {
  let mut buf = [0; <Duration>::MAX_ENCODED_LEN.get()];
  let Ok(encoded_len) = value.encode(&mut buf) else {
    return false;
  };
  if encoded_len != value.encoded_len() || (value.encoded_len() > <Duration>::MAX_ENCODED_LEN) {
    return false;
  }

  if let Ok((bytes_read, decoded)) = <Duration>::decode(&buf) {
    value == decoded && encoded_len == bytes_read
  } else {
    false
  }
}

#[cfg(feature = "std")]
#[test]
fn decode_rejects_nanos_overflow_without_panic() {
  // Malformed wire: secs = u64::MAX, nanos = 1_000_000_000 (>= 1e9), which
  // makes `Duration::new`'s nanos->secs carry overflow `u64` and panic.
  let value = ((u64::MAX as u128) << 32) | 1_000_000_000;
  let encoded = encode_u128_varint(value);
  let result = std::panic::catch_unwind(move || decode_duration(&encoded));
  assert!(matches!(result, Ok(Err(_))));

  // A normal duration still round-trips.
  let normal = Duration::new(42, 123_456_789);
  let enc = encode_duration(&normal);
  let (read, decoded) = decode_duration(&enc).unwrap();
  assert_eq!(decoded, normal);
  assert_eq!(read.get(), enc.len());
}

#[test]
fn decode_rejects_out_of_range_seconds() {
  // Malformed wire: any value with a bit at or above position 96 exceeds the
  // 96-bit duration layout (32-bit nanos + 64-bit secs). The decoder must reject
  // it instead of silently discarding the high bits and returning `Duration::ZERO`.
  let encoded = encode_u128_varint(1u128 << 96);
  assert!(decode_duration(&encoded).is_err());

  // A normal duration still round-trips.
  let normal = Duration::new(42, 123_456_789);
  let enc = encode_duration(&normal);
  let (read, decoded) = decode_duration(&enc).unwrap();
  assert_eq!(decoded, normal);
  assert_eq!(read.get(), enc.len());
}
