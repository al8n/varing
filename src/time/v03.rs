use core::num::NonZeroUsize;

use time_0_3::{Date, Duration, Month, PrimitiveDateTime, Time, UtcDateTime};

use crate::{
  ConstDecodeError, ConstEncodeError, DecodeError, EncodeError, NON_ZERO_USIZE_ONE, Varint,
  time_utils::{self, DurationBuffer},
};

pub use time_utils::{DateBuffer, DateTimeBuffer, TimeBuffer};

macro_rules! impl_varint_for_time {
  ($($ty:ident($max:expr, $min:expr).$fn:ident), +$(,)?) => {
    paste::paste! {
      $(
        impl Varint for $ty {
          const MIN_ENCODED_LEN: NonZeroUsize = {
            assert!($min.get() <= $max.get(), concat!("`", stringify!($ty), "::MIN_ENCODED_LEN` must be less than or equal to `", stringify!($ty), "MAX_ENCODED_LEN`"));
            $min
          };

          const MAX_ENCODED_LEN: NonZeroUsize = {
            assert!($max.get() >= $min.get(), concat!("`", stringify!($ty), "::MAX_ENCODED_LEN` must be greater than or equal to `", stringify!($ty), "MIN_ENCODED_LEN`"));
            $max
          };

          fn encoded_len(&self) -> NonZeroUsize {
            [< encoded_ $fn _len >](self)
          }

          fn encode(&self, buf: &mut [u8]) -> Result<NonZeroUsize, EncodeError> {
            [< encode_ $fn _to >](self, buf).map_err(Into::into)
          }

          fn decode(buf: &[u8]) -> Result<(NonZeroUsize, Self), DecodeError>
          where
            Self: Sized,
          {
            [< decode_ $fn >](buf).map_err(Into::into)
          }
        }
      )*
    }
  };
}

/// Returns the encoded length of the value in LEB128 variable length format.
/// The returned value will be in range [`Duration::ENCODED_LEN_RANGE`].
#[inline]
pub const fn encoded_duration_len(duration: &Duration) -> NonZeroUsize {
  time_utils::encoded_secs_and_subsec_nanos_len(
    duration.whole_seconds(),
    duration.subsec_nanoseconds(),
  )
}

/// Encodes a `Duration` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub const fn encode_duration(duration: &Duration) -> DurationBuffer {
  time_utils::encode_secs_and_subsec_nanos(duration.whole_seconds(), duration.subsec_nanoseconds())
}

/// Encodes a `Duration` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub const fn encode_duration_to(
  duration: &Duration,
  buf: &mut [u8],
) -> Result<NonZeroUsize, ConstEncodeError> {
  time_utils::encode_secs_and_subsec_nanos_to(
    duration.whole_seconds(),
    duration.subsec_nanoseconds(),
    buf,
  )
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
      // `time::Duration` always has `|subsec_nanoseconds()| < 1_000_000_000`;
      // reject anything else so `Duration::new`'s nanos->secs carry is zero and
      // cannot overflow `secs` (which would panic).
      if nanos <= -1_000_000_000 || nanos >= 1_000_000_000 {
        return Err(ConstDecodeError::other("nanos out of range"));
      }
      // Canonicity: a well-formed `time::Duration` keeps `whole_seconds()` and
      // `subsec_nanoseconds()` in sign agreement, so mixed signs (e.g.
      // `secs = 1, nanos = -1`) never come from the encoder. Left unchecked they
      // normalize through `Duration::new` into a valid-but-different value
      // (0.999999999s), aliasing a canonical encoding, so reject them.
      if (secs > 0 && nanos < 0) || (secs < 0 && nanos > 0) {
        return Err(ConstDecodeError::other("non-canonical duration"));
      }
      Ok((bytes_read, Duration::new(secs, nanos)))
    }
    Err(e) => Err(e),
  }
}

/// Returns the encoded length of the value in LEB128 variable length format.
/// The returned value will be in range [`Date::ENCODED_LEN_RANGE`].
#[inline]
pub const fn encoded_date_len(date: &Date) -> NonZeroUsize {
  time_utils::encoded_date_len(date.year(), date.month() as u8, date.day())
}

/// Encodes a `Date` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub const fn encode_date(date: &Date) -> DateBuffer {
  time_utils::encode_date(date.year(), date.month() as u8, date.day())
}

/// Encodes a `Date` value into LEB128 variable length format, and writes it to the buffer.
///
/// Returns the number of bytes written to the buffer.
#[inline]
pub const fn encode_date_to(date: &Date, buf: &mut [u8]) -> Result<NonZeroUsize, ConstEncodeError> {
  time_utils::encode_date_to(date.year(), date.month() as u8, date.day(), buf)
}

/// Decodes a `Date` in LEB128 encoded format from the buffer.
///
/// Returns the bytes readed and the decoded value if successful.
#[inline]
pub const fn decode_date(buf: &[u8]) -> Result<(NonZeroUsize, Date), ConstDecodeError> {
  match time_utils::decode_date(buf) {
    Ok((bytes_read, year, month, day)) => {
      let month = match u8_to_month(month) {
        Ok(month) => month,
        Err(e) => return Err(e),
      };
      match Date::from_calendar_date(year, month, day) {
        Ok(date) => Ok((bytes_read, date)),
        Err(_) => Err(ConstDecodeError::other("invalid date value")),
      }
    }
    Err(e) => Err(e),
  }
}

/// Returns the encoded length of the value in LEB128 variable length format.
/// The returned value will be in range [`UtcDateTime::ENCODED_LEN_RANGE`].
#[inline]
pub const fn encoded_utc_len(dt: &UtcDateTime) -> NonZeroUsize {
  encoded_datetime_len(&PrimitiveDateTime::new(dt.date(), dt.time()))
}

/// Encodes a `UtcDateTime` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub const fn encode_utc(dt: &UtcDateTime) -> DateTimeBuffer {
  encode_datetime(&PrimitiveDateTime::new(dt.date(), dt.time()))
}

/// Encodes a `UtcDateTime` value into LEB128 variable length format, and writes it to the buffer.
///
/// Returns the number of bytes written to the buffer.
#[inline]
pub const fn encode_utc_to(
  dt: &UtcDateTime,
  buf: &mut [u8],
) -> Result<NonZeroUsize, ConstEncodeError> {
  encode_datetime_to(&PrimitiveDateTime::new(dt.date(), dt.time()), buf)
}

/// Decodes a `UtcDateTime` in LEB128 encoded format from the buffer.
///
/// Returns the bytes readed and the decoded value if successful.
#[inline]
pub const fn decode_utc(buf: &[u8]) -> Result<(NonZeroUsize, UtcDateTime), ConstDecodeError> {
  match decode_datetime(buf) {
    Ok((bytes_read, dt)) => Ok((bytes_read, UtcDateTime::new(dt.date(), dt.time()))),
    Err(e) => Err(e),
  }
}

/// Returns the encoded length of the value in LEB128 variable length format.
/// The returned value will be in range [`PrimitiveDateTime::ENCODED_LEN_RANGE`].
#[inline]
pub const fn encoded_datetime_len(dt: &PrimitiveDateTime) -> NonZeroUsize {
  time_utils::encoded_datetime_len(
    dt.year(),
    dt.month() as u8,
    dt.day(),
    dt.hour(),
    dt.minute(),
    dt.second(),
    dt.nanosecond(),
  )
}

/// Encodes a `PrimitiveDateTime` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub const fn encode_datetime(dt: &PrimitiveDateTime) -> DateTimeBuffer {
  time_utils::encode_datetime(
    dt.year(),
    dt.month() as u8,
    dt.day(),
    dt.hour(),
    dt.minute(),
    dt.second(),
    dt.nanosecond(),
  )
}

/// Encodes a `PrimitiveDateTime` value into LEB128 variable length format, and writes it to the buffer.
///
/// Returns the number of bytes written to the buffer.
#[inline]
pub const fn encode_datetime_to(
  dt: &PrimitiveDateTime,
  buf: &mut [u8],
) -> Result<NonZeroUsize, ConstEncodeError> {
  time_utils::encode_datetime_to(
    dt.year(),
    dt.month() as u8,
    dt.day(),
    dt.hour(),
    dt.minute(),
    dt.second(),
    dt.nanosecond(),
    buf,
  )
}

/// Decodes a `PrimitiveDateTime` in LEB128 encoded format from the buffer.
///
/// Returns the bytes readed and the decoded value if successful.
#[inline]
pub const fn decode_datetime(
  buf: &[u8],
) -> Result<(NonZeroUsize, PrimitiveDateTime), ConstDecodeError> {
  match time_utils::decode_datetime(buf) {
    Ok((bytes_read, year, month, day, hour, minute, second, nano)) => {
      let month = match u8_to_month(month) {
        Ok(month) => month,
        Err(e) => return Err(e),
      };

      // Create date and time components
      let date = match Date::from_calendar_date(year, month, day) {
        Ok(date) => date,
        Err(_) => return Err(ConstDecodeError::other("invalid date value")),
      };
      let time = match Time::from_hms_nano(hour, minute, second, nano) {
        Ok(time) => time,
        Err(_) => return Err(ConstDecodeError::other("invalid time value")),
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
pub const fn encoded_time_len(time: &Time) -> NonZeroUsize {
  time_utils::encoded_time_len(time.nanosecond(), time.second(), time.minute(), time.hour())
}

/// Encodes a `Time` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub const fn encode_time(time: &Time) -> TimeBuffer {
  time_utils::encode_time(time.nanosecond(), time.second(), time.minute(), time.hour())
}

/// Encodes a `Time` value into LEB128 variable length format, and writes it to the buffer.
///
/// Returns the number of bytes written to the buffer.
#[inline]
pub const fn encode_time_to(time: &Time, buf: &mut [u8]) -> Result<NonZeroUsize, ConstEncodeError> {
  time_utils::encode_time_to(
    time.nanosecond(),
    time.second(),
    time.minute(),
    time.hour(),
    buf,
  )
}

/// Decodes a `Time` in LEB128 encoded format from the buffer.
///
/// Returns the bytes readed and the decoded value if successful.
#[inline]
pub const fn decode_time(buf: &[u8]) -> Result<(NonZeroUsize, Time), ConstDecodeError> {
  match time_utils::decode_time(buf) {
    Ok((bytes_read, nano, second, minute, hour)) => {
      // Create Time
      match Time::from_hms_nano(hour, minute, second, nano) {
        Ok(time) => Ok((bytes_read, time)),
        Err(_) => Err(ConstDecodeError::other("invalid time value")),
      }
    }
    Err(e) => Err(e),
  }
}

impl_varint_for_time!(
  Duration(i128::MAX_ENCODED_LEN, i128::MIN_ENCODED_LEN).duration,
  Time(TimeBuffer::CAPACITY, u64::MIN_ENCODED_LEN).time,
  PrimitiveDateTime(DateTimeBuffer::CAPACITY, NON_ZERO_USIZE_ONE).datetime,
  UtcDateTime(i128::MAX_ENCODED_LEN, NON_ZERO_USIZE_ONE).utc,
  Date(DateBuffer::CAPACITY, NON_ZERO_USIZE_ONE).date
);

const fn u8_to_month(val: u8) -> Result<Month, ConstDecodeError> {
  Ok(match val {
    1 => Month::January,
    2 => Month::February,
    3 => Month::March,
    4 => Month::April,
    5 => Month::May,
    6 => Month::June,
    7 => Month::July,
    8 => Month::August,
    9 => Month::September,
    10 => Month::October,
    11 => Month::November,
    12 => Month::December,
    _ => return Err(ConstDecodeError::other("invalid month value")),
  })
}

#[cfg(all(test, feature = "std"))]
mod tests;
