use core::ops::{BitAnd, BitOr, Shl, Shr};
use ruint_1::Uint;

use crate::packable::Packable;

/// Packs `Uint<LBITS, LLIMBS>` and `Uint<RBITS, RLIMBS>` into `Uint<PBITS, PLIMBS>`.
fn pack_uint<
  const LBITS: usize,
  const LLIMBS: usize,
  const RBITS: usize,
  const RLIMBS: usize,
  const PBITS: usize,
  const PLIMBS: usize,
>(
  lhs: &Uint<LBITS, LLIMBS>,
  rhs: &Uint<RBITS, RLIMBS>,
) -> Uint<PBITS, PLIMBS> {
  if LBITS == 0 && RBITS == 0 {
    return Uint::<PBITS, PLIMBS>::ZERO;
  }

  assert!(
    LBITS + RBITS <= PBITS,
    "The sum of LBITS and RBITS must be equal to the PBITS"
  );
  assert!(
    LLIMBS + RLIMBS <= PLIMBS,
    "The sum of LLIMBS and RLIMBS must be equal to the PLIMBS"
  );

  let (small_bits, small, large) = if LBITS > RBITS {
    let small = Uint::<PBITS, PLIMBS>::from_limbs_slice(rhs.as_limbs());
    let large = Uint::<PBITS, PLIMBS>::from_limbs_slice(lhs.as_limbs());
    (RBITS, small, large)
  } else {
    let small = Uint::<PBITS, PLIMBS>::from_limbs_slice(lhs.as_limbs());
    let large = Uint::<PBITS, PLIMBS>::from_limbs_slice(rhs.as_limbs());
    (LBITS, small, large)
  };

  large.shl(small_bits as u32).bitor(small)
}

/// Unpacks `Uint<PBITS, PLIMBS>` into `Uint<LBITS, LLIMBS>` and `Uint<RBITS, RLIMBS>`.
fn unpack_uint<
  const LBITS: usize,
  const LLIMBS: usize,
  const RBITS: usize,
  const RLIMBS: usize,
  const PBITS: usize,
  const PLIMBS: usize,
>(
  packed: &Uint<PBITS, PLIMBS>,
) -> (Uint<LBITS, LLIMBS>, Uint<RBITS, RLIMBS>) {
  if LBITS == 0 && RBITS == 0 {
    return (Uint::<LBITS, LLIMBS>::ZERO, Uint::<RBITS, RLIMBS>::ZERO);
  }

  assert!(
    LBITS + RBITS <= PBITS,
    "The sum of LBITS and RBITS must be equal to the PBITS"
  );
  assert!(
    LLIMBS + RLIMBS <= PLIMBS,
    "The sum of LLIMBS and RLIMBS must be equal to the PLIMBS"
  );

  if LBITS > RBITS {
    let small = packed.bitand(Uint::<PBITS, PLIMBS>::from_limbs_slice(
      Uint::<RBITS, RLIMBS>::MAX.as_limbs(),
    ));
    let large = packed.shr(RLIMBS as u32);

    let lhs = Uint::<LBITS, LLIMBS>::from_limbs_slice(large.as_limbs());
    let rhs = Uint::<RBITS, RLIMBS>::from_limbs_slice(small.as_limbs());
    (lhs, rhs)
  } else {
    let small = packed.bitand(Uint::<PBITS, PLIMBS>::from_limbs_slice(
      Uint::<LBITS, LLIMBS>::MAX.as_limbs(),
    ));
    let large = packed.shr(LLIMBS as u32);

    let lhs = Uint::<LBITS, LLIMBS>::from_limbs_slice(small.as_limbs());
    let rhs = Uint::<RBITS, RLIMBS>::from_limbs_slice(large.as_limbs());
    (lhs, rhs)
  }
}

impl<
    const LBITS: usize,
    const LLIMBS: usize,
    const RBITS: usize,
    const RLIMBS: usize,
    const PBITS: usize,
    const PLIMBS: usize,
  > Packable<Uint<LBITS, LLIMBS>, Uint<RBITS, RLIMBS>> for Uint<PBITS, PLIMBS>
{
  fn pack(&self, rhs: &Uint<LBITS, LLIMBS>) -> Uint<RBITS, RLIMBS> {
    pack_uint(self, rhs)
  }

  fn unpack(packed: Uint<RBITS, RLIMBS>) -> (Self, Uint<LBITS, LLIMBS>)
  where
    Self: Sized,
    Uint<LBITS, LLIMBS>: Sized,
  {
    unpack_uint(&packed)
  }
}
