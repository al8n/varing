use crate::{bnum::Packable, DecodeError, EncodeError, Varint};
use ::bnum_0_13::{BInt, BIntD16, BIntD32, BIntD8, BUint, BUintD16, BUintD32, BUintD8};
use num_rational_0_4::Ratio;

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
