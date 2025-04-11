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

  assert_consts::<LBITS, LLIMBS, RBITS, RLIMBS, PBITS, PLIMBS>();

  // Decide which value goes to the high bits and which to the low bits
  let (high_bits, high_value, low_value, low_bits) = if LBITS > RBITS {
    // lhs goes to high bits, rhs goes to low bits
    let high = Uint::<PBITS, PLIMBS>::from_limbs_slice(lhs.as_limbs());
    let low = Uint::<PBITS, PLIMBS>::from_limbs_slice(rhs.as_limbs());
    (LBITS, high, low, RBITS)
  } else {
    // rhs goes to high bits, lhs goes to low bits
    let high = Uint::<PBITS, PLIMBS>::from_limbs_slice(rhs.as_limbs());
    let low = Uint::<PBITS, PLIMBS>::from_limbs_slice(lhs.as_limbs());
    (RBITS, high, low, LBITS)
  };

  // Create mask for the low value to ensure it doesn't exceed its bit width
  let low_mask = if low_bits == PBITS {
    Uint::<PBITS, PLIMBS>::MAX
  } else {
    (Uint::<PBITS, PLIMBS>::from(1u64) << low_bits) - Uint::<PBITS, PLIMBS>::from(1u64)
  };

  // Apply mask to low value
  let masked_low = low_value.bitand(&low_mask);

  // Create mask for the high value
  let high_mask = if high_bits == PBITS {
    Uint::<PBITS, PLIMBS>::MAX
  } else {
    (Uint::<PBITS, PLIMBS>::from(1u64) << high_bits) - Uint::<PBITS, PLIMBS>::from(1u64)
  };

  // Apply mask to high value, shift it to proper position, and combine with low value
  let masked_high = high_value.bitand(&high_mask);

  masked_high.shl(low_bits as u32).bitor(&masked_low)
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

  assert_consts::<LBITS, LLIMBS, RBITS, RLIMBS, PBITS, PLIMBS>();

  // Determine which value was placed in high bits vs low bits
  let low_bits = if LBITS > RBITS { RBITS } else { LBITS };

  // Create masks for extracting each value
  let low_mask = if low_bits == PBITS {
    Uint::<PBITS, PLIMBS>::MAX
  } else {
    (Uint::<PBITS, PLIMBS>::from(1u64) << low_bits) - Uint::<PBITS, PLIMBS>::from(1u64)
  };

  // Extract the low bits part
  let low_value = packed.bitand(&low_mask);

  // Extract the high bits part
  let high_value = packed.shr(low_bits as u32);

  // Create properly sized results based on whether lhs or rhs was the larger value
  if LBITS > RBITS {
    // lhs was in high bits, rhs was in low bits
    let lhs = Uint::<LBITS, LLIMBS>::from_limbs_slice(&high_value.as_limbs()[..LLIMBS]);
    let rhs = Uint::<RBITS, RLIMBS>::from_limbs_slice(&low_value.as_limbs()[..RLIMBS]);
    (lhs, rhs)
  } else {
    // rhs was in high bits, lhs was in low bits
    let lhs = Uint::<LBITS, LLIMBS>::from_limbs_slice(&low_value.as_limbs()[..LLIMBS]);
    let rhs = Uint::<RBITS, RLIMBS>::from_limbs_slice(&high_value.as_limbs()[..RLIMBS]);
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

const fn assert_consts<
  const LBITS: usize,
  const LLIMBS: usize,
  const RBITS: usize,
  const RLIMBS: usize,
  const PBITS: usize,
  const PLIMBS: usize,
>() {
  // Check if there's enough space in the packed value for both integers
  assert!(
    LBITS + RBITS <= PBITS,
    "The sum of LBITS and RBITS must be less than or equal to PBITS"
  );
  assert!(
    LLIMBS + RLIMBS <= PLIMBS,
    "The sum of LLIMBS and RLIMBS must be less than or equal to PLIMBS"
  );
}

#[cfg(test)]
mod tests {
  use super::*;

  fn roundtrip<
    const LBITS: usize,
    const LLIMBS: usize,
    const RBITS: usize,
    const RLIMBS: usize,
    const PBITS: usize,
    const PLIMBS: usize,
  >(
    lhs: &Uint<LBITS, LLIMBS>,
    rhs: &Uint<RBITS, RLIMBS>,
  ) -> bool {
    let packed = lhs.pack(rhs);
    let (lhs_unpacked, rhs_unpacked) =
      <Uint<LBITS, LLIMBS> as Packable<Uint<RBITS, RLIMBS>, Uint<PBITS, PLIMBS>>>::unpack(packed);
    lhs == &lhs_unpacked && rhs == &rhs_unpacked
  }

  macro_rules! fuzzy_packable {
    ($(($bits:literal, $limbs:literal)),+$(,)?) => {
      paste::paste! {
        quickcheck::quickcheck! {
          $(
            fn [<fuzzy_u $bits:snake>](a: [<U $bits>], b: [<U $bits>]) -> bool {
              roundtrip::<$bits, $limbs, $bits, $limbs, {$bits * 2}, {$limbs * 2}>(&a, &b)
            }
          )*
        }
      }
    };
  }

  use ruint_1::aliases::*;

  fuzzy_packable!(
    (64, 1),
    (128, 2),
    (256, 4),
    (512, 8),
    (1024, 16),
    (2048, 32),
  );

  #[test]
  fn zero() {
    let output = pack_uint(&Uint::<0, 0>::ZERO, &Uint::<0, 0>::ZERO);
    assert_eq!(output, Uint::<0, 0>::ZERO);
    let (lhs, rhs) = unpack_uint::<0, 0, 0, 0, 0, 0>(&output);
    assert_eq!(lhs, Uint::<0, 0>::ZERO);
    assert_eq!(rhs, Uint::<0, 0>::ZERO);
  }

  #[test]
  #[should_panic]
  fn assert_consts_panic() {
    assert_consts::<1, 1, 1, 1, 2, 0>();
  }

  #[test]
  #[should_panic]
  fn assert_consts_panic_2() {
    assert_consts::<1, 1, 1, 1, 0, 2>();
  }
}
