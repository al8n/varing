use crate::{
  time_utils::{self, DurationBuffer},
  DecodeError, EncodeError, Varint,
};

use chrono_0_4::{
  DateTime, Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, Timelike, Utc,
};

pub use time_utils::{DateBuffer, DateTimeBuffer, TimeBuffer};

/// Returns the encoded length of the value in LEB128 variable length format.
/// The returned value will be in range [`Duration::ENCODED_LEN_RANGE`].
#[inline]
pub const fn encoded_duration_len(duration: &Duration) -> usize {
  time_utils::encoded_secs_and_subsec_nanos_len(duration.num_seconds(), duration.subsec_nanos())
}

/// Encodes a `Duration` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub const fn encode_duration(duration: &Duration) -> DurationBuffer {
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
        None => Err(DecodeError::other(
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
        .ok_or(DecodeError::other("invalid date"))
        .map(|date| (read, date))
    })
  }
}

impl Varint for NaiveTime {
  const MIN_ENCODED_LEN: usize = 1;
  const MAX_ENCODED_LEN: usize = TimeBuffer::CAPACITY;

  #[inline]
  fn encoded_len(&self) -> usize {
    time_utils::encoded_time_len(
      self.nanosecond(),
      self.second() as u8,
      self.minute() as u8,
      self.hour() as u8,
    )
  }

  #[inline]
  fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
    time_utils::encode_time_to(
      self.nanosecond(),
      self.second() as u8,
      self.minute() as u8,
      self.hour() as u8,
      buf,
    )
  }

  #[inline]
  fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError>
  where
    Self: Sized,
  {
    time_utils::decode_time(buf).and_then(|(read, nano, second, minute, hour)| {
      // Construct NaiveTime from components
      NaiveTime::from_hms_nano_opt(hour as u32, minute as u32, second as u32, nano)
        .ok_or(DecodeError::other(
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
  fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
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
  }

  #[inline]
  fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError>
  where
    Self: Sized,
  {
    time_utils::decode_datetime(buf).and_then(
      |(read, year, month, day, hour, minute, second, nano)| {
        // Create date and time
        let date = NaiveDate::from_ymd_opt(year, month as u32, day as u32)
          .ok_or(DecodeError::other("invalid date"))?;

        let time = NaiveTime::from_hms_nano_opt(hour as u32, minute as u32, second as u32, nano)
          .ok_or(DecodeError::other(
            "invalid hour, minute, second and/or nanosecond",
          ))?;

        // Combine into NaiveDateTime
        Ok((read, NaiveDateTime::new(date, time)))
      },
    )
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

            let mut buf = [0; <<[< Time $ty >] as IntoChrono>::Target>::MAX_ENCODED_LEN];
            let Ok(encoded_len) = value.encode(&mut buf) else {
              return false;
            };
            if encoded_len != value.encoded_len() || (value.encoded_len() > <Duration>::MAX_ENCODED_LEN) {
              return false;
            }

            let Ok(consumed) = crate::consume_varint(&buf) else {
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
    if encoded.len() != encoded_duration_len(&value)
      || (encoded.len() > <Duration>::MAX_ENCODED_LEN)
    {
      return false;
    }

    let Ok(consumed) = crate::consume_varint(&encoded) else {
      return false;
    };
    if consumed != encoded.len() {
      return false;
    }

    if let Ok((bytes_read, decoded)) = decode_duration(&encoded) {
      value == decoded && encoded.len() == bytes_read
    } else {
      false
    }
  }

  #[quickcheck_macros::quickcheck]
  fn fuzzy_duration_varint(value: DurationWrapper) -> bool {
    let value = value.0;
    let mut buf = [0; <Duration>::MAX_ENCODED_LEN];
    let Ok(encoded_len) = value.encode(&mut buf) else {
      return false;
    };
    if encoded_len != value.encoded_len() || (value.encoded_len() > <Duration>::MAX_ENCODED_LEN) {
      return false;
    }

    let Ok(consumed) = crate::consume_varint(&buf) else {
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
}
