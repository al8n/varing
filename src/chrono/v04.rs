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
mod tests;
