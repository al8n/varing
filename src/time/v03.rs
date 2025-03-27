use time_0_3::{Date, Duration, Month, PrimitiveDateTime, Time, UtcDateTime};

use crate::{
  decode_i128_varint, decode_i32_varint, decode_u64_varint, encode_i128_varint,
  encode_i128_varint_to, encode_i32_varint, encode_i32_varint_to, encode_u64_varint,
  encode_u64_varint_to, encoded_i128_varint_len, encoded_i32_varint_len, encoded_u64_varint_len,
  DecodeError, EncodeError, I128VarintBuffer, I32VarintBuffer, U64VarintBuffer, Varint,
};

macro_rules! impl_varint_for_time {
  ($($ty:ident($inner:ident).$fn:ident), +$(,)?) => {
    paste::paste! {
      $(
        impl Varint for $ty {
          const MIN_ENCODED_LEN: usize = $inner::MIN_ENCODED_LEN;

          const MAX_ENCODED_LEN: usize = $inner::MAX_ENCODED_LEN;

          fn encoded_len(&self) -> usize {
            [< encoded_ $fn _len >](self)
          }

          fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
            [< encode_ $fn _to >](self, buf)
          }

          fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError>
          where
            Self: Sized,
          {
            [< decode_ $fn >](buf)
          }
        }
      )*
    }
  };
}

/// A buffer for storing LEB128 encoded [`Duration`] value.
pub type DurationBuffer = I128VarintBuffer;

/// A buffer for storing LEB128 encoded [`Date`] value.
pub type DateBuffer = I32VarintBuffer;

/// A buffer for storing LEB128 encoded [`UtcDateTime`] value.
pub type UtcDateTimeBuffer = I128VarintBuffer;

/// A buffer for storing LEB128 encoded [`PrimitiveDateTime`] value.
pub type PrimitiveDateTimeBuffer = I128VarintBuffer;

/// A buffer for storing LEB128 encoded [`Time`] value.
pub type TimeBuffer = U64VarintBuffer;

/// Returns the encoded length of the value in LEB128 variable length format.
/// The returned value will be in range [`Duration::ENCODED_LEN_RANGE`].
#[inline]
pub const fn encoded_duration_len(duration: &Duration) -> usize {
  // Use lower 96 bits: 64 for seconds, 32 for nanos
  let value = ((duration.whole_seconds() as i128) << 32) | (duration.subsec_nanoseconds() as i128);
  encoded_i128_varint_len(value)
}

/// Encodes a `Duration` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub const fn encode_duration(duration: &Duration) -> DurationBuffer {
  // Use lower 96 bits: 64 for seconds, 32 for nanos
  let value = ((duration.whole_seconds() as i128) << 32) | (duration.subsec_nanoseconds() as i128);
  encode_i128_varint(value)
}

/// Encodes a `Duration` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub const fn encode_duration_to(duration: &Duration, buf: &mut [u8]) -> Result<usize, EncodeError> {
  // Use lower 96 bits: 64 for seconds, 32 for nanos
  let value = ((duration.whole_seconds() as i128) << 32) | (duration.subsec_nanoseconds() as i128);
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
      Ok((bytes_read, Duration::new(secs, nanos)))
    }
    Err(e) => Err(e),
  }
}

/// Returns the encoded length of the value in LEB128 variable length format.
/// The returned value will be in range [`Date::ENCODED_LEN_RANGE`].
#[inline]
pub const fn encoded_date_len(date: &Date) -> usize {
  encoded_i32_varint_len(date_to_merged(date))
}

/// Encodes a `Date` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub const fn encode_date(date: &Date) -> DateBuffer {
  encode_i32_varint(date_to_merged(date))
}

/// Encodes a `Date` value into LEB128 variable length format, and writes it to the buffer.
///
/// Returns the number of bytes written to the buffer.
#[inline]
pub const fn encode_date_to(date: &Date, buf: &mut [u8]) -> Result<usize, EncodeError> {
  encode_i32_varint_to(date_to_merged(date), buf)
}

/// Decodes a `Date` in LEB128 encoded format from the buffer.
///
/// Returns the bytes readed and the decoded value if successful.
#[inline]
pub const fn decode_date(buf: &[u8]) -> Result<(usize, Date), DecodeError> {
  match decode_i32_varint(buf) {
    Ok((bytes_read, value)) => {
      let day = (value & 0b11111) as u8; // get lower 5 bits
      let month = match u8_to_month(((value >> 5) & 0b1111) as u8) {
        Ok(month) => month,
        Err(e) => return Err(e),
      }; // get next 4 bits
      let year = value >> 9; // get remaining 16 bits

      match Date::from_calendar_date(year, month, day) {
        Ok(date) => Ok((bytes_read, date)),
        Err(_) => Err(DecodeError::custom("invalid date value")),
      }
    }
    Err(e) => Err(e),
  }
}

/// Returns the encoded length of the value in LEB128 variable length format.
/// The returned value will be in range [`UtcDateTime::ENCODED_LEN_RANGE`].
#[inline]
pub const fn encoded_utc_len(dt: &UtcDateTime) -> usize {
  encoded_i128_varint_len(utc_to_merged(dt))
}

/// Encodes a `UtcDateTime` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub const fn encode_utc(dt: &UtcDateTime) -> UtcDateTimeBuffer {
  encode_i128_varint(utc_to_merged(dt))
}

/// Encodes a `UtcDateTime` value into LEB128 variable length format, and writes it to the buffer.
///
/// Returns the number of bytes written to the buffer.
#[inline]
pub const fn encode_utc_to(dt: &UtcDateTime, buf: &mut [u8]) -> Result<usize, EncodeError> {
  encode_i128_varint_to(utc_to_merged(dt), buf)
}

/// Decodes a `UtcDateTime` in LEB128 encoded format from the buffer.
///
/// Returns the bytes readed and the decoded value if successful.
#[inline]
pub const fn decode_utc(buf: &[u8]) -> Result<(usize, UtcDateTime), DecodeError> {
  match decode_i128_varint(buf) {
    Ok((bytes_read, encoded)) => {
      // Extract timestamp and nanoseconds
      let nanos = (encoded & 0x3FFFFFFF) as u32; // 30 bits for nanoseconds
      let timestamp_seconds = (encoded >> 30) as i64; // 64 bits for timestamp

      // Recreate the UtcDateTime from timestamp and nanoseconds
      match UtcDateTime::from_unix_timestamp_nanos(
        timestamp_seconds as i128 * 1_000_000_000 + nanos as i128,
      ) {
        Ok(dt) => Ok((bytes_read, dt)),
        Err(_) => Err(DecodeError::custom("invalid timestamp value")),
      }
    }
    Err(e) => Err(e),
  }
}

/// Returns the encoded length of the value in LEB128 variable length format.
/// The returned value will be in range [`PrimitiveDateTime::ENCODED_LEN_RANGE`].
#[inline]
pub const fn encoded_datetime_len(dt: &PrimitiveDateTime) -> usize {
  encoded_i128_varint_len(datetime_to_merged(dt))
}

/// Encodes a `PrimitiveDateTime` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub const fn encode_datetime(dt: &PrimitiveDateTime) -> PrimitiveDateTimeBuffer {
  encode_i128_varint(datetime_to_merged(dt))
}

/// Encodes a `PrimitiveDateTime` value into LEB128 variable length format, and writes it to the buffer.
///
/// Returns the number of bytes written to the buffer.
#[inline]
pub const fn encode_datetime_to(
  dt: &PrimitiveDateTime,
  buf: &mut [u8],
) -> Result<usize, EncodeError> {
  encode_i128_varint_to(datetime_to_merged(dt), buf)
}

/// Decodes a `PrimitiveDateTime` in LEB128 encoded format from the buffer.
///
/// Returns the bytes readed and the decoded value if successful.
#[inline]
pub const fn decode_datetime(buf: &[u8]) -> Result<(usize, PrimitiveDateTime), DecodeError> {
  match decode_i128_varint(buf) {
    Ok((bytes_read, encoded)) => {
      // Extract components
      let nano = (encoded & 0x3FFFFFFF) as u32; // 30 bits for nanoseconds
      let second = ((encoded >> 30) & 0x3F) as u8; // 6 bits for seconds
      let minute = ((encoded >> 36) & 0x3F) as u8; // 6 bits for minutes
      let hour = ((encoded >> 42) & 0x1F) as u8; // 5 bits for hours
      let day = ((encoded >> 47) & 0x1F) as u8; // 5 bits for day
      let month = match u8_to_month(((encoded >> 52) & 0xF) as u8) {
        Ok(month) => month,
        Err(e) => return Err(e),
      }; // 4 bits for month
      let year = ((encoded >> 56) & 0xFFFF) as i32; // 16 bits for year

      // Create date and time components
      let date = match Date::from_calendar_date(year, month, day) {
        Ok(date) => date,
        Err(_) => return Err(DecodeError::custom("invalid date value")),
      };
      let time = match Time::from_hms_nano(hour, minute, second, nano) {
        Ok(time) => time,
        Err(_) => return Err(DecodeError::custom("invalid time value")),
      };

      // Combine into PrimitiveDateTime
      Ok((bytes_read, PrimitiveDateTime::new(date, time)))
    }
    Err(e) => Err(e),
  }
}

/// Returns the encoded length of the value in LEB128 variable length format.
/// The returned value will be in range [`Time::ENCODED_LEN_RANGE`].
#[inline]
pub const fn encoded_time_len(time: &Time) -> usize {
  encoded_u64_varint_len(time_to_merged(time))
}

/// Encodes a `Time` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub const fn encode_time(time: &Time) -> TimeBuffer {
  encode_u64_varint(time_to_merged(time))
}

/// Encodes a `Time` value into LEB128 variable length format, and writes it to the buffer.
///
/// Returns the number of bytes written to the buffer.
#[inline]
pub const fn encode_time_to(time: &Time, buf: &mut [u8]) -> Result<usize, EncodeError> {
  encode_u64_varint_to(time_to_merged(time), buf)
}

/// Decodes a `Time` in LEB128 encoded format from the buffer.
///
/// Returns the bytes readed and the decoded value if successful.
#[inline]
pub const fn decode_time(buf: &[u8]) -> Result<(usize, Time), DecodeError> {
  match decode_u64_varint(buf) {
    Ok((bytes_read, encoded)) => {
      // Extract components
      let nano = (encoded & 0x3FFFFFFF) as u32; // 30 bits for nanoseconds
      let second = ((encoded >> 30) & 0x3F) as u8; // 6 bits for seconds
      let minute = ((encoded >> 36) & 0x3F) as u8; // 6 bits for minutes
      let hour = ((encoded >> 42) & 0x1F) as u8; // 5 bits for hours

      // Create Time
      match Time::from_hms_nano(hour, minute, second, nano) {
        Ok(time) => Ok((bytes_read, time)),
        Err(_) => Err(DecodeError::custom("invalid time value")),
      }
    }
    Err(e) => Err(e),
  }
}

impl_varint_for_time!(
  Duration(i128).duration,
  Time(u64).time,
  PrimitiveDateTime(i128).datetime,
  UtcDateTime(i128).utc,
  Date(i32).date
);

#[inline]
const fn date_to_merged(date: &Date) -> i32 {
  let day = date.day() as i32; // 1-31: 5 bits
  let month = date.month() as u8 as i32; // 1-12: 4 bits
  let year = date.year(); // year: 16 bits

  // Place smallest values in lower bits for LEB128 efficiency
  day | (month << 5) | (year << 9)
}

#[inline]
const fn utc_to_merged(dt: &UtcDateTime) -> i128 {
  // Get Unix timestamp (seconds since 1970-01-01 00:00:00 UTC)
  let timestamp_seconds = dt.unix_timestamp() as i128;

  // Get nanosecond component (0-999,999,999)
  let nanos = dt.nanosecond() as i128;

  // Place nanoseconds in lower bits to optimize for LEB128
  nanos | (timestamp_seconds << 30)
}

#[inline]
const fn datetime_to_merged(dt: &PrimitiveDateTime) -> i128 {
  let date = dt.date();
  let time = dt.time();

  let nano = time.nanosecond() as i128; // 0-999,999,999: 30 bits
  let second = time.second() as i128; // 0-59: 6 bits
  let minute = time.minute() as i128; // 0-59: 6 bits
  let hour = time.hour() as i128; // 0-23: 5 bits
  let day = date.day() as i128; // 1-31: 5 bits
  let month = date.month() as u8 as i128; // 1-12: 4 bits
  let year = date.year() as i128; // ±32767: 16 bits

  // Place smaller values in lower bits for LEB128 efficiency
  nano | (second << 30) | (minute << 36) | (hour << 42) | (day << 47) | (month << 52) | (year << 56)
}

const fn u8_to_month(val: u8) -> Result<Month, DecodeError> {
  Ok(match val {
    1 => Month::January,
    2 => Month::February,
    3 => Month::March,
    4 => Month::April,
    5 => Month::May,
    6 => Month::June,
    7 => Month::July,
    8 => Month::August,
    9 => Month::December,
    10 => Month::October,
    11 => Month::November,
    12 => Month::December,
    _ => return Err(DecodeError::custom("invalid month value")),
  })
}

#[inline]
const fn time_to_merged(time: &Time) -> u64 {
  let nano = time.nanosecond() as u64; // 0-999,999,999: 30 bits
  let second = time.second() as u64; // 0-59: 6 bits
  let minute = time.minute() as u64; // 0-59: 6 bits
  let hour = time.hour() as u64; // 0-23: 5 bits

  // Place components with likely smaller/zero values in the lower bits
  nano | (second << 30) | (minute << 36) | (hour << 42)
}
