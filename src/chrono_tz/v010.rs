use chrono_tz_0_10::{TZ_VARIANTS, Tz};

use crate::{
  ConstDecodeError, ConstEncodeError, DecodeError, EncodeError, Varint, decode_i16_varint,
  encode_i16_varint_to, encoded_i16_varint_len, utils::Buffer,
};

use core::num::NonZeroUsize;

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
pub const fn encoded_tz_len(tz: Tz) -> NonZeroUsize {
  encoded_i16_varint_len(tz as i16)
}

/// Encodes the timezone value into the buffer.
///
/// Returns the number of bytes written to the buffer.
#[inline]
pub const fn encode_tz_to(tz: Tz, buf: &mut [u8]) -> Result<NonZeroUsize, ConstEncodeError> {
  encode_i16_varint_to(tz as i16, buf)
}

/// Encodes the timezone value into the buffer.
#[inline]
pub const fn encode_tz(tz: Tz) -> Buffer<{ Tz::MAX_ENCODED_LEN.get() + 1 }> {
  let mut buf = [0; Tz::MAX_ENCODED_LEN.get() + 1];
  let len = match encode_tz_to(tz, &mut buf) {
    Ok(len) => len,
    Err(_) => panic!(
      "Timezone value is larger than buffer capacity, please report bug to https://github.com/al8n/varing/issues"
    ),
  };

  buf[Tz::MAX_ENCODED_LEN.get()] = len.get() as u8;

  Buffer::new(buf)
}

/// Decodes the timezone value from the buffer.
///
/// Returns the number of bytes read and the decoded timezone value.
#[inline]
pub const fn decode_tz(buf: &[u8]) -> Result<(NonZeroUsize, Tz), ConstDecodeError> {
  match decode_i16_varint(buf) {
    Ok((len, tz)) => {
      // `chrono-tz`'s `Tz` enum has no explicit discriminants and `TZ_VARIANTS`
      // lists variants in the same order as the enum declaration, so
      // `TZ_VALUES[i] == i as i16` holds for every valid index `i`. That lets
      // us jump straight to the slot instead of scanning `TZ_VALUES` linearly.
      // The `TZ_VALUES[idx] == tz` guard keeps the exact same accept/reject
      // behavior as the old linear scan even if that invariant ever changes.
      let found = if tz >= 0 {
        let idx = tz as usize;
        if idx < TZ_VALUES.len() && TZ_VALUES[idx] == tz {
          Some(TZ_VARIANTS[idx])
        } else {
          None
        }
      } else {
        None
      };

      if let Some(tz) = found {
        Ok((len, tz))
      } else {
        Err(ConstDecodeError::other("Invalid timezone value"))
      }
    }
    Err(err) => Err(err),
  }
}

impl Varint for Tz {
  const MIN_ENCODED_LEN: NonZeroUsize = i16::MIN_ENCODED_LEN;

  const MAX_ENCODED_LEN: NonZeroUsize = encoded_i16_varint_len(TZ_VALUES.len() as i16);

  #[inline]
  fn encoded_len(&self) -> NonZeroUsize {
    encoded_tz_len(*self)
  }

  #[inline]
  fn encode(&self, buf: &mut [u8]) -> Result<NonZeroUsize, EncodeError> {
    encode_tz_to(*self, buf).map_err(Into::into)
  }

  #[inline]
  fn decode(buf: &[u8]) -> Result<(NonZeroUsize, Self), DecodeError>
  where
    Self: Sized,
  {
    decode_tz(buf).map_err(Into::into)
  }
}

#[cfg(test)]
mod tests;
