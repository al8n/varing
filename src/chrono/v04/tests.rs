use super::*;

type TimeTime = time_0_3::Time;
type TimeDate = time_0_3::Date;
type TimeDateTime = time_0_3::PrimitiveDateTime;
type TimeUtc = time_0_3::UtcDateTime;

trait IntoChrono {
  type Target;
  fn into_chrono(self) -> Option<Self::Target>;
}

impl IntoChrono for TimeTime {
  type Target = NaiveTime;
  fn into_chrono(self) -> Option<Self::Target> {
    NaiveTime::from_hms_nano_opt(
      self.hour() as u32,
      self.minute() as u32,
      self.second() as u32,
      self.nanosecond(),
    )
  }
}

impl IntoChrono for TimeDate {
  type Target = NaiveDate;
  fn into_chrono(self) -> Option<Self::Target> {
    NaiveDate::from_ymd_opt(self.year(), self.month() as u32, self.day() as u32)
  }
}

impl IntoChrono for TimeDateTime {
  type Target = NaiveDateTime;
  fn into_chrono(self) -> Option<Self::Target> {
    Some(NaiveDateTime::new(
      self.date().into_chrono()?,
      self.time().into_chrono()?,
    ))
  }
}

impl IntoChrono for TimeUtc {
  type Target = DateTime<Utc>;
  fn into_chrono(self) -> Option<Self::Target> {
    Some(DateTime::<Utc>::from_naive_utc_and_offset(
      TimeDateTime::new(self.date(), self.time()).into_chrono()?,
      Utc,
    ))
  }
}

macro_rules! fuzzy_chrono_types {
  ($($ty:ident), +$(,)?) => {
    paste::paste! {
      $(
        #[quickcheck_macros::quickcheck]
        fn [< fuzzy_ $ty:snake >](value: [< Time $ty >]) -> bool {
          let value = match value.into_chrono() {
            Some(value) => value,
            None => return true,
          };

          let mut buf = [0; <<[< Time $ty >] as IntoChrono>::Target>::MAX_ENCODED_LEN.get()];
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

          if let Ok((bytes_read, decoded)) = <<[< Time $ty >] as IntoChrono>::Target>::decode(&buf) {
            value == decoded && encoded_len == bytes_read
          } else {
            false
          }
        }
      )*
    }
  };
}

fuzzy_chrono_types!(Time, Date, DateTime, Utc);

#[derive(Debug, Clone, Copy)]
struct DurationWrapper(Duration);

impl quickcheck::Arbitrary for DurationWrapper {
  fn arbitrary(g: &mut quickcheck::Gen) -> Self {
    let d: core::time::Duration = quickcheck::Arbitrary::arbitrary(g);

    Self(Duration::new(d.as_secs() as i64, d.subsec_nanos()).unwrap())
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

#[cfg(feature = "std")]
#[test]
fn decode_rejects_out_of_range_secs_without_panic() {
  // Malformed wire: secs = i64::MAX, outside chrono `TimeDelta`'s second
  // bound, which makes `Duration::seconds` panic.
  let merged = time_utils::secs_and_subsec_nanos_to_merged(i64::MAX, 0);
  let encoded = crate::encode_u128_varint(merged);
  let result = std::panic::catch_unwind(move || decode_duration(&encoded));
  assert!(matches!(result, Ok(Err(_))));

  // A normal duration still round-trips.
  let normal = Duration::new(42, 123_456_789).unwrap();
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
  let normal = Duration::new(42, 123_456_789).unwrap();
  let enc = encode_duration(&normal);
  let (read, decoded) = decode_duration(&enc).unwrap();
  assert_eq!(decoded, normal);
  assert_eq!(read.get(), enc.len());
}

#[test]
fn decode_duration_negative_round_trip() {
  // -5.000000123s: num_seconds() == -5, subsec_nanos() == -123. Proves the
  // new out-of-range-nanos guard does not reject valid negative durations.
  let value = Duration::seconds(-5) + Duration::nanoseconds(-123);
  let enc = encode_duration(&value);
  let (read, decoded) = decode_duration(&enc).unwrap();
  assert_eq!(decoded, value);
  assert_eq!(read.get(), enc.len());
}

#[test]
fn decode_duration_positive_fractional_round_trip() {
  // 3.5s: num_seconds() == 3, subsec_nanos() == 500_000_000.
  let value = Duration::seconds(3) + Duration::nanoseconds(500_000_000);
  let enc = encode_duration(&value);
  let (read, decoded) = decode_duration(&enc).unwrap();
  assert_eq!(decoded, value);
  assert_eq!(read.get(), enc.len());
}

#[test]
fn decode_rejects_out_of_range_nanos() {
  // Malformed wire: nanos == 1_000_000_000 (>= 1e9) is non-canonical. A
  // well-formed encoder only ever emits `TimeDelta::subsec_nanos()`, whose
  // magnitude is always < 1_000_000_000, so this must be rejected.
  let merged = time_utils::secs_and_subsec_nanos_to_merged(0, 1_000_000_000);
  let encoded = crate::encode_u128_varint(merged);
  assert!(decode_duration(&encoded).is_err());

  // Malformed wire: nanos == -1_000_000_000 (<= -1e9) is likewise
  // non-canonical and must also be rejected.
  let merged = time_utils::secs_and_subsec_nanos_to_merged(0, -1_000_000_000);
  let encoded = crate::encode_u128_varint(merged);
  assert!(decode_duration(&encoded).is_err());
}

#[test]
fn decode_rejects_non_canonical_time() {
  // Bit 48 lies above the 48-bit time layout and is dropped on decode, so
  // without the canonicity guard it aliases midnight.
  let encoded = crate::encode_u64_varint(1u64 << 48);
  assert!(NaiveTime::decode(&encoded).is_err());

  // A valid time still round-trips.
  let value = NaiveTime::from_hms_nano_opt(12, 34, 56, 789_000_000).unwrap();
  let mut buf = [0u8; <NaiveTime>::MAX_ENCODED_LEN.get()];
  let n = value.encode(&mut buf).unwrap();
  let (read, decoded) = NaiveTime::decode(&buf).unwrap();
  assert_eq!(decoded, value);
  assert_eq!(read, n);
}

#[test]
fn decode_rejects_non_canonical_datetime() {
  // A bit above the datetime layout is dropped when the year is truncated to
  // `i32`, so without the guard it aliases the valid datetime below.
  let valid = time_utils::date_time_to_merged(2024, 6, 15, 12, 34, 56, 789_000_000);
  let bad = valid | (1i128 << 100);
  let encoded = crate::encode_i128_varint(bad);
  assert!(NaiveDateTime::decode(&encoded).is_err());

  // A valid datetime still round-trips.
  let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
  let time = NaiveTime::from_hms_nano_opt(12, 34, 56, 789_000_000).unwrap();
  let value = NaiveDateTime::new(date, time);
  let mut buf = [0u8; <NaiveDateTime>::MAX_ENCODED_LEN.get()];
  let n = value.encode(&mut buf).unwrap();
  let (read, decoded) = NaiveDateTime::decode(&buf).unwrap();
  assert_eq!(decoded, value);
  assert_eq!(read, n);
}

#[test]
fn decode_rejects_sign_mismatch_duration() {
  // `secs = 1, nanos = -1` passes the magnitude guard but folds to
  // 0.999999999s, aliasing the canonical `secs = 0, nanos = 999_999_999`.
  let merged = time_utils::secs_and_subsec_nanos_to_merged(1, -1);
  assert!(decode_duration(&crate::encode_u128_varint(merged)).is_err());

  // `secs = -1, nanos = 1` is the mirror-image alias of -0.999999999s.
  let merged = time_utils::secs_and_subsec_nanos_to_merged(-1, 1);
  assert!(decode_duration(&crate::encode_u128_varint(merged)).is_err());

  // Sign-agreeing durations (zero-second, zero-nanos, positive-fractional and
  // fully-negative) must still round-trip.
  for value in [
    Duration::nanoseconds(-5),
    Duration::seconds(5),
    Duration::seconds(3) + Duration::nanoseconds(500_000_000),
    Duration::seconds(-5) + Duration::nanoseconds(-5),
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
  let value = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
  let mut buf = [0u8; <NaiveDate>::MAX_ENCODED_LEN.get()];
  let n = value.encode(&mut buf).unwrap();
  let (read, decoded) = NaiveDate::decode(&buf).unwrap();
  assert_eq!(decoded, value);
  assert_eq!(read, n);
}
