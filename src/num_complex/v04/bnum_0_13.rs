use crate::{
  bnum::{decode_uint_d8, encode_uint_d8_to, encoded_uint_d8_len},
  packable::Packable,
  utils::{pack_i128, pack_u128, unpack_i128, unpack_u128},
  DecodeError, EncodeError, Varint,
};
use ::bnum_0_13::{BInt, BIntD16, BIntD32, BIntD8, BUint, BUintD16, BUintD32, BUintD8};

use num_complex_0_4::Complex;

type U256 = BUintD8<32>;

impl_varint_for_complex!(128(U256));

/// Returns the encoded length of the `Complex<u128>` value.
#[inline]
pub const fn encoded_complex_u128_len(val: &Complex<u128>) -> usize {
  encoded_uint_d8_len(&pack_u128(val.re, val.im))
}

/// Encodes the `Complex<u128>` value.
#[inline]
pub const fn encode_complex_u128_to(
  val: &Complex<u128>,
  buf: &mut [u8],
) -> Result<usize, EncodeError> {
  encode_uint_d8_to(pack_u128(val.re, val.im), buf)
}

/// Decodes the `Complex<u128>` from the given buffer
///
/// Returns the bytes read and the value.
#[inline]
pub const fn decode_complex_u128(buf: &[u8]) -> Result<(usize, Complex<u128>), DecodeError> {
  match decode_uint_d8::<32>(buf) {
    Ok((read, val)) => {
      let (re, im) = unpack_u128(val);
      Ok((read, Complex { re, im }))
    }
    Err(e) => Err(e),
  }
}

/// Returns the encoded length of the `Complex<i128>` value.
#[inline]
pub const fn encoded_complex_i128_len(val: &Complex<i128>) -> usize {
  encoded_uint_d8_len(&pack_i128(val.re, val.im))
}

/// Encodes the `Complex<i128>` value.
#[inline]
pub const fn encode_complex_i128_to(
  val: &Complex<i128>,
  buf: &mut [u8],
) -> Result<usize, EncodeError> {
  encode_uint_d8_to(pack_i128(val.re, val.im), buf)
}

/// Decodes the `Complex<i128>` from the given buffer
///
/// Returns the bytes read and the value.
#[inline]
pub const fn decode_complex_i128(buf: &[u8]) -> Result<(usize, Complex<i128>), DecodeError> {
  match decode_uint_d8::<32>(buf) {
    Ok((read, val)) => {
      let (re, im) = unpack_i128(val);
      Ok((read, Complex { re, im }))
    }
    Err(e) => Err(e),
  }
}

macro_rules! impl_varint_for_complex_bnum {
  (@unsigned $( ($base:ident($($bytes:literal),+$(,)?)) ), +$(,)?) => {
    paste::paste! {
      $(
        $(
          impl Varint for Complex<$base<{ $bytes }>> {
            const MIN_ENCODED_LEN: usize = $base::<{ ($bytes) * 2 }>::MAX_ENCODED_LEN;

            const MAX_ENCODED_LEN: usize = $base::<{ ($bytes) * 2 }>::MAX_ENCODED_LEN;

            fn encoded_len(&self) -> usize {
              Packable::<$base::<{ $bytes }>, $base::<{ ($bytes) * 2 }>>::pack(&self.re, &self.im).encoded_len()
            }

            fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
              Packable::<$base::<{ $bytes }>, $base::<{ ($bytes) * 2 }>>::pack(&self.re, &self.im).encode(buf)
            }

            fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError>
            where
              Self: Sized,
            {
              let (bytes_read, merged) = $base::< { ($bytes) * 2 }>::decode(buf)?;
              let (re, im): ($base<{ $bytes }>, $base<{ $bytes }>) = Packable::<$base::<{ $bytes }>, $base::<{ ($bytes) * 2 }>>::unpack(merged);
              Ok((bytes_read, Complex { re, im }))
            }
          }
        )*
      )*
    }
  };
  (@signed $(  ($base:ident <=> $unsigned:ident($($bytes:literal),+$(,)?)) ), +$(,)?) => {
    paste::paste! {
      $(
        $(
          impl Varint for Complex<$base<{ $bytes }>> {
            const MIN_ENCODED_LEN: usize = $unsigned::<{($bytes) * 2}>::MAX_ENCODED_LEN;

            const MAX_ENCODED_LEN: usize = $unsigned::<{($bytes) * 2}>::MAX_ENCODED_LEN;

            fn encoded_len(&self) -> usize {
              Packable::<$base::<{ $bytes }>, $unsigned::<{($bytes) * 2}>>::pack(&self.re, &self.im).encoded_len()
            }

            fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
              Packable::<$base::<{ $bytes }>, $unsigned::<{($bytes) * 2}>>::pack(&self.re, &self.im).encode(buf)
            }

            fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError>
            where
              Self: Sized,
            {
              let (bytes_read, merged) = $unsigned::< {($bytes) * 2}>::decode(buf)?;
              let (re, im): ($base< { $bytes } >, $base< { $bytes }>) = Packable::<$base::<{ $bytes }>, $unsigned::<{($bytes) * 2}>>::unpack(merged);
              Ok((bytes_read, Complex { re, im }))
            }
          }
        )*
      )*
    }
  };
}

// TODO: this can be implemented for all Uint<BITS, LIMBS> when feature(const_generics) is stable
impl_varint_for_complex_bnum!(@unsigned
  (BUintD8(1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096)),
  (BUintD16(1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096)),
  (BUintD32(1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096)),
  (BUint(1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096)),
);

impl_varint_for_complex_bnum!(@signed
  (BIntD8 <=> BUintD8(1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096)),
  (BIntD16 <=> BUintD16(1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096)),
  (BIntD32 <=> BUintD32(1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096)),
  (BInt <=> BUint(1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096)),
);
