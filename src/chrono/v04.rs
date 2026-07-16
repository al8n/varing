use crate::{
  ConstDecodeError, ConstEncodeError, DecodeError, EncodeError, NON_ZERO_USIZE_ONE, Varint,
  time_utils::{self, DurationBuffer},
};

use chrono_0_4::{
  DateTime, Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, Timelike, Utc,
};

use core::num::NonZeroUsize;

pub use time_utils::{DateBuffer, DateTimeBuffer, TimeBuffer};

/// Returns the encoded length of the value in LEB128 variable length format.
/// The returned value will be in range [`Duration::ENCODED_LEN_RANGE`].
#[inline]
pub const fn encoded_duration_len(duration: &Duration) -> NonZeroUsize {
  time_utils::encoded_secs_and_subsec_nanos_len(duration.num_seconds(), duration.subsec_nanos())
}

/// Encodes a `Duration` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub const fn encode_duration(duration: &Duration) -> DurationBuffer {
  time_utils::encode_secs_and_subsec_nanos(duration.num_seconds(), duration.subsec_nanos())
}

/// Encodes a `Duration` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub const fn encode_duration_to(
  duration: &Duration,
  buf: &mut [u8],
) -> Result<NonZeroUsize, ConstEncodeError> {
  time_utils::encode_secs_and_subsec_nanos_to(duration.num_seconds(), duration.subsec_nanos(), buf)
}

/// Decodes a `Duration` in LEB128 encoded format from the buffer.
///
/// Returns the bytes readed and the decoded value if successful.
#[inline]
pub const fn decode_duration(buf: &[u8]) -> Result<(NonZeroUsize, Duration), ConstDecodeError> {
  match time_utils::decode_secs_and_subsec_nanos(buf) {
    Ok((bytes_read, secs, nanos)) => {
      // `decode_secs_and_subsec_nanos` returns a raw zigzag-decoded `nanos`
      // that is not bounded to the canonical subsecond range. A well-formed
      // chrono `TimeDelta` only ever emits `subsec_nanos()`, which is always
      // in the open range (-1_000_000_000, 1_000_000_000) with sign matching
      // the duration; reject anything else so malformed input cannot fold
      // through `Duration::nanoseconds(nanos as i64).checked_add(...)` below
      // into a structurally valid but non-canonical `Duration`.
      if nanos <= -1_000_000_000 || nanos >= 1_000_000_000 {
        return Err(ConstDecodeError::other("nanos out of range"));
      }

      // Canonicity: a well-formed chrono `TimeDelta` keeps `num_seconds()` and
      // `subsec_nanos()` in sign agreement, so mixed signs (e.g.
      // `secs = 1, nanos = -1`) never come from the encoder. Left unchecked they
      // fold through the `checked_add` below into a valid-but-different value
      // (0.999999999s), aliasing a canonical encoding, so reject them.
      if (secs > 0 && nanos < 0) || (secs < 0 && nanos > 0) {
        return Err(ConstDecodeError::other("non-canonical duration"));
      }

      // `Duration::seconds` (chrono `TimeDelta::seconds`) panics when `secs` is
      // outside TimeDelta's representable range, so use the non-panicking
      // `try_seconds`. `nanoseconds` is infallible and `checked_add` guards the
      // final combination, so malformed input yields an error, never a panic.
      let base = match Duration::try_seconds(secs) {
        Some(base) => base,
        None => {
          return Err(ConstDecodeError::other(
            "duration is out of bounds, or nanos ≥ 1,000,000,000",
          ));
        }
      };
      match base.checked_add(&Duration::nanoseconds(nanos as i64)) {
        Some(duration) => Ok((bytes_read, duration)),
        None => Err(ConstDecodeError::other(
          "duration is out of bounds, or nanos ≥ 1,000,000,000",
        )),
      }
    }
    Err(e) => Err(e),
  }
}

impl Varint for Duration {
  const MIN_ENCODED_LEN: NonZeroUsize = i128::MIN_ENCODED_LEN;
  const MAX_ENCODED_LEN: NonZeroUsize = i128::MAX_ENCODED_LEN;

  #[inline]
  fn encoded_len(&self) -> NonZeroUsize {
    encoded_duration_len(self)
  }

  #[inline]
  fn encode(&self, buf: &mut [u8]) -> Result<NonZeroUsize, EncodeError> {
    encode_duration_to(self, buf).map_err(Into::into)
  }

  #[inline]
  fn decode(buf: &[u8]) -> Result<(NonZeroUsize, Self), DecodeError>
  where
    Self: Sized,
  {
    decode_duration(buf).map_err(Into::into)
  }
}

impl Varint for NaiveDate {
  const MIN_ENCODED_LEN: NonZeroUsize = NON_ZERO_USIZE_ONE;
  const MAX_ENCODED_LEN: NonZeroUsize = DateBuffer::CAPACITY;

  #[inline]
  fn encoded_len(&self) -> NonZeroUsize {
    time_utils::encoded_date_len(self.year(), self.month() as u8, self.day() as u8)
  }

  #[inline]
  fn encode(&self, buf: &mut [u8]) -> Result<NonZeroUsize, EncodeError> {
    time_utils::encode_date_to(self.year(), self.month() as u8, self.day() as u8, buf)
      .map_err(Into::into)
  }

  #[inline]
  fn decode(buf: &[u8]) -> Result<(NonZeroUsize, Self), DecodeError>
  where
    Self: Sized,
  {
    time_utils::decode_date(buf)
      .and_then(|(read, year, month, day)| {
        NaiveDate::from_ymd_opt(year, month as u32, day as u32)
          .map(|date| (read, date))
          .ok_or_else(|| ConstDecodeError::other("invalid date"))
      })
      .map_err(Into::into)
  }
}

impl Varint for NaiveTime {
  const MIN_ENCODED_LEN: NonZeroUsize = NON_ZERO_USIZE_ONE;
  const MAX_ENCODED_LEN: NonZeroUsize = TimeBuffer::CAPACITY;

  #[inline]
  fn encoded_len(&self) -> NonZeroUsize {
    time_utils::encoded_time_len(
      self.nanosecond(),
      self.second() as u8,
      self.minute() as u8,
      self.hour() as u8,
    )
  }

  #[inline]
  fn encode(&self, buf: &mut [u8]) -> Result<NonZeroUsize, EncodeError> {
    time_utils::encode_time_to(
      self.nanosecond(),
      self.second() as u8,
      self.minute() as u8,
      self.hour() as u8,
      buf,
    )
    .map_err(Into::into)
  }

  #[inline]
  fn decode(buf: &[u8]) -> Result<(NonZeroUsize, Self), DecodeError>
  where
    Self: Sized,
  {
    time_utils::decode_time(buf).map_err(Into::into).and_then(
      |(read, nano, second, minute, hour)| {
        // Construct NaiveTime from components
        NaiveTime::from_hms_nano_opt(hour as u32, minute as u32, second as u32, nano)
          .ok_or_else(|| DecodeError::other("invalid hour, minute, second and/or nanosecond"))
          .map(|time| (read, time))
      },
    )
  }
}

impl Varint for NaiveDateTime {
  const MIN_ENCODED_LEN: NonZeroUsize = NON_ZERO_USIZE_ONE;
  const MAX_ENCODED_LEN: NonZeroUsize = DateTimeBuffer::CAPACITY;

  #[inline]
  fn encoded_len(&self) -> NonZeroUsize {
    time_utils::encoded_datetime_len(
      self.year(),
      self.month() as u8,
      self.day() as u8,
      self.hour() as u8,
      self.minute() as u8,
      self.second() as u8,
      self.nanosecond(),
    )
  }

  #[inline]
  fn encode(&self, buf: &mut [u8]) -> Result<NonZeroUsize, EncodeError> {
    time_utils::encode_datetime_to(
      self.year(),
      self.month() as u8,
      self.day() as u8,
      self.hour() as u8,
      self.minute() as u8,
      self.second() as u8,
      self.nanosecond(),
      buf,
    )
    .map_err(Into::into)
  }

  #[inline]
  fn decode(buf: &[u8]) -> Result<(NonZeroUsize, Self), DecodeError>
  where
    Self: Sized,
  {
    time_utils::decode_datetime(buf)
      .map_err(Into::into)
      .and_then(|(read, year, month, day, hour, minute, second, nano)| {
        // Create date and time
        let date = NaiveDate::from_ymd_opt(year, month as u32, day as u32)
          .ok_or(ConstDecodeError::other("invalid date"))?;

        let time = NaiveTime::from_hms_nano_opt(hour as u32, minute as u32, second as u32, nano)
          .ok_or(ConstDecodeError::other(
            "invalid hour, minute, second and/or nanosecond",
          ))?;

        // Combine into NaiveDateTime
        Ok((read, NaiveDateTime::new(date, time)))
      })
  }
}

impl Varint for DateTime<Utc> {
  const MIN_ENCODED_LEN: NonZeroUsize = i128::MIN_ENCODED_LEN;

  const MAX_ENCODED_LEN: NonZeroUsize = i128::MAX_ENCODED_LEN;

  fn encoded_len(&self) -> NonZeroUsize {
    self.naive_utc().encoded_len()
  }

  fn encode(&self, buf: &mut [u8]) -> Result<NonZeroUsize, EncodeError> {
    self.naive_utc().encode(buf)
  }

  fn decode(buf: &[u8]) -> Result<(NonZeroUsize, Self), DecodeError>
  where
    Self: Sized,
  {
    let (read, naive_utc) = NaiveDateTime::decode(buf)?;
    Ok((read, DateTime::from_naive_utc_and_offset(naive_utc, Utc)))
  }
}

#[cfg(test)]
mod tests {
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
}
