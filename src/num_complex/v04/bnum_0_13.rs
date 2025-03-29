use crate::{bnum::Packable, DecodeError, EncodeError, Varint};
use ::bnum_0_13::{BInt, BIntD16, BIntD32, BIntD8, BUint, BUintD16, BUintD32, BUintD8};
use num_complex_0_4::Complex;

macro_rules! impl_varint_for_complex_bnum {
  (@unsigned $($base:ident($($bytes:literal),+$(,)?)), +$(,)?) => {
    paste::paste! {
      $(
        $(
          impl Varint for Complex<$base<$bytes>> {
            const MIN_ENCODED_LEN: usize = $base::<{$bytes * 2}>::MAX_ENCODED_LEN;

            const MAX_ENCODED_LEN: usize = $base::<{$bytes * 2}>::MAX_ENCODED_LEN;

            fn encoded_len(&self) -> usize {
              Packable::<$base::<{$bytes * 2}>>::pack(self.re, self.im).encoded_len()
            }

            fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
              Packable::<$base::<{$bytes * 2}>>::pack(self.re, self.im).encode(buf)
            }

            fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError>
            where
              Self: Sized,
            {
              let (bytes_read, merged) = $base::< {$bytes * 2}>::decode(buf)?;
              let (re, im): ($base<$bytes>, $base<$bytes>) = Packable::<$base::<{$bytes * 2}>>::unpack(merged);
              Ok((bytes_read, Complex { re, im }))
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
          impl Varint for Complex<$base<$bytes>> {
            const MIN_ENCODED_LEN: usize = $unsigned::<{$bytes * 2}>::MAX_ENCODED_LEN;

            const MAX_ENCODED_LEN: usize = $unsigned::<{$bytes * 2}>::MAX_ENCODED_LEN;

            fn encoded_len(&self) -> usize {
              Packable::<$unsigned::<{$bytes * 2}>>::pack(self.re, self.im).encoded_len()
            }

            fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
              Packable::<$unsigned::<{$bytes * 2}>>::pack(self.re, self.im).encode(buf)
            }

            fn decode(buf: &[u8]) -> Result<(usize, Self), DecodeError>
            where
              Self: Sized,
            {
              let (bytes_read, merged) = $unsigned::< {$bytes * 2}>::decode(buf)?;
              let (re, im): ($base<$bytes>, $base<$bytes>) = Packable::<$unsigned::<{$bytes * 2}>>::unpack(merged);
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
  BUintD8(1, 2, 3, 4, 5, 6, 7, 8, 12, 16, 32, 64, 128, 256, 512),
  BUintD16(1, 2, 3, 4, 5, 6, 7, 8, 12, 16, 32, 64, 128, 256),
  BUintD32(1, 2, 3, 4, 5, 6, 7, 8, 12, 16, 32, 64, 128),
  BUint(1, 2, 3, 4, 5, 6, 7, 8, 12, 16, 32, 64),
);

impl_varint_for_complex_bnum!(@signed
  BIntD8 <=> BUintD8(1, 2, 3, 4, 5, 6, 7, 8, 12, 16, 32, 64, 128, 256, 512),
  BIntD16 <=> BUintD16(1, 2, 3, 4, 5, 6, 7, 8, 12, 16, 32, 64, 128, 256),
  BIntD32 <=> BUintD32(1, 2, 3, 4, 5, 6, 7, 8, 12, 16, 32, 64, 128),
  BInt <=> BUint(1, 2, 3, 4, 5, 6, 7, 8, 12, 16, 32, 64),
);
