use crate::{
  decode_i128_varint, decode_i32_varint, decode_i64_varint, encode_i128_varint,
  encode_i128_varint_to, encode_i32_varint_to, encode_i64_varint_to, encoded_i128_varint_len,
  encoded_i32_varint_len, encoded_i64_varint_len, DecodeError, EncodeError, I128VarintBuffer,
  Varint,
};

use chrono_0_4::{
  DateTime, Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, Timelike, Utc,
};

/// A buffer for storing LEB128 encoded [`Duration`] value.
pub type ChronoDurationBuffer = I128VarintBuffer;

/// A buffer for storing LEB128 encoded [`DataTime<Utc>`](Utc) value.
pub type ChronoUtcBuffer = I128VarintBuffer;

/// Returns the encoded length of the value in LEB128 variable length format.
/// The returned value will be in range [`Duration::ENCODED_LEN_RANGE`].
#[inline]
pub const fn encoded_duration_len(duration: &Duration) -> usize {
  // Use lower 96 bits: 64 for seconds, 32 for nanos
  let value = ((duration.num_seconds() as i128) << 32) | (duration.subsec_nanos() as i128);
  encoded_i128_varint_len(value)
}

/// Encodes a `Duration` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub const fn encode_duration(duration: &Duration) -> ChronoDurationBuffer {
  // Use lower 96 bits: 64 for seconds, 32 for nanos
  let value = ((duration.num_seconds() as i128) << 32) | (duration.subsec_nanos() as i128);
  encode_i128_varint(value)
}

/// Encodes a `Duration` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub const fn encode_duration_to(duration: &Duration, buf: &mut [u8]) -> Result<usize, EncodeError> {
  // Use lower 96 bits: 64 for seconds, 32 for nanos
  let value = ((duration.num_seconds() as i128) << 32) | (duration.subsec_nanos() as i128);
  encode_i128_varint_to(value, buf)
}

/// Decodes a `Duration` in LEB128 encoded format from the buffer.
///
/// Returns the bytes readed and the decoded value if successful.
#[inline]
pub const fn decode_duration(buf: &[u8]) -> Result<(usize, Duration), DecodeError> {
  match decode_i128_varint(buf) {
    Ok((bytes_read, value)) => {
      let secs = (value >> 32) as i64; // get upper 64 bits
      let nanos = (value & 0xFFFFFFFF) as i32; // get lower 32 bits
      match Duration::new(secs, nanos as u32) {
        Some(duration) => Ok((bytes_read, duration)),
        None => Err(DecodeError::custom(
          "duration is out of bounds, or nanos ≥ 1,000,000,000",
        )),
      }
    }
    Err(e) => Err(e),
  }
}

/// Returns the encoded length of the value in LEB128 variable length format.
/// The returned value will be in range [`DateTime::<Utc>::ENCODED_LEN_RANGE`](DateTime::ENCODED_LEN_RANGE).
#[inline]
pub const fn encode_utc(dt: &DateTime<Utc>) -> ChronoUtcBuffer {
  encode_i128_varint(utc_to_merged(dt))
}

/// Encodes a `DateTime<Utc>` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub const fn encode_utc_to(dt: &DateTime<Utc>, buf: &mut [u8]) -> Result<usize, EncodeError> {
  encode_i128_varint_to(utc_to_merged(dt), buf)
}

/// Decodes a `DateTime<Utc>` in LEB128 encoded format from the buffer.
///
/// Returns the bytes readed and the decoded value if successful.
#[inline]
pub const fn decode_utc(buf: &[u8]) -> Result<(usize, DateTime<Utc>), DecodeError> {
  match decode_i128_varint(buf) {
    Ok((read, val)) => match decode_merged_to_utc(val) {
      Ok(dt) => Ok((read, dt)),
      Err(e) => Err(e),
    },
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
  const MIN_ENCODED_LEN: usize = i32::MIN_ENCODED_LEN;
  const MAX_ENCODED_LEN: usize = i32::MAX_ENCODED_LEN;

  #[inline]
  fn encoded_len(&self) -> usize {
    encoded_i32_varint_len(native_date_to_merged(self))
  }

  #[inline]
  fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
    encode_i32_varint_to(native_date_to_merged(self), buf)
  }

  #[inline]
  fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError>
  where
    Self: Sized,
  {
    decode_i32_varint(buf).and_then(|(read, encoded)| {
      let day = (encoded & 0x1F) as u32; // 5 bits for day
      let month = ((encoded >> 5) & 0xF) as u32; // 4 bits for month
      let year = (encoded >> 9) & 0xFFFF; // 16 bits for year

      NaiveDate::from_ymd_opt(year, month, day)
        .ok_or(DecodeError::custom("invalid date"))
        .map(|date| (read, date))
    })
  }
}

impl Varint for NaiveTime {
  const MIN_ENCODED_LEN: usize = i64::MIN_ENCODED_LEN;
  const MAX_ENCODED_LEN: usize = i64::MAX_ENCODED_LEN;

  #[inline]
  fn encoded_len(&self) -> usize {
    encoded_i64_varint_len(native_time_to_merged(self))
  }

  #[inline]
  fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
    encode_i64_varint_to(native_time_to_merged(self), buf)
  }

  #[inline]
  fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError>
  where
    Self: Sized,
  {
    decode_i64_varint(buf).and_then(|(read, encoded)| {
      let nanos = (encoded & 0x3FFFFFFF) as u32; // 30 bits for nanoseconds
      let seconds = ((encoded >> 30) & 0x3F) as u32; // 6 bits for seconds
      let minutes = ((encoded >> 36) & 0x3F) as u32; // 6 bits for minutes
      let hours = ((encoded >> 42) & 0x1F) as u32; // 5 bits for hours

      // Construct NaiveTime from components
      NaiveTime::from_hms_nano_opt(hours, minutes, seconds, nanos)
        .ok_or(DecodeError::custom(
          "invalid hour, minute, second and/or nanosecond",
        ))
        .map(|time| (read, time))
    })
  }
}

impl Varint for NaiveDateTime {
  const MIN_ENCODED_LEN: usize = i128::MIN_ENCODED_LEN;
  const MAX_ENCODED_LEN: usize = i128::MAX_ENCODED_LEN;

  #[inline]
  fn encoded_len(&self) -> usize {
    encoded_i128_varint_len(native_date_time_to_merged(self))
  }

  #[inline]
  fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
    encode_i128_varint_to(native_date_time_to_merged(self), buf)
  }

  #[inline]
  fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError>
  where
    Self: Sized,
  {
    decode_i128_varint(buf).and_then(|(read, encoded)| {
      // Extract components
      let nano = (encoded & 0x3FFFFFFF) as u32; // 30 bits for nanoseconds
      let second = ((encoded >> 30) & 0x3F) as u32; // 6 bits for seconds
      let minute = ((encoded >> 36) & 0x3F) as u32; // 6 bits for minutes
      let hour = ((encoded >> 42) & 0x1F) as u32; // 5 bits for hours
      let day = ((encoded >> 47) & 0x1F) as u32; // 5 bits for day
      let month = ((encoded >> 52) & 0xF) as u32; // 4 bits for month
      let year = ((encoded >> 56) & 0xFFFF) as i32; // 16 bits for year

      // Create date and time
      let date =
        NaiveDate::from_ymd_opt(year, month, day).ok_or(DecodeError::custom("invalid date"))?;

      let time = NaiveTime::from_hms_nano_opt(hour, minute, second, nano).ok_or(
        DecodeError::custom("invalid hour, minute, second and/or nanosecond"),
      )?;

      // Combine into NaiveDateTime
      Ok((read, NaiveDateTime::new(date, time)))
    })
  }
}

impl Varint for DateTime<Utc> {
  const MIN_ENCODED_LEN: usize = i128::MAX_ENCODED_LEN;

  const MAX_ENCODED_LEN: usize = i128::MIN_ENCODED_LEN;

  fn encoded_len(&self) -> usize {
    encoded_i128_varint_len(utc_to_merged(self))
  }

  fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
    encode_i128_varint_to(utc_to_merged(self), buf)
  }

  fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError>
  where
    Self: Sized,
  {
    decode_i128_varint(buf)
      .and_then(|(read, encoded)| decode_merged_to_utc(encoded).map(|dt| (read, dt)))
  }
}

fn native_time_to_merged(time: &NaiveTime) -> i64 {
  let nano = time.nanosecond() as i64; // 0-999,999,999: 30 bits
  let second = time.second() as i64; // 0-59: 6 bits
  let minute = time.minute() as i64; // 0-59: 6 bits
  let hour = time.hour() as i64; // 0-23: 5 bits

  // Place lower-valued components in lower bits
  nano | (second << 30) | (minute << 36) | (hour << 42)
}

fn native_date_to_merged(date: &NaiveDate) -> i32 {
  let day = date.day() as i32; // 1-31: 5 bits
  let month = date.month() as i32; // 1-12: 4 bits
  let year = date.year(); // ±32767: 16 bits

  // Place in order of increasing range with day in lowest bits
  day | (month << 5) | (year << 9)
}

fn native_date_time_to_merged(dt: &NaiveDateTime) -> i128 {
  // Date components
  let year = dt.year() as i128;
  let month = dt.month() as i128;
  let day = dt.day() as i128;

  // Time components
  let hour = dt.hour() as i128;
  let minute = dt.minute() as i128;
  let second = dt.second() as i128;
  let nano = dt.nanosecond() as i128;

  // Place components with likely smaller values in the lower bits
  nano |                          // Nanoseconds in lowest 30 bits
  (second << 30) |                // Seconds shifted by 30 bits
  (minute << 36) |                // Minutes shifted by 36 bits
  (hour << 42) |                  // Hours shifted by 42 bits
  (day << 47) |                   // Day shifted by 47 bits
  (month << 52) |                 // Month shifted by 52 bits
  (year << 56) // Year in highest bits
}

#[inline]
const fn utc_to_merged(dt: &DateTime<Utc>) -> i128 {
  // Get Unix timestamp (seconds since 1970-01-01 00:00:00 UTC)
  let timestamp_seconds = dt.timestamp() as i128;

  // Get nanosecond component (0-999,999,999)
  let nanos = dt.timestamp_subsec_nanos() as i128;

  // Place nanoseconds in lower bits to optimize for LEB128
  nanos | (timestamp_seconds << 30)
}

#[inline]
const fn decode_merged_to_utc(merged: i128) -> Result<DateTime<Utc>, DecodeError> {
  // Extract timestamp and nanoseconds
  let nanos = (merged & 0x3FFFFFFF) as u32; // 30 bits for nanoseconds
  let timestamp_seconds = (merged >> 30) as i64; // 64 bits for timestamp

  // Recreate the DateTime<Utc> from timestamp and nanoseconds
  match DateTime::<Utc>::from_timestamp(timestamp_seconds, nanos) {
    Some(dt) => Ok(dt),
    None => Err(DecodeError::custom("invalid timestamp and/or nanosecond")),
  }
}
