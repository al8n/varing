use chrono_tz_0_10::{Tz, TZ_VARIANTS};

use crate::{
  decode_i16_varint, encode_i16_varint_to, encoded_i16_varint_len, utils::Buffer, DecodeError,
  EncodeError, Varint,
};

const TZ_VALUES: [i16; TZ_VARIANTS.len()] = const {
  let mut values = [0; TZ_VARIANTS.len()];
  let mut i = 0;
  while i < TZ_VARIANTS.len() {
    values[i] = TZ_VARIANTS[i] as i16;
    i += 1;
  }
  values
};

/// Returns the length of the encoded timezone value.
#[inline]
pub const fn encoded_tz_len(tz: Tz) -> usize {
  encoded_i16_varint_len(tz as i16)
}

/// Encodes the timezone value into the buffer.
///
/// Returns the number of bytes written to the buffer.
#[inline]
pub const fn encode_tz_to(tz: Tz, buf: &mut [u8]) -> Result<usize, EncodeError> {
  encode_i16_varint_to(tz as i16, buf)
}

/// Encodes the timezone value into the buffer.
#[inline]
pub const fn encode_tz(tz: Tz) -> Buffer<{ Tz::MAX_ENCODED_LEN + 1 }> {
  let mut buf = [0; Tz::MAX_ENCODED_LEN + 1];
  let len = match encode_tz_to(tz, &mut buf) {
    Ok(len) => len,
    Err(_) => panic!("Timezone value is larger than buffer capacity, please report bug to https://github.com/al8n/varing/issues"),
  };

  buf[Tz::MAX_ENCODED_LEN] = len as u8;

  Buffer::new(buf)
}

/// Decodes the timezone value from the buffer.
///
/// Returns the number of bytes read and the decoded timezone value.
#[inline]
pub const fn decode_tz(buf: &[u8]) -> Result<(usize, Tz), DecodeError> {
  match decode_i16_varint(buf) {
    Ok((len, tz)) => {
      let mut i = 0;
      let mut found = None;

      while i < TZ_VALUES.len() {
        if TZ_VALUES[i] == tz {
          found = Some(TZ_VARIANTS[i]);
          break;
        }
        i += 1;
      }

      if let Some(tz) = found {
        Ok((len, tz))
      } else {
        Err(DecodeError::custom("Invalid timezone value"))
      }
    }
    Err(err) => Err(err),
  }
}

impl Varint for Tz {
  const MIN_ENCODED_LEN: usize = i16::MIN_ENCODED_LEN;

  const MAX_ENCODED_LEN: usize = encoded_i16_varint_len(TZ_VALUES.len() as i16);

  #[inline]
  fn encoded_len(&self) -> usize {
    encoded_tz_len(*self)
  }

  #[inline]
  fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
    encode_tz_to(*self, buf)
  }

  #[inline]
  fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError>
  where
    Self: Sized,
  {
    decode_tz(buf)
  }
}

#[cfg(test)]
mod tests {
  use quickcheck::Arbitrary;

  use super::{decode_tz, encode_tz, encoded_tz_len, Varint, TZ_VALUES, TZ_VARIANTS};

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
}
