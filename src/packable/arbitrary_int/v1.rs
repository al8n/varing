use super::super::Packable;
use arbitrary_int_1::*;

macro_rules! impl_packable {
  ($($storage:literal), +$(,)?) => {
    paste::paste! {
      $(
        #[doc = "Packs `UInt<u" $storage ", LBITS>` and `UInt<u" $storage ", RBITS>` into a single `UInt<u" $storage ", PBITS>`."]
        #[inline]
        pub const fn [<pack_uint_d $storage>]<const LBITS: usize, const RBITS: usize, const PBITS: usize>(a: UInt<[<u $storage>], LBITS>, b: UInt<[<u $storage>], RBITS>) -> UInt<[<u $storage>], PBITS> {
          assert!(
            LBITS + RBITS <= PBITS,
            "The sum of LBITS and RBITS must be less or equal to the PBITS"
          );

          if LBITS > RBITS {
            let small = b.value();
            let large = a.value();
            UInt::<[<u $storage>], PBITS>::new((large << LBITS) | small)
          } else {
            let small = a.value();
            let large = b.value();
            UInt::<[<u $storage>], PBITS>::new((large << LBITS) | small)
          }
        }

        #[doc = "Unpacks `UInt<u" $storage ", PBITS>` into `UInt<u" $storage ", LBITS>` and `UInt<u" $storage ", RBITS>`."]
        #[inline]
        pub const fn [<unpack_uint_d $storage>]<const LBITS: usize, const RBITS: usize, const PBITS: usize>(packed: UInt<[<u $storage>], PBITS>) -> (UInt<[<u $storage>], LBITS>, UInt<[<u $storage>], RBITS>) {
          assert!(
            LBITS + RBITS <= PBITS,
            "The sum of LBITS and RBITS must be less or equal to the PBITS"
          );

          let packed = packed.value();

          if LBITS > RBITS {
            let small = packed & UInt::<[<u $storage>], RBITS>::MASK;
            let large = packed >> RBITS;

            let lhs = UInt::<[<u $storage>], LBITS>::new(small);
            let rhs = UInt::<[<u $storage>], RBITS>::new(large);
            (lhs, rhs)
          } else {
            let small = packed & UInt::<[<u $storage>], LBITS>::MASK;
            let large = packed >> LBITS;

            let lhs = UInt::<[<u $storage>], LBITS>::new(large);
            let rhs = UInt::<[<u $storage>], RBITS>::new(small);
            (lhs, rhs)
          }
        }

        impl<const LBITS: usize, const RBITS: usize, const PBITS: usize>
          Packable<UInt<[<u $storage>], RBITS>, UInt<[<u $storage>], PBITS>> for UInt<[<u $storage>], LBITS>
        {
          #[inline]
          fn pack(&self, rhs: &UInt<[<u $storage>], RBITS>) -> UInt<[<u $storage>], PBITS> {
            [< pack_uint_d $storage >]::<LBITS, RBITS, PBITS>(*self, *rhs)
          }

          #[inline]
          fn unpack(packed: UInt<[<u $storage>], PBITS>) -> (UInt<[<u $storage>], LBITS>, UInt<[<u $storage>], RBITS>) {
            [< unpack_uint_d $storage >]::<LBITS, RBITS, PBITS>(packed)
          }
        }
      )*
    }
  };
}

impl_packable!(8, 16, 32, 64, 128);
