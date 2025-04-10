use super::super::Packable;
use bnum_0_13::*;

macro_rules! impl_packable {
  ($($suffix:ident ($storage:literal)), +$(,)?) => {
    paste::paste! {
      $(
        #[doc = "Zigzag encode `B" $suffix "<N>` value"]
        #[inline]
        pub const fn [< zigzag_encode_ $suffix:snake >]<const N: usize>(value: &[< B $suffix:camel >]<N>) -> [< BU $suffix >]<N> {
          if N == 0 {
            return [< BU $suffix >]::<N>::ZERO;
          }

          let bits = match [< B $suffix:camel >]::<N>::BITS.checked_sub(1) {
            Some(val) => val,
            None => 0,
          };

          value.shl(1).bitxor(value.shr(bits as u32)).to_bits()
        }

        #[doc = "Zigzag decode `B" $suffix:camel "<N>` value"]
        #[inline]
        pub const fn [< zigzag_decode_ $suffix:snake >]<const N: usize>(value: &[< BU $suffix >]<N>) -> [< B $suffix:camel >]<N> {
          if N == 0 {
            return [< B $suffix:camel >]::<N>::ZERO;
          }

          let a = [< B $suffix:camel >]::<N>::from_bits(value.shr(1));
          let b = [< B $suffix:camel >]::<N>::from_bits(value.bitand([< BU $suffix >]::<N>::from_digit(1))).neg();
          a.bitxor(b)
        }

        #[doc = "Packs `BU`" $suffix "<L>` and `BU" $suffix "<R>` into a single `BU" $suffix "<P>` value."]
        pub const fn [< pack_u $suffix:snake>]<const L: usize, const R: usize, const P: usize>(a: &[< BU $suffix >]<L>, b: &[< BU $suffix >]<R>) -> [< BU $suffix >]<P> {
          assert!(L + R <= P, "L + R must be less than or equal to P");

          if L == 0 && R == 0 {
            return [< BU $suffix >]::<P>::ZERO;
          }

          let mut low = [0; P];
          let mut high = [0; P];

          // Determine which input has smaller size (in bits)
          let (high_bits, high_value, low_value, low_bits) = if L > R {
            unsafe { core::ptr::copy(b.digits().as_ptr(), low.as_mut_ptr(), L) };
            unsafe { core::ptr::copy(a.digits().as_ptr(), high.as_mut_ptr(), R) };

            let low = [< BU $suffix >]::<P>::from_digits(low);
            let high = [< BU $suffix >]::<P>::from_digits(high);

            (L * $storage, high, low, R * $storage)
          } else {
            unsafe { core::ptr::copy(a.digits().as_ptr(), low.as_mut_ptr(), R) };
            unsafe { core::ptr::copy(b.digits().as_ptr(), high.as_mut_ptr(), L) };

            let low = [< BU $suffix >]::<P>::from_digits(low);
            let high = [< BU $suffix >]::<P>::from_digits(high);

            (R * $storage, high, low, L * $storage)
          };

          // Create mask for the low value to ensure it doesn't exceed its bit width
          let low_mask = if low_bits == { P * $storage } {
            [< BU $suffix >]::<P>::MAX
          } else {
            [< BU $suffix >]::<P>::ONE.shl(low_bits as u32).sub([< BU $suffix >]::<P>::ONE)
          };

          // Apply mask to low value
          let masked_low = low_value.bitand(low_mask);

          // Create mask for the high value
          let high_mask = if high_bits == { P * $storage } {
            [< BU $suffix >]::<P>::MAX
          } else {
            [< BU $suffix >]::<P>::ONE.shl(high_bits as u32).sub([< BU $suffix >]::<P>::ONE)
          };

          // Apply mask to high value, shift it to proper position, and combine with low value
          let masked_high = high_value.bitand(high_mask);

          masked_high.shl(low_bits as u32).bitor(masked_low)
        }

        #[doc = "Unpacks `BU`" $suffix "<P>` into `BU" $suffix "<L>` and `BU" $suffix "<R>`"]
        pub const fn [< unpack_u $suffix:snake >]<const L: usize, const R: usize, const P: usize>(packed: [< BU $suffix >]<P>) -> ([< BU $suffix >]<L>, [< BU $suffix >]<R>) {
          assert!(L + R <= P, "L + R must be less than or equal to P");

          if L == 0 && R == 0 {
            return ([< BU $suffix >]::<L>::ZERO, [< BU $suffix >]::<R>::ZERO);
          }

          // Determine which value was placed in high bits vs low bits
          let low_bits = if L > R {
            R * $storage
          } else {
            L * $storage
          };

          // Create masks for extracting each value
          let low_mask = if low_bits == { P * $storage } {
            [< BU $suffix >]::<P>::MAX
          } else {
            [< BU $suffix >]::<P>::ONE.shl(low_bits as u32).sub([< BU $suffix >]::<P>::ONE)
          };

          // Extract the low bits part
          let low_value = packed.bitand(low_mask);

          // Extract the high bits part
          let high_value = packed.shr(low_bits as u32);

          if L > R {
            let mut lhs = [0; L];
            unsafe {
              core::ptr::copy(high_value.digits().as_ptr(), lhs.as_mut_ptr(), L);
            }
            let lhs = [< BU $suffix >]::<L>::from_digits(lhs);
            let mut rhs = [0; R];
            unsafe {
              core::ptr::copy(low_value.digits().as_ptr(), rhs.as_mut_ptr(), R);
            }
            let rhs = [< BU $suffix >]::<R>::from_digits(rhs);
            (lhs, rhs)
          } else {
            let mut lhs = [0; L];
            unsafe {
              core::ptr::copy(low_value.digits().as_ptr(), lhs.as_mut_ptr(), L);
            }
            let lhs = [< BU $suffix >]::<L>::from_digits(lhs);
            let mut rhs = [0; R];
            unsafe {
              core::ptr::copy(high_value.digits().as_ptr(), rhs.as_mut_ptr(), R);
            }
            let rhs = [< BU $suffix >]::<R>::from_digits(rhs);
            (lhs, rhs)
          }
        }

        #[doc = "Packs `B`" $suffix:camel "<L>` and `B" $suffix:camel "<R>` into a single `BU" $suffix "<P>` value."]
        #[inline]
        pub const fn [< pack_ $suffix:snake>]<const L: usize, const R: usize, const P: usize>(a: &[< B $suffix:camel >]<L>, b: &[< B $suffix:camel >]<R>) -> [< BU $suffix >]<P> {
          if L == 0 && R == 0 {
            return [< BU $suffix >]::<P>::ZERO;
          }

          let a = [< zigzag_encode_ $suffix:snake >](&a);
          let b = [< zigzag_encode_ $suffix:snake >](&b);
          [< pack_u $suffix:snake >](&a, &b)
        }

        #[doc = "Unpacks `BU`" $suffix "<P>` into `B" $suffix:camel "<L>` and `B" $suffix:camel "<R>`"]
        #[inline]
        pub const fn [< unpack_ $suffix:snake >]<const L: usize, const R: usize, const P: usize>(packed: [< BU $suffix >]<P>) -> ([< B $suffix:camel >]<L>, [< B $suffix:camel >]<R>) {
          if L == 0 && R == 0 {
            return ([< B $suffix:camel >]::<L>::ZERO, [< B $suffix:camel >]::<R>::ZERO);
          }

          let (a, b) = [< unpack_u $suffix:snake >](packed);
          let a = [< zigzag_decode_ $suffix:snake >](&a);
          let b = [< zigzag_decode_ $suffix:snake >](&b);
          (a, b)
        }

        #[doc = "Packs `B`" $suffix:camel "<L>` and `BU" $suffix "<R>` into a single `BU" $suffix "<P>` value."]
        #[inline]
        pub const fn [< pack_ $suffix:snake _u $suffix:snake>]<const L: usize, const R: usize, const P: usize>(a: &[< B $suffix:camel >]<L>, b: &[< BU $suffix >]<R>) -> [< BU $suffix >]<P> {
          if L == 0 && R == 0 {
            return [< BU $suffix >]::<P>::ZERO;
          }

          let a = [< zigzag_encode_ $suffix:snake >](&a);
          [< pack_u $suffix:snake >](&a, b)
        }

        #[doc = "Unpacks `BU`" $suffix "<P>` into `B" $suffix:camel "<L>` and `BU" $suffix "<R>`"]
        #[inline]
        pub const fn [< unpack_ $suffix:snake _u $suffix:snake >]<const L: usize, const R: usize, const P: usize>(packed: [< BU $suffix >]<P>) -> ([< B $suffix:camel >]<L>, [< BU $suffix >]<R>) {
          if L == 0 && R == 0 {
            return ([< B $suffix:camel >]::<L>::ZERO, [< BU $suffix >]::<R>::ZERO);
          }

          let (a, b) = [< unpack_u $suffix:snake >](packed);
          let a = [< zigzag_decode_ $suffix:snake >](&a);
          (a, b)
        }

        #[doc = "Packs `B`" $suffix:camel "<L>` and `BU" $suffix "<R>` into a single `BU" $suffix "<P>` value."]
        #[inline]
        pub const fn [< pack_u $suffix:snake _ $suffix:snake>]<const L: usize, const R: usize, const P: usize>(a: &[< BU $suffix >]<L>, b: &[< B $suffix:camel >]<R>) -> [< BU $suffix >]<P> {
          if L == 0 && R == 0 {
            return [< BU $suffix >]::<P>::ZERO;
          }

          let b = [< zigzag_encode_ $suffix:snake >](&b);
          [< pack_u $suffix:snake >](a, &b)
        }

        #[doc = "Unpacks `BU`" $suffix "<P>` into `BU" $suffix "<L>` and `B" $suffix:camel "<R>`"]
        #[inline]
        pub const fn [< unpack_u $suffix:snake _ $suffix:snake >]<const L: usize, const R: usize, const P: usize>(packed: [< BU $suffix >]<P>) -> ([< BU $suffix >]<L>, [< B $suffix:camel >]<R>) {
          if L == 0 && R == 0 {
            return ([< BU $suffix >]::<L>::ZERO, [< B $suffix:camel >]::<R>::ZERO);
          }

          let (a, b) = [< unpack_u $suffix:snake >](packed);
          let b = [< zigzag_decode_ $suffix:snake >](&b);
          (a, b)
        }

        impl<const L: usize, const R: usize, const P: usize> Packable<[< BU $suffix>]<R>, [< BU $suffix>]<P>> for [< BU $suffix>]<L> {
          fn pack(&self, rhs: &[< BU $suffix>]<R>) -> [< BU $suffix>]<P> {
            [< pack_u $suffix:snake >](self, rhs)
          }

          fn unpack(packed: [< BU $suffix>]<P>) -> (Self, [< BU $suffix>]<R>)
          where
            Self: Sized,
            BUint<R>: Sized
          {
            [< unpack_u $suffix:snake >](packed)
          }
        }

        impl<const L: usize, const R: usize, const P: usize> Packable<[< B $suffix:camel>]<R>, [< BU $suffix>]<P>> for [< B $suffix:camel >]<L> {
          fn pack(&self, rhs: &[< B $suffix:camel >]<R>) -> [< BU $suffix>]<P> {
            [< pack_ $suffix:snake >](self, rhs)
          }

          fn unpack(packed: [< BU $suffix>]<P>) -> (Self, [< B $suffix:camel>]<R>)
          where
            Self: Sized,
            BUint<R>: Sized
          {
            [< unpack_ $suffix:snake >](packed)
          }
        }

        impl<const L: usize, const R: usize, const P: usize> Packable<[< B $suffix:camel>]<R>, [< BU $suffix>]<P>> for [< BU $suffix>]<L> {
          fn pack(&self, rhs: &[< B $suffix:camel>]<R>) -> [< BU $suffix>]<P> {
            [< pack_u $suffix:snake _ $suffix:snake >](self, rhs)
          }

          fn unpack(packed: [< BU $suffix>]<P>) -> (Self, [< B $suffix:camel>]<R>)
          where
            Self: Sized,
            BUint<R>: Sized
          {
            [< unpack_u $suffix:snake _ $suffix:snake >](packed)
          }
        }

        impl<const L: usize, const R: usize, const P: usize> Packable<[< BU $suffix>]<R>, [< BU $suffix>]<P>> for [< B $suffix:camel>]<L> {
          fn pack(&self, rhs: &[< BU $suffix>]<R>) -> [< BU $suffix>]<P> {
            [< pack_ $suffix:snake _u $suffix:snake >](self, rhs)
          }

          fn unpack(packed: [< BU $suffix>]<P>) -> (Self, [< BU $suffix>]<R>)
          where
            Self: Sized,
            BUint<R>: Sized
          {
            [< unpack_ $suffix:snake _u $suffix:snake >](packed)
          }
        }

        #[cfg(test)]
        fn [< roundtrip_u $suffix:snake _test>]<const L: usize, const R: usize, const P: usize>(lhs: [< BU $suffix>]<L>, rhs: [< BU $suffix>]<R>) -> bool {
          let packed = <[< BU $suffix>]<L> as Packable<[< BU $suffix>]<R>, [< BU $suffix>]<P>>>::pack(&lhs, &rhs);
          let (lhs2, rhs2) = <[< BU $suffix>]<L> as Packable<[< BU $suffix>]<R>, _>>::unpack(packed);
          lhs == lhs2 && rhs == rhs2
        }

        #[cfg(test)]
        fn [< roundtrip_ $suffix:snake _test>]<const L: usize, const R: usize, const P: usize>(lhs: [< B $suffix: camel>]<L>, rhs: [< B $suffix: camel>]<R>) -> bool {
          let packed = <[< B $suffix: camel>]<L> as Packable<[< B $suffix:camel>]<R>, [< BU $suffix>]<P>>>::pack(&lhs, &rhs);
          let (lhs2, rhs2) = <[< B $suffix: camel>]<L> as Packable<[< B $suffix: camel>]<R>, _>>::unpack(packed);
          lhs == lhs2 && rhs == rhs2
        }

        #[cfg(test)]
        fn [< roundtrip_u $suffix:snake _ $suffix:snake _test>]<const L: usize, const R: usize, const P: usize>(lhs: [< BU $suffix>]<L>, rhs: [< B $suffix:camel>]<R>) -> bool {
          let packed = <[< BU $suffix>]<L> as Packable<[< B $suffix:camel>]<R>, [< BU $suffix>]<P>>>::pack(&lhs, &rhs);
          let (lhs2, rhs2) = <[< BU $suffix>]<L> as Packable<[< B $suffix:camel>]<R>, _>>::unpack(packed);
          lhs == lhs2 && rhs == rhs2
        }

        #[cfg(test)]
        fn [< roundtrip_ $suffix:snake _u $suffix:snake _test>]<const L: usize, const R: usize, const P: usize>(lhs: [< B $suffix:camel>]<L>, rhs: [< BU $suffix>]<R>) -> bool {
          let packed = <[< B $suffix:camel>]<L> as Packable<[< BU $suffix>]<R>, [< BU $suffix>]<P>>>::pack(&lhs, &rhs);
          let (lhs2, rhs2) = <[< B $suffix:camel>]<L> as Packable<[< BU $suffix>]<R>, _>>::unpack(packed);
          lhs == lhs2 && rhs == rhs2
        }
      )*
    }
  };
}

impl_packable!(intD8(8), intD16(16), intD32(32), int(64),);

macro_rules! fuzzy_packable {
  ( $( $suffix:ident ($($bits: literal), +$(,)?) ), +$(,)? ) => {
    paste::paste! {
      $(
        $(
          #[cfg(test)]
          quickcheck::quickcheck! {
            fn [< fuzzy_u $suffix:snake _ $bits _roundtrip>](a: [< BU $suffix>]<{ $bits / 8 }>, b: [< BU $suffix>]<{ $bits / 8 }>) -> bool {
              [< roundtrip_u $suffix:snake _test>]::<{$bits / 8}, {$bits / 8}, { ($bits / 8) * 2}>(a, b)
            }

            fn [< fuzzy_ $suffix:snake _ $bits _roundtrip>](a: [< B $suffix:camel>]<{ $bits / 8 }>, b: [< B $suffix:camel>]<{ $bits / 8 }>) -> bool {
              [< roundtrip_ $suffix:snake _test>]::<{$bits / 8}, {$bits / 8}, { ($bits / 8) * 2}>(a, b)
            }

            fn [< fuzzy_u $suffix:snake _ $suffix:snake _ $bits _roundtrip>](a: [< BU $suffix>]<{ $bits / 8 }>, b: [< B $suffix:camel>]<{ $bits / 8 }>) -> bool {
              [< roundtrip_u $suffix:snake _ $suffix:snake _test>]::<{$bits / 8}, {$bits / 8}, { ($bits / 8) * 2}>(a, b)
            }

            fn [< fuzzy_ $suffix:snake _u $suffix:snake _ $bits _roundtrip>](a: [< B $suffix:camel>]<{ $bits / 8 }>, b: [< BU $suffix>]<{ $bits / 8 }>) -> bool {
              [< roundtrip_ $suffix:snake _u $suffix:snake _test>]::<{$bits / 8}, {$bits / 8}, { ($bits / 8) * 2}>(a, b)
            }
          }
        )*
      )*
    }
  };
}

fuzzy_packable!(
  intD8(8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192),
  intD16(8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192),
  intD32(8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192),
  int(8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192),
);
