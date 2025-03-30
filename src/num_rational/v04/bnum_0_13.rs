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
pub const fn encode_ratio_u128_to(val: &Ratio<u128>, buf: &mut [u8]) -> Result<usize, EncodeError> {
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
pub const fn encode_ratio_i128_to(val: &Ratio<i128>, buf: &mut [u8]) -> Result<usize, EncodeError> {
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
  (@unsigned $( $storage:literal($base:ident($($bits:literal),+$(,)?)) ), +$(,)?) => {
    paste::paste! {
      $(
        $(
          impl Varint for Ratio<$base<{ $bits / 8 }>> {
            const MIN_ENCODED_LEN: usize = $base::<{($bits / 8) * 2}>::MAX_ENCODED_LEN;

            const MAX_ENCODED_LEN: usize = $base::<{($bits / 8) * 2}>::MAX_ENCODED_LEN;

            fn encoded_len(&self) -> usize {
              Packable::<$base::<{($bits / 8) * 2}>>::pack(*self.numer(), *self.denom()).encoded_len()
            }

            fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
              Packable::<$base::<{($bits / 8) * 2}>>::pack(*self.numer(), *self.denom()).encode(buf)
            }

            fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError>
            where
              Self: Sized,
            {
              let (bytes_read, merged) = $base::< {($bits / 8) * 2} >::decode(buf)?;
              let (numer, denom): ($base<{ $bits / 8 }>, $base<{ $bits / 8 }>) = Packable::<$base::<{($bits / 8) * 2}>>::unpack(merged);
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
  (@signed $( $storage:literal ($base:ident <=> $unsigned:ident($($bits:literal),+$(,)?)) ), +$(,)?) => {
    paste::paste! {
      $(
        $(
          impl Varint for Ratio<$base<{ $bits / 8 }>> {
            const MIN_ENCODED_LEN: usize = $unsigned::<{($bits / 8) * 2}>::MAX_ENCODED_LEN;

            const MAX_ENCODED_LEN: usize = $unsigned::<{($bits / 8) * 2}>::MAX_ENCODED_LEN;

            fn encoded_len(&self) -> usize {
              Packable::<$unsigned::<{($bits / 8) * 2}>>::pack(*self.numer(), *self.denom()).encoded_len()
            }

            fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
              Packable::<$unsigned::<{($bits / 8) * 2}>>::pack(*self.numer(), *self.denom()).encode(buf)
            }

            fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError>
            where
              Self: Sized,
            {
              let (bytes_read, merged) = $unsigned::< {($bits / 8) * 2}>::decode(buf)?;
              let (numer, denom): ($base<{ $bits / 8 }>, $base<{ $bits / 8 }>) = Packable::<$unsigned::<{($bits / 8) * 2}>>::unpack(merged);
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
  8(BUintD8(8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768)),
  16(BUintD16(8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768)),
  32(BUintD32(8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768)),
  64(BUint(8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768)),
);

impl_varint_for_ratio_bnum!(@signed
  8(BIntD8 <=> BUintD8(8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768)),
  16(BIntD16 <=> BUintD16(8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768)),
  32(BIntD32 <=> BUintD32(8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768)),
  64(BInt <=> BUint(8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768)),
);
