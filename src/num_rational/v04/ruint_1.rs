use crate::{DecodeError, EncodeError, Varint, ruint_impl::Packable};
use ::ruint_1::Uint;
use num_rational_0_4::Ratio;

use core::num::NonZeroUsize;

#[cfg(not(feature = "bnum_0_13"))]
use ::ruint_1::aliases::U256;

// `MIN_ENCODED_LEN` is the encoded length of the shortest representable `Ratio`,
// `0/1`, which packs to `1 << 128` (unsigned) / `2 << 128` (signed, denominator
// `1` zigzags to `2`) — not the merged `U256`'s one-byte minimum. `ruint`'s pack
// and `encoded_len` are not `const`, so the length is computed directly as
// `ceil((128 + off) / 7)`.
#[cfg(not(feature = "bnum_0_13"))]
impl_varint_for_ratio!(@inner
  u::128(U256) => match NonZeroUsize::new((128usize + 1).div_ceil(7)) {
    Some(v) => v,
    None => unreachable!(),
  },
  i::128(U256) => match NonZeroUsize::new((128usize + 2).div_ceil(7)) {
    Some(v) => v,
    None => unreachable!(),
  },
);

macro_rules! impl_varint_for_ratio_ruint {
  ($($bits:literal),+$(,)?) => {
    paste::paste! {
      $(
        impl Varint for Ratio<Uint<$bits, { $bits / 64 } >> {
          // Shortest representable `Ratio` is `0/1`, which packs to `1 << BITS`
          // (numerator low, denominator high); its encoded length is
          // `ceil((BITS + 1) / 7)`, not the merged integer's one-byte minimum.
          const MIN_ENCODED_LEN: NonZeroUsize =
            match NonZeroUsize::new(($bits as usize + 1).div_ceil(7)) {
              Some(v) => v,
              None => unreachable!(),
            };

          const MAX_ENCODED_LEN: NonZeroUsize = Uint::<{$bits * 2}, {($bits * 2) / 64}>::MAX_ENCODED_LEN;

          fn encoded_len(&self) -> NonZeroUsize {
            Packable::<Uint::<{$bits * 2}, {($bits * 2) / 64}>>::pack(*self.numer(), *self.denom()).encoded_len()
          }

          fn encode(&self, buf: &mut [u8]) -> Result<NonZeroUsize, EncodeError> {
            Packable::<Uint::<{$bits * 2}, {($bits * 2) / 64}>>::pack(*self.numer(), *self.denom()).encode(buf)
          }

          fn decode(buf: &[u8]) -> Result<(NonZeroUsize, Self), DecodeError>
          where
            Self: Sized,
          {
            let (bytes_read, merged) = Uint::< { $bits * 2 }, {($bits * 2) / 64}>::decode(buf)?;
            let (numer, denom): (Uint<$bits, { $bits / 64 } >, Uint<$bits, { $bits / 64 } >) = Packable::<Uint::<{$bits * 2}, {($bits * 2) / 64}>>::unpack(merged);
            if denom.is_zero() {
              return Err(DecodeError::other("denominator cannot be zero"));
            }
            Ok((bytes_read, Ratio::new_raw(numer, denom)))
          }
        }
      )*
    }
  };
}

// TODO: this can be implemented for all Uint<BITS, LIMBS> when feature(const_generics) is stable
impl_varint_for_ratio_ruint!(64, 128, 192, 256, 320, 384, 448, 512, 768, 1024, 2048, 4096,);
