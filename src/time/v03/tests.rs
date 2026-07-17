use super::*;

type Utc = UtcDateTime;
type Datetime = PrimitiveDateTime;

#[derive(Debug, Clone, Copy)]
struct DurationWrapper(Duration);

impl quickcheck::Arbitrary for DurationWrapper {
  fn arbitrary(g: &mut quickcheck::Gen) -> Self {
    let d: core::time::Duration = quickcheck::Arbitrary::arbitrary(g);

    Self(Duration::new(d.as_secs() as i64, d.subsec_nanos() as i32))
  }
}

#[quickcheck_macros::quickcheck]
fn fuzzy_duration(value: DurationWrapper) -> bool {
  let value = value.0;
  let encoded = encode_duration(&value);
  if encoded.len() != encoded_duration_len(&value).get()
    || (encoded.len() > <Duration>::MAX_ENCODED_LEN.get())
  {
    return false;
  }

  let Some(consumed) = crate::consume_varint_checked(&encoded) else {
    return false;
  };
  if consumed.get() != encoded.len() {
    return false;
  }

  if let Ok((bytes_read, decoded)) = decode_duration(&encoded) {
    value == decoded && encoded.len() == bytes_read.get()
  } else {
    false
  }
}

#[quickcheck_macros::quickcheck]
fn fuzzy_duration_varint(value: DurationWrapper) -> bool {
  let value = value.0;
  let mut buf = [0; <Duration>::MAX_ENCODED_LEN.get()];
  let Ok(encoded_len) = value.encode(&mut buf) else {
    return false;
  };
  if encoded_len != value.encoded_len() || (value.encoded_len() > <Duration>::MAX_ENCODED_LEN) {
    return false;
  }

  let Some(consumed) = crate::consume_varint_checked(&buf) else {
    return false;
  };
  if consumed != encoded_len {
    return false;
  }

  if let Ok((bytes_read, decoded)) = <Duration>::decode(&buf) {
    value == decoded && encoded_len == bytes_read
  } else {
    false
  }
}

fuzzy!(@varing_ref(Date, Datetime, Time, Utc));
fuzzy!(@varint(Date, PrimitiveDateTime, Time, UtcDateTime));

#[test]
fn decode_rejects_nanos_overflow_without_panic() {
  // Malformed wire: secs = i64::MAX, nanos = 1_000_000_000 (>= 1e9), which
  // makes `Duration::new`'s nanos->secs carry overflow `i64` and panic.
  let merged = time_utils::secs_and_subsec_nanos_to_merged(i64::MAX, 1_000_000_000);
  let encoded = crate::encode_u128_varint(merged);
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
fn decode_rejects_out_of_range_bits_without_panic() {
  // A merged value with a bit at or above position 96 exceeds the 96-bit
  // duration layout. The shared decoder must reject it (returning `Err`, never
  // panicking) rather than silently discarding the high bits.
  let encoded = crate::encode_u128_varint(1u128 << 96);
  assert!(decode_duration(&encoded).is_err());

  // A valid duration still round-trips.
  let normal = Duration::new(42, 123_456_789);
  let enc = encode_duration(&normal);
  let (read, decoded) = decode_duration(&enc).unwrap();
  assert_eq!(decoded, normal);
  assert_eq!(read.get(), enc.len());
}

#[test]
fn decode_rejects_non_canonical_time() {
  // Bit 48 lies above the 48-bit time layout; `merged_to_time` drops it, so
  // without the canonicity guard this aliases midnight (all-zero components).
  let encoded = crate::encode_u64_varint(1u64 << 48);
  assert!(decode_time(&encoded).is_err());

  // A valid time still round-trips.
  let value = Time::from_hms_nano(12, 34, 56, 789_000_000).unwrap();
  let enc = encode_time(&value);
  let (read, decoded) = decode_time(&enc).unwrap();
  assert_eq!(decoded, value);
  assert_eq!(read.get(), enc.len());
}

#[test]
fn decode_rejects_non_canonical_datetime() {
  // A bit above the datetime layout is dropped when the year is truncated to
  // `i32`, so without the guard this aliases the valid datetime below.
  let valid = time_utils::date_time_to_merged(2024, 6, 15, 12, 34, 56, 789_000_000);
  let bad = valid | (1i128 << 100);
  let encoded = crate::encode_i128_varint(bad);
  assert!(decode_datetime(&encoded).is_err());

  // A valid datetime still round-trips.
  let date = Date::from_calendar_date(2024, Month::June, 15).unwrap();
  let time = Time::from_hms_nano(12, 34, 56, 789_000_000).unwrap();
  let value = PrimitiveDateTime::new(date, time);
  let enc = encode_datetime(&value);
  let (read, decoded) = decode_datetime(&enc).unwrap();
  assert_eq!(decoded, value);
  assert_eq!(read.get(), enc.len());
}

#[test]
fn decode_rejects_sign_mismatch_duration() {
  // `secs = 1, nanos = -1` passes the magnitude guard but normalizes to
  // 0.999999999s, aliasing the canonical `secs = 0, nanos = 999_999_999`.
  let merged = time_utils::secs_and_subsec_nanos_to_merged(1, -1);
  assert!(decode_duration(&crate::encode_u128_varint(merged)).is_err());

  // `secs = -1, nanos = 1` is the mirror-image alias of -0.999999999s.
  let merged = time_utils::secs_and_subsec_nanos_to_merged(-1, 1);
  assert!(decode_duration(&crate::encode_u128_varint(merged)).is_err());

  // Zero-second, zero-nanos, positive-fractional and fully-negative durations
  // all keep sign agreement and must still round-trip.
  for value in [
    Duration::new(0, -5),
    Duration::new(5, 0),
    Duration::new(3, 500_000_000),
    Duration::new(-5, -5),
  ] {
    let enc = encode_duration(&value);
    let (read, decoded) = decode_duration(&enc).unwrap();
    assert_eq!(decoded, value);
    assert_eq!(read.get(), enc.len());
  }
}

#[test]
fn decode_date_round_trip() {
  // `merged_to_date`/`date_to_merged` partition all 32 bits bijectively, so
  // there is no non-canonical date encoding; a valid date must round-trip.
  let value = Date::from_calendar_date(2024, Month::June, 15).unwrap();
  let enc = encode_date(&value);
  let (read, decoded) = decode_date(&enc).unwrap();
  assert_eq!(decoded, value);
  assert_eq!(read.get(), enc.len());
}
