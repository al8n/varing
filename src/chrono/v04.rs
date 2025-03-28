use crate::{
  time_utils, DecodeError, EncodeError, I128VarintBuffer, U128VarintBuffer, Varint
};

use chrono_0_4::{
  DateTime, Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, Timelike, Utc,
};

pub use time_utils::{DateBuffer, DateTimeBuffer, TimeBuffer};

/// A buffer for storing LEB128 encoded [`Duration`] value.
pub type ChronoDurationBuffer = I128VarintBuffer;

/// A buffer for storing LEB128 encoded [`DataTime<Utc>`](Utc) value.
pub type ChronoUtcBuffer = I128VarintBuffer;

/// Returns the encoded length of the value in LEB128 variable length format.
/// The returned value will be in range [`Duration::ENCODED_LEN_RANGE`].
#[inline]
pub const fn encoded_duration_len(duration: &Duration) -> usize {
  time_utils::encoded_secs_and_subsec_nanos_len(duration.num_seconds(), duration.subsec_nanos())
}

/// Encodes a `Duration` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub const fn encode_duration(duration: &Duration) -> U128VarintBuffer {
  time_utils::encode_secs_and_subsec_nanos(duration.num_seconds(), duration.subsec_nanos())
}

/// Encodes a `Duration` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub const fn encode_duration_to(duration: &Duration, buf: &mut [u8]) -> Result<usize, EncodeError> {
  time_utils::encode_secs_and_subsec_nanos_to(duration.num_seconds(), duration.subsec_nanos(), buf)
}

/// Decodes a `Duration` in LEB128 encoded format from the buffer.
///
/// Returns the bytes readed and the decoded value if successful.
#[inline]
pub const fn decode_duration(buf: &[u8]) -> Result<(usize, Duration), DecodeError> {
  match time_utils::decode_secs_and_subsec_nanos(buf) {
    Ok((bytes_read, secs, nanos)) => {
      match Duration::seconds(secs).checked_add(&Duration::nanoseconds(nanos as i64)) {
        Some(duration) => Ok((bytes_read, duration)),
        None => Err(DecodeError::custom(
          "duration is out of bounds, or nanos ≥ 1,000,000,000",
        )),
      }
    }
    Err(e) => Err(e),
  }
}

impl Varint for Duration {
  const MIN_ENCODED_LEN: usize = i128::MIN_ENCODED_LEN;
  const MAX_ENCODED_LEN: usize = i128::MAX_ENCODED_LEN;

  #[inline]
  fn encoded_len(&self) -> usize {
    encoded_duration_len(self)
  }

  #[inline]
  fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
    encode_duration_to(self, buf)
  }

  #[inline]
  fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError>
  where
    Self: Sized,
  {
    decode_duration(buf)
  }
}

impl Varint for NaiveDate {
  const MIN_ENCODED_LEN: usize = 1;
  const MAX_ENCODED_LEN: usize = DateBuffer::CAPACITY;

  #[inline]
  fn encoded_len(&self) -> usize {
    time_utils::encoded_date_len(self.year(), self.month() as u8, self.day() as u8)
  }

  #[inline]
  fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
    time_utils::encode_date_to(self.year(), self.month() as u8, self.day() as u8, buf)
  }

  #[inline]
  fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError>
  where
    Self: Sized,
  {
    time_utils::decode_date(buf).and_then(|(read, year, month, day)| {
      NaiveDate::from_ymd_opt(year, month as u32, day as u32)
        .ok_or(DecodeError::custom("invalid date"))
        .map(|date| (read, date))
    })
  }
}

impl Varint for NaiveTime {
  const MIN_ENCODED_LEN: usize = 1;
  const MAX_ENCODED_LEN: usize = TimeBuffer::CAPACITY;

  #[inline]
  fn encoded_len(&self) -> usize {
    time_utils::encoded_time_len(self.nanosecond(), self.second() as u8, self.minute() as u8, self.hour() as u8)
  }

  #[inline]
  fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
    time_utils::encode_time_to(self.nanosecond(), self.second() as u8, self.minute() as u8, self.hour() as u8, buf)
  }

  #[inline]
  fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError>
  where
    Self: Sized,
  {
    time_utils::decode_time(buf).and_then(|(read, nano, second, minute, hour)| {
      // Construct NaiveTime from components
      NaiveTime::from_hms_nano_opt(hour as u32, minute as u32, second as u32, nano)
        .ok_or(DecodeError::custom(
          "invalid hour, minute, second and/or nanosecond",
        ))
        .map(|time| (read, time))
    })
  }
}

impl Varint for NaiveDateTime {
  const MIN_ENCODED_LEN: usize = 1;
  const MAX_ENCODED_LEN: usize = DateTimeBuffer::CAPACITY;

  #[inline]
  fn encoded_len(&self) -> usize {
    time_utils::encoded_datetime_len(self.year(), self.month() as u8, self.day() as u8, self.hour() as u8, self.minute() as u8, self.second() as u8, self.nanosecond())
  }

  #[inline]
  fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
    time_utils::encode_datetime_to(self.year(), self.month() as u8, self.day() as u8, self.hour() as u8, self.minute() as u8, self.second() as u8, self.nanosecond(), buf)
  }

  #[inline]
  fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError>
  where
    Self: Sized,
  {
    time_utils::decode_datetime(buf).and_then(|(
      read,
      year,
      month,
      day,
      hour,
      minute,
      second,
      nano,
    )| {
      // Create date and time
      let date =
        NaiveDate::from_ymd_opt(year, month as u32, day as u32).ok_or(DecodeError::custom("invalid date"))?;

      let time = NaiveTime::from_hms_nano_opt(hour as u32, minute as u32, second as u32, nano).ok_or(
        DecodeError::custom("invalid hour, minute, second and/or nanosecond"),
      )?;

      // Combine into NaiveDateTime
      Ok((read, NaiveDateTime::new(date, time)))
    })
  }
}

impl Varint for DateTime<Utc> {
  const MIN_ENCODED_LEN: usize = i128::MIN_ENCODED_LEN;

  const MAX_ENCODED_LEN: usize = i128::MAX_ENCODED_LEN;

  fn encoded_len(&self) -> usize {
    self.naive_utc().encoded_len()
  }

  fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
    self.naive_utc().encode(buf)
  }

  fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError>
  where
    Self: Sized,
  {
    let (read, naive_utc) = NaiveDateTime::decode(buf)?;
    Ok((read, DateTime::from_naive_utc_and_offset(naive_utc, Utc)))
  }
}
