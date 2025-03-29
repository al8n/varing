use crate::{
  bnum::{decode_uint_d8, encode_uint_d8_to, encoded_uint_d8_len, Packable},
  utils::{pack_i128, pack_u128, unpack_i128, unpack_u128},
  DecodeError, EncodeError, Varint,
};
use ::bnum_0_13::{BInt, BIntD16, BIntD32, BIntD8, BUint, BUintD16, BUintD32, BUintD8};
use num_rational_0_4::Ratio;

type U256 = BUintD8<32>;

impl_varint_for_ratio!(128(U256));

/// Returns the encoded length of the `Ratio<u128>` value.
#[inline]
pub const fn encoded_ratio_u128_len(val: &Ratio<u128>) -> usize {
  encoded_uint_d8_len(&pack_u128(*val.numer(), *val.denom()))
}

/// Encodes the `Ratio<u128>` value.
#[inline]
pub const fn encode_ratio_u128(val: &Ratio<u128>, buf: &mut [u8]) -> Result<usize, EncodeError> {
  encode_uint_d8_to(pack_u128(*val.numer(), *val.denom()), buf)
}

/// Decodes the `Ratio<u128>` from the given buffer
///
/// Returns the bytes read and the value.
#[inline]
pub const fn decode_ratio_u128(buf: &[u8]) -> Result<(usize, Ratio<u128>), DecodeError> {
  match decode_uint_d8::<32>(buf) {
    Ok((read, val)) => {
      let (numer, denom) = unpack_u128(val);
      if denom == 0 {
        return Err(DecodeError::custom("denominator cannot be zero"));
      }
      Ok((read, Ratio::new_raw(numer, denom)))
    }
    Err(e) => Err(e),
  }
}

/// Returns the encoded length of the `Ratio<i128>` value.
#[inline]
pub const fn encoded_ratio_i128_len(val: &Ratio<i128>) -> usize {
  encoded_uint_d8_len(&pack_i128(*val.numer(), *val.denom()))
}

/// Encodes the `Ratio<i128>` value.
#[inline]
pub const fn encode_ratio_i128(val: &Ratio<i128>, buf: &mut [u8]) -> Result<usize, EncodeError> {
  encode_uint_d8_to(pack_i128(*val.numer(), *val.denom()), buf)
}

/// Decodes the `Ratio<i128>` from the given buffer
///
/// Returns the bytes read and the value.
#[inline]
pub const fn decode_ratio_i128(buf: &[u8]) -> Result<(usize, Ratio<i128>), DecodeError> {
  match decode_uint_d8::<32>(buf) {
    Ok((read, val)) => {
      let (numer, denom) = unpack_i128(val);
      if denom == 0 {
        return Err(DecodeError::custom("denominator cannot be zero"));
      }
      Ok((read, Ratio::new_raw(numer, denom)))
    }
    Err(e) => Err(e),
  }
}

macro_rules! impl_varint_for_ratio_bnum {
  (@unsigned $($base:ident($($bytes:literal),+$(,)?)), +$(,)?) => {
    paste::paste! {
      $(
        $(
          impl Varint for Ratio<$base<$bytes>> {
            const MIN_ENCODED_LEN: usize = $base::<{$bytes * 2}>::MAX_ENCODED_LEN;

            const MAX_ENCODED_LEN: usize = $base::<{$bytes * 2}>::MAX_ENCODED_LEN;

            fn encoded_len(&self) -> usize {
              Packable::<$base::<{$bytes * 2}>>::pack(*self.numer(), *self.denom()).encoded_len()
            }

            fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
              Packable::<$base::<{$bytes * 2}>>::pack(*self.numer(), *self.denom()).encode(buf)
            }

            fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError>
            where
              Self: Sized,
            {
              let (bytes_read, merged) = $base::< {$bytes * 2}>::decode(buf)?;
              let (numer, denom): ($base<$bytes>, $base<$bytes>) = Packable::<$base::<{$bytes * 2}>>::unpack(merged);
              if denom.is_zero() {
                return Err(DecodeError::custom("denominator cannot be zero"));
              }
              Ok((bytes_read, Ratio::new_raw(numer, denom)))
            }
          }
        )*
      )*
    }
  };
  (@signed $($base:ident <=> $unsigned:ident($($bytes:literal),+$(,)?)), +$(,)?) => {
    paste::paste! {
      $(
        $(
          impl Varint for Ratio<$base<$bytes>> {
            const MIN_ENCODED_LEN: usize = $unsigned::<{$bytes * 2}>::MAX_ENCODED_LEN;

            const MAX_ENCODED_LEN: usize = $unsigned::<{$bytes * 2}>::MAX_ENCODED_LEN;

            fn encoded_len(&self) -> usize {
              Packable::<$unsigned::<{$bytes * 2}>>::pack(*self.numer(), *self.denom()).encoded_len()
            }

            fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
              Packable::<$unsigned::<{$bytes * 2}>>::pack(*self.numer(), *self.denom()).encode(buf)
            }

            fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError>
            where
              Self: Sized,
            {
              let (bytes_read, merged) = $unsigned::< {$bytes * 2}>::decode(buf)?;
              let (numer, denom): ($base<$bytes>, $base<$bytes>) = Packable::<$unsigned::<{$bytes * 2}>>::unpack(merged);
              if denom.is_zero() {
                return Err(DecodeError::custom("denominator cannot be zero"));
              }
              Ok((bytes_read, Ratio::new_raw(numer, denom)))
            }
          }
        )*
      )*
    }
  };
}

// TODO: this can be implemented for all Uint<BITS, LIMBS> when feature(const_generics) is stable
impl_varint_for_ratio_bnum!(@unsigned
  BUintD8(1, 2, 3, 4, 5, 6, 7, 8, 12, 16, 32, 64, 128, 256, 512),
  BUintD16(1, 2, 3, 4, 5, 6, 7, 8, 12, 16, 32, 64, 128, 256),
  BUintD32(1, 2, 3, 4, 5, 6, 7, 8, 12, 16, 32, 64, 128),
  BUint(1, 2, 3, 4, 5, 6, 7, 8, 12, 16, 32, 64),
);

impl_varint_for_ratio_bnum!(@signed
  BIntD8 <=> BUintD8(1, 2, 3, 4, 5, 6, 7, 8, 12, 16, 32, 64, 128, 256, 512),
  BIntD16 <=> BUintD16(1, 2, 3, 4, 5, 6, 7, 8, 12, 16, 32, 64, 128, 256),
  BIntD32 <=> BUintD32(1, 2, 3, 4, 5, 6, 7, 8, 12, 16, 32, 64, 128),
  BInt <=> BUint(1, 2, 3, 4, 5, 6, 7, 8, 12, 16, 32, 64),
);
