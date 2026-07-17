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
