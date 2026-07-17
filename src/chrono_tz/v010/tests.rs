use quickcheck::Arbitrary;

use super::{
  ConstDecodeError, TZ_VALUES, TZ_VARIANTS, Varint, decode_tz, encode_i16_varint_to, encode_tz,
  encoded_tz_len,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Tz(super::Tz);

impl From<Tz> for super::Tz {
  fn from(value: Tz) -> Self {
    value.0
  }
}

impl Arbitrary for Tz {
  fn arbitrary(g: &mut quickcheck::Gen) -> Self {
    let val = (u16::arbitrary(g) % super::TZ_VARIANTS.len() as u16) as i16;
    let idx = TZ_VALUES.iter().position(|&v| v == val).unwrap();
    Self(TZ_VARIANTS[idx])
  }
}

fuzzy!(@varint_into (Tz(super::Tz)));
fuzzy!(@const_varint_into (Tz(super::Tz)));

#[test]
fn decode_tz_round_trips_first_middle_last() {
  let first = TZ_VARIANTS[0];
  let last = TZ_VARIANTS[TZ_VARIANTS.len() - 1];
  let middle = TZ_VARIANTS[TZ_VARIANTS.len() / 2];

  for tz in [first, middle, last] {
    let encoded = encode_tz(tz);
    assert_eq!(encoded.len(), encoded_tz_len(tz).get());

    let (bytes_read, decoded) = decode_tz(&encoded).expect("valid timezone must decode");
    assert_eq!(decoded, tz);
    assert_eq!(bytes_read.get(), encoded.len());
  }
}

#[test]
fn decode_tz_rejects_out_of_range_and_negative_values() {
  // One past the last valid discriminant, and a comfortably out-of-range value.
  for invalid in [TZ_VARIANTS.len() as i16, i16::MAX] {
    let mut buf = [0u8; 4];
    let len = encode_i16_varint_to(invalid, &mut buf).unwrap();
    assert_eq!(
      decode_tz(&buf[..len.get()]),
      Err(ConstDecodeError::other("Invalid timezone value"))
    );
  }

  // Negative values are never valid discriminants.
  for invalid in [-1i16, i16::MIN] {
    let mut buf = [0u8; 4];
    let len = encode_i16_varint_to(invalid, &mut buf).unwrap();
    assert_eq!(
      decode_tz(&buf[..len.get()]),
      Err(ConstDecodeError::other("Invalid timezone value"))
    );
  }
}

#[test]
fn decode_tz_round_trips_every_variant() {
  for (i, &tz) in TZ_VARIANTS.iter().enumerate() {
    let encoded = encode_tz(tz);
    let (bytes_read, decoded) = decode_tz(&encoded).expect("every table entry must decode");
    assert_eq!(decoded, tz, "mismatch at index {i}");
    assert_eq!(bytes_read.get(), encoded.len());
  }
}
