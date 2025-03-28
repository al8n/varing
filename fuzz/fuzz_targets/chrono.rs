
#![no_main]

use arbitrary::Arbitrary;
use ::chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use const_varint::*;
use libfuzzer_sys::fuzz_target;

macro_rules! fuzzy {
    ($($ty:ty), +$(,)?) => {
        $(
            paste::paste! {
                fn [<check_ $ty:snake>](value: $ty) {
                    {
                        {
                            let mut buf = [0; <$ty>::MAX_ENCODED_LEN];
                            let encoded_len = value.encode(&mut buf).unwrap();
                            assert!(!(encoded_len != value.encoded_len() || !(value.encoded_len() <= <$ty>::MAX_ENCODED_LEN)));
                            let consumed = consume_varint(&buf).unwrap();
                            assert_eq!(consumed, encoded_len);

                            let (bytes_read, decoded) = <$ty>::decode(&buf).unwrap();
                            assert!(value == decoded && encoded_len == bytes_read);
                        }
                    }
                }
            }
        )*
    };
}

type ChronoUtc = DateTime<Utc>;

fuzzy!(Duration, NaiveDateTime, NaiveDate, NaiveTime, ChronoUtc);

#[derive(Debug, Clone, Copy)]
struct Time {
  hour: u32,
  minute: u32,
  second: u32,
  nanos: u32,
}

impl From<Time> for NaiveTime {
  fn from(value: Time) -> Self {
    NaiveTime::from_hms_nano_opt(value.hour, value.minute, value.second, value.nanos).unwrap()
  }
}

impl<'a> Arbitrary<'a> for Time {
  fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
    Ok(Self {
      hour: u.int_in_range(0..=23)?,
      minute: u.int_in_range(0..=59)?,
      second: u.int_in_range(0..=59)?,
      nanos: u.int_in_range(0..=999_999_999)?,
    })
  }
}

#[derive(Debug, Clone, Copy, arbitrary::Arbitrary)]
struct ArbitraryDateTime {
  date: NaiveDate,
  time: Time,
}

impl From<ArbitraryDateTime> for NaiveDateTime {
  fn from(value: ArbitraryDateTime) -> Self {
    NaiveDateTime::new(value.date, value.time.into())
  }
}

#[derive(Debug, Clone, Copy, arbitrary::Arbitrary)]
enum Types {
  Duration(Duration),
  NaiveDateTime(ArbitraryDateTime),
  NaiveDate(NaiveDate),
  NaiveTime(Time),
  Utc(DateTime<Utc>),
}

impl Types {
  fn check(self) {
    match self {
      Self::Duration(value) => check_duration(value),
      Self::NaiveDateTime(value) => check_naive_date_time(value.into()),
      Self::NaiveDate(value) => check_naive_date(value),
      Self::NaiveTime(value) => check_naive_time(value.into()),
      Self::Utc(value) => check_chrono_utc(value),
    }
  }
}

fuzz_target!(|data: Types| {
  data.check();
});
