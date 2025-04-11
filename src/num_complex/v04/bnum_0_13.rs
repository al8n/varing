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
  (@unsigned $( ($base:ident($($bits:literal),+$(,)?)) ), +$(,)?) => {
    paste::paste! {
      $(
        $(
          impl Varint for Complex<$base<{ $bits / 8 }>> {
            const MIN_ENCODED_LEN: usize = $base::<{ ($bits) * 2 }>::MAX_ENCODED_LEN;

            const MAX_ENCODED_LEN: usize = $base::<{ ($bits) * 2 }>::MAX_ENCODED_LEN;

            fn encoded_len(&self) -> usize {
              Packable::<$base::<{ $bits / 8 }>, $base::<{ ($bits) * 2 }>>::pack(&self.re, &self.im).encoded_len()
            }

            fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
              Packable::<$base::<{ $bits / 8 }>, $base::<{ ($bits) * 2 }>>::pack(&self.re, &self.im).encode(buf)
            }

            fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError>
            where
              Self: Sized,
            {
              let (bytes_read, merged) = $base::< { ($bits) * 2 }>::decode(buf)?;
              let (re, im): ($base<{ $bits / 8 }>, $base<{ $bits / 8 }>) = Packable::<$base::<{ $bits / 8 }>, $base::<{ ($bits) * 2 }>>::unpack(merged);
              Ok((bytes_read, Complex { re, im }))
            }
          }
        )*
      )*
    }
  };
  (@signed $(  ($base:ident <=> $unsigned:ident($($bits:literal),+$(,)?)) ), +$(,)?) => {
    paste::paste! {
      $(
        $(
          impl Varint for Complex<$base<{ $bits / 8 }>> {
            const MIN_ENCODED_LEN: usize = $unsigned::<{($bits / 8) * 2}>::MAX_ENCODED_LEN;

            const MAX_ENCODED_LEN: usize = $unsigned::<{($bits / 8) * 2}>::MAX_ENCODED_LEN;

            fn encoded_len(&self) -> usize {
              Packable::<$base::<{ $bits / 8 }>, $unsigned::<{($bits / 8) * 2}>>::pack(&self.re, &self.im).encoded_len()
            }

            fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
              Packable::<$base::<{ $bits / 8 }>, $unsigned::<{($bits / 8) * 2}>>::pack(&self.re, &self.im).encode(buf)
            }

            fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError>
            where
              Self: Sized,
            {
              let (bytes_read, merged) = $unsigned::< {($bits / 8) * 2}>::decode(buf)?;
              let (re, im): ($base< { $bits / 8 } >, $base< { $bits / 8 }>) = Packable::<$base::<{ $bits / 8 }>, $unsigned::<{($bits / 8) * 2}>>::unpack(merged);
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
  (BUintD8(8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768)),
  (BUintD16(8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768)),
  (BUintD32(8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768)),
  (BUint(8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768)),
);

impl_varint_for_complex_bnum!(@signed
  (BIntD8 <=> BUintD8(8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768)),
  (BIntD16 <=> BUintD16(8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768)),
  (BIntD32 <=> BUintD32(8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768)),
  (BInt <=> BUint(8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768)),
);
