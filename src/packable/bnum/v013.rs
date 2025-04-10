use super::super::Packable;
use bnum_0_13::*;

macro_rules! impl_packable {
  ($($suffix: ident), +$(,)?) => {
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

          let mut small = [0; P];
          let mut large = [0; P];

          // SAFETY: We have checked that L + R <= P, so we can safely copy the digits.
          // We also know that L and R are less than or equal to P, so we can safely
          // copy the digits from a and b into small and large.
          let small_bits = if L > R {
            unsafe { core::ptr::copy(a.digits().as_ptr(), large.as_mut_ptr(), L) };
            unsafe { core::ptr::copy(b.digits().as_ptr(), small.as_mut_ptr(), R) };
            R
          } else {
            unsafe { core::ptr::copy(b.digits().as_ptr(), large.as_mut_ptr(), R) };
            unsafe { core::ptr::copy(a.digits().as_ptr(), small.as_mut_ptr(), L) };
            L
          };

          let small = [< BU $suffix >]::<P>::from_digits(small);
          let large = [< BU $suffix >]::<P>::from_digits(large);

          large.shl(small_bits as u32).bitor(small)
        }

        #[doc = "Unpacks `BU`" $suffix "<P>` into `BU" $suffix "<L>` and `BU" $suffix "<R>`"]
        pub const fn [< unpack_u $suffix:snake >]<const L: usize, const R: usize, const P: usize>(packed: [< BU $suffix >]<P>) -> ([< BU $suffix >]<L>, [< BU $suffix >]<R>) {
          assert!(L + R <= P, "L + R must be less than or equal to P");

          if L == 0 && R == 0 {
            return ([< BU $suffix >]::<L>::ZERO, [< BU $suffix >]::<R>::ZERO);
          }

          let mut mask = [0; P];

          if L > R {
            unsafe {
              core::ptr::copy([< BU $suffix >]::<R>::MAX.digits().as_ptr(), mask.as_mut_ptr(), R);
            }
            let small = packed.bitand([< BU $suffix >]::<P>::from_digits(mask));
            let large = packed.shr(R as u32);

            let mut lhs = [0; L];
            unsafe {
              core::ptr::copy(large.digits().as_ptr(), lhs.as_mut_ptr(), L);
            }
            let lhs = [< BU $suffix >]::<L>::from_digits(lhs);
            let mut rhs = [0; R];
            unsafe {
              core::ptr::copy(small.digits().as_ptr(), rhs.as_mut_ptr(), R);
            }
            let rhs = [< BU $suffix >]::<R>::from_digits(rhs);
            (lhs, rhs)
          } else {
            unsafe {
              core::ptr::copy([< BU $suffix >]::<L>::MAX.digits().as_ptr(), mask.as_mut_ptr(), L);
            }
            let small = packed.bitand([< BU $suffix >]::<P>::from_digits(mask));
            let large = packed.shr(L as u32);

            let mut lhs = [0; L];
            unsafe {
              core::ptr::copy(small.digits().as_ptr(), lhs.as_mut_ptr(), L);
            }
            let lhs = [< BU $suffix >]::<L>::from_digits(lhs);
            let mut rhs = [0; R];
            unsafe {
              core::ptr::copy(large.digits().as_ptr(), rhs.as_mut_ptr(), R);
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
      )*
    }
  };
}

impl_packable!(intD8, intD16, intD32, int,);
