use super::Packable;
use crate::utils::*;

macro_rules! impl_packable_for_primitives {
  (@self $($bits:literal => $packed:literal), +$(,)?) => {
    impl_packable_for_primitives!(@self_packable $($bits => $packed),+);
    impl_packable_for_primitives!(@self_fn $($bits => $packed),+);
  };
  (@self_fn $($bits:literal => $packed:literal), +$(,)?) => {
    paste::paste! {
      $(
        #[doc = "Packs two `u" $bits "` into a `u" $packed "`."]
        #[inline]
        pub const fn [< pack_u $bits >](low: [<u $bits>], high: [<u $bits>]) -> [< u $packed>] {
          low as [< u $packed>] | (high as [< u $packed>]) << $bits
        }

        #[doc = "Unpacks a `u" $packed "` into two `u" $bits "`."]
        #[inline]
        pub const fn [< unpack_u $bits >](value: [< u $packed>]) -> ([<u $bits>], [<u $bits>]) {
          let low = value as [< u $bits >];
          let high = (value >> $bits) as [< u $bits >];
          (low, high)
        }

        #[doc = "Packs two `i" $bits "` into a `u" $packed "`."]
        #[inline]
        pub const fn [< pack_i $bits >](low: [<i $bits>], high: [<i $bits>]) -> [< u $packed>] {
          let low = $crate::utils::[< zigzag_encode_i $bits>](low);
          let high = $crate::utils::[< zigzag_encode_i $bits>](high);
          [< pack_u $bits>](low, high)
        }

        #[doc = "Unpacks a `u" $packed "` into two `i" $bits "`."]
        #[inline]
        pub const fn [< unpack_i $bits >](value: [< u $packed>]) -> ([<i $bits>], [<i $bits>]) {
          let (low, high) = [< unpack_u $bits>](value);
          let low = $crate::utils::[< zigzag_decode_i $bits>](low);
          let high = $crate::utils::[< zigzag_decode_i $bits>](high);

          (low, high)
        }

        #[doc = "Packs `u" $bits "` and `i" $bits "` into a single `u" $packed "` value, which suitable for LEB128 encoding/decoding."]
        #[inline]
        pub const fn [<pack_ u $bits _i $bits>](a: [<u $bits>], b: [<i $bits>]) -> [< u $packed>] {
          let b = [< zigzag_encode_i $bits >](b);
          [<pack_ u $bits>](a, b)
        }

        #[doc = "Packs `i" $bits "` and `u" $bits "` into a single `u" $packed "` value, which suitable for LEB128 encoding/decoding."]
        #[inline]
        pub const fn [<pack_ i $bits _u $bits>](a: [<i $bits>], b: [<u $bits>]) -> [< u $packed>] {
          [<pack_ u $bits _i $bits>](b, a)
        }

        #[doc = "Unpacks `u" $packed "` into `u" $bits "` and `i" $bits "` values."]
        #[inline]
        pub const fn [<unpack_ u $bits _i $bits>](packed: [< u $packed>]) -> ([< u $bits >], [< i $bits >]) {
          let (a, b) = [<unpack_ u $bits>](packed);
          (a, [< zigzag_decode_i $bits >](b))
        }

        #[doc = "Unpacks `u" $packed "` into `i" $bits "` and `u" $bits "` values."]
        #[inline]
        pub const fn [<unpack_ i $bits _u $bits>](packed: [< u $packed>]) -> ([< i $bits >], [< u $bits >]) {
          let (a, b) = [<unpack_ u $bits _i $bits>](packed);
          (b, a)
        }
      )*
    }
  };
  (@self_packable $($bits:literal => $packed:literal), +$(,)?) => {
    paste::paste! {
      $(
        impl Packable<Self, [< u $packed>]> for [<u $bits>] {
          fn pack(&self, rhs: &Self) -> [< u $packed>] {
            [< pack_u $bits >](*self, *rhs)
          }

          fn unpack(packed: [< u $packed>]) -> (Self, Self) where Self: Sized, [< u $bits>]: Sized {
            [< unpack_u $bits >](packed)
          }
        }

        impl Packable<Self, [< u $packed>]> for [<i $bits>] {
          fn pack(&self, rhs: &Self) -> [< u $packed>] {
            [< pack_i $bits >](*self, *rhs)
          }

          fn unpack(packed: [< u $packed>]) -> (Self, Self) where Self: Sized, [< i $bits>]: Sized {
            [< unpack_i $bits >](packed)
          }
        }

        impl Packable<[< i $bits >], [< u $packed>]> for [< u $bits >] {
          fn pack(&self, rhs: &[< i $bits >]) -> [< u $packed>] {
            [<pack_ u $bits _i $bits>](*self, *rhs)
          }

          fn unpack(packed: [< u $packed>]) -> (Self, [< i $bits >]) where Self: Sized, [< i $bits >]: Sized {
            [<unpack_ u $bits _i $bits>](packed)
          }
        }

        impl Packable<[< u $bits >], [< u $packed>]> for [< i $bits >] {
          #[inline]
          fn pack(&self, rhs: &[< u $bits >]) -> [< u $packed>] {
            [<pack_ i $bits _u $bits>](*self, *rhs)
          }

          #[inline]
          fn unpack(packed: [< u $packed>]) -> (Self, [< u $bits >]) where Self: Sized, [< u $bits >]: Sized {
            [<unpack_ i $bits _u $bits>](packed)
          }
        }
      )*
    }
  };
  ($(($a:literal, $b:literal) => $packed:literal), +$(,)?) => {
    paste::paste! {
      impl_packable_for_primitives!(@exchange $(($a, $b) => $packed),+);
      impl_packable_for_primitives!(@exchange_fn $(($a, $b) => $packed),+);
      impl_packable_for_primitives!(@mix $(($a, $b) => $packed),+);
      impl_packable_for_primitives!(@mix_fn $(($a, $b) => $packed),+);
    }
  };
  (@exchange $(($a:literal, $b:literal) => $packed:literal), +$(,)?) => {
    paste::paste! {
      $(
        impl Packable<[< u $b >], [< u $packed>]> for [< u $a >] {
          fn pack(&self, rhs: &[< u $b >]) -> [< u $packed>] {
            [<pack_ u $a _u $b>](*self, *rhs)
          }

          fn unpack(packed: [< u $packed>]) -> (Self, [< u $b >]) where Self: Sized, [< u $b >]: Sized {
            [<unpack_ u $a _u $b>](packed)
          }
        }

        impl Packable<[< u $a >], [< u $packed>]> for [< u $b >] {
          #[inline]
          fn pack(&self, rhs: &[< u $a >]) -> [< u $packed>] {
            [<pack_ u $b _u $a>](*self, *rhs)
          }

          #[inline]
          fn unpack(packed: [< u $packed>]) -> (Self, [< u $a >]) where Self: Sized, [< u $a >]: Sized {
            [<unpack_ u $b _u $a>](packed)
          }
        }

        impl Packable<[< i $b >], [< u $packed>]> for [< i $a >] {
          fn pack(&self, rhs: &[< i $b >]) -> [< u $packed>] {
            [<pack_ i $a _i $b>](*self, *rhs)
          }

          fn unpack(packed: [< u $packed>]) -> (Self, [< i $b >]) where Self: Sized, [< i $b >]: Sized {
            [<unpack_ i $a _i $b>](packed)
          }
        }

        impl Packable<[< i $a >], [< u $packed>]> for [< i $b >] {
          #[inline]
          fn pack(&self, rhs: &[< i $a >]) -> [< u $packed>] {
            [<pack_ i $b _i $a>](*self, *rhs)
          }

          #[inline]
          fn unpack(packed: [< u $packed>]) -> (Self, [< i $a >]) where Self: Sized, [< u $a >]: Sized {
            [<unpack_ i $b _i $a>](packed)
          }
        }
      )*
    }
  };
  (@exchange_fn $(($a:literal, $b:literal) => $packed:literal), +$(,)?) => {
    paste::paste! {
      $(
        #[doc = "Packs `i" $a "` and `i" $b "` into a single `u" $packed "` value, which suitable for LEB128 encoding/decoding."]
        #[inline]
        pub const fn [<pack_ i $a _i $b>](a: [<i $a>], b: [<i $b>]) -> [< u $packed>] {
          let a = [< zigzag_encode_i $a >](a);
          let b = [< zigzag_encode_i $b >](b);
          [<pack_ u $a _u $b>](a, b)
        }

        #[doc = "Packs `i" $b "` and `i" $a "` into a single `u" $packed "` value, which suitable for LEB128 encoding/decoding."]
        #[inline]
        pub const fn [<pack_ i $b _i $a>](a: [<i $b>], b: [<i $a>]) -> [< u $packed>] {
          [<pack_ i $a _i $b>](b, a)
        }

        #[doc = "Unpacks `u" $packed "` into `i" $a "` and `i" $b "` values."]
        #[inline]
        pub const fn [<unpack_ i $a _i $b>](packed: [< u $packed>]) -> ([< i $a >], [< i $b >]) {
          let (a, b) = [<unpack_ u $a _u $b>](packed);
          ([< zigzag_decode_i $a >](a), [< zigzag_decode_i $b >](b))
        }

        #[doc = "Unpacks `u" $packed "` into `i" $b "` and `i" $a "` values."]
        #[inline]
        pub const fn [<unpack_ i $b _i $a>](packed: [< u $packed>]) -> ([< i $b >], [< i $a >]) {
          let (a, b) = [<unpack_ i $a _i $b>](packed);
          (b, a)
        }

        #[doc = "Packs `u" $a "` and `u" $b "` into a single `u" $packed "` value, which suitable for LEB128 encoding/decoding."]
        #[inline]
        pub const fn [<pack_ u $a _u $b>](a: [<u $a>], b: [<u $b>]) -> [< u $packed>] {
          let small = a as [< u $packed >];
          let large = b as [< u $packed >];

          (large << [< u $a >]::BITS) | small
        }

        #[doc = "Packs `u" $b "` and `u" $a "` into a single `u" $packed "` value, which suitable for LEB128 encoding/decoding."]
        #[inline]
        pub const fn [<pack_ u $b _u $a>](a: [<u $b>], b: [<u $a>]) -> [< u $packed>] {
          [<pack_ u $a _u $b>](b, a)
        }

        #[doc = "Unpacks `u" $packed "` into `u" $a "` and `u" $b "` values."]
        #[inline]
        pub const fn [<unpack_ u $a _u $b>](packed: [< u $packed>]) -> ([< u $a >], [< u $b >]) {
          let small_mask: [< u $packed >] = (1 << [< u $a >]::BITS) - 1;

          let small: [< u $packed >] = packed & small_mask;
          let large: [< u $packed >] = packed >> [< u $a >]::BITS;

          (small as _, large as _)
        }

        #[doc = "Unpacks `u" $packed "` into `u" $b "` and `u" $a "` values."]
        #[inline]
        pub const fn [<unpack_ u $b _u $a>](packed: [< u $packed>]) -> ([< u $b >], [< u $a >]) {
          let (a, b) = [<unpack_ u $a _u $b>](packed);
          (b, a)
        }
      )*
    }
  };
  (@mix $(($a:literal, $b:literal) => $packed:literal), +$(,)?) => {
    paste::paste! {
      $(
        impl Packable<[< i $b >], [< u $packed>]> for [< u $a >] {
          fn pack(&self, rhs: &[< i $b >]) -> [< u $packed>] {
            let b = [< zigzag_encode_i $b >](*rhs);
            [< pack_ u $a _u $b>](*self, b)
          }

          fn unpack(packed: [< u $packed>]) -> (Self, [< i $b >]) where Self: Sized, [< i $b >]: Sized {
            let (a, b) = <[< u $a >] as Packable<[< u $b >], [< u $packed>]>>::unpack(packed);
            (a, [< zigzag_decode_i $b >](b))
          }
        }

        impl Packable<[< u $a >], [< u $packed>]> for [< i $b >] {
          #[inline]
          fn pack(&self, rhs: &[< u $a >]) -> [< u $packed>] {
            <[< u $a >] as Packable<[< i $b >], [< u $packed>]>>::pack(rhs, self)
          }

          #[inline]
          fn unpack(packed: [< u $packed>]) -> (Self, [< u $a >]) where Self: Sized, [< u $a >]: Sized {
            let (a, b) = <[< u $a >] as Packable<[< i $b >], [< u $packed>]>>::unpack(packed);
            (b, a)
          }
        }

        impl Packable<[< u $b >], [< u $packed>]> for [< i $a >] {
          fn pack(&self, rhs: &[< u $b >]) -> [< u $packed>] {
            [< pack_ i $a _u $b>](*self, *rhs)
          }

          fn unpack(packed: [< u $packed>]) -> (Self, [< u $b >]) where Self: Sized, [< u $b >]: Sized {
            [< unpack_ i $a _u $b>](packed)
          }
        }

        impl Packable<[< i $a >], [< u $packed>]> for [< u $b >] {
          #[inline]
          fn pack(&self, rhs: &[< i $a >]) -> [< u $packed>] {
            [< pack_ u $b _i $a>](*self, *rhs)
          }

          #[inline]
          fn unpack(packed: [< u $packed>]) -> (Self, [< i $a >]) where Self: Sized, [< i $a >]: Sized {
            [< unpack_ u $b _i $a>](packed)
          }
        }
      )*
    }
  };
  (@mix_fn $(($a:literal, $b:literal) => $packed:literal), +$(,)?) => {
    paste::paste! {
      $(
        #[doc = "Packs `u" $a "` and `i" $b "` into a single `u" $packed "` value, which suitable for LEB128 encoding/decoding."]
        #[inline]
        pub const fn [<pack_ u $a _i $b>](a: [<u $a>], b: [<i $b>]) -> [< u $packed>] {
          let b = [< zigzag_encode_i $b >](b);
          [<pack_ u $a _u $b>](a, b)
        }

        #[doc = "Packs `i" $b "` and `u" $a "` into a single `u" $packed "` value, which suitable for LEB128 encoding/decoding."]
        #[inline]
        pub const fn [<pack_ i $b _u $a>](a: [<i $b>], b: [<u $a>]) -> [< u $packed>] {
          [<pack_ u $a _i $b>](b, a)
        }

        #[doc = "Unpacks `u" $packed "` into `u" $a "` and `i" $b "` values."]
        #[inline]
        pub const fn [<unpack_ u $a _i $b>](packed: [< u $packed>]) -> ([< u $a >], [< i $b >]) {
          let (a, b) = [<unpack_ u $a _u $b>](packed);
          (a, [< zigzag_decode_i $b >](b))
        }

        #[doc = "Unpacks `u" $packed "` into `i" $b "` and `u" $a "` values."]
        #[inline]
        pub const fn [<unpack_ i $b _u $a>](packed: [< u $packed>]) -> ([< i $b >], [< u $a >]) {
          let (a, b) = [<unpack_ u $a _i $b>](packed);
          (b, a)
        }

        #[doc = "Packs `i" $a "` and `u" $b "` into a single `u" $packed "` value, which suitable for LEB128 encoding/decoding."]
        #[inline]
        pub const fn [<pack_ i $a _u $b>](a: [<i $a>], b: [<u $b>]) -> [< u $packed>] {
          let a = [< zigzag_encode_i $a >](a);
          [<pack_ u $a _u $b>](a, b)
        }

        #[doc = "Packs `u" $b "` and `i" $a "` into a single `u" $packed "` value, which suitable for LEB128 encoding/decoding."]
        #[inline]
        pub const fn [<pack_ u $b _i $a>](a: [<u $b>], b: [<i $a>]) -> [< u $packed>] {
          [<pack_ i $a _u $b>](b, a)
        }

        #[doc = "Unpacks `u" $packed "` into `i" $a "` and `u" $b "` values."]
        #[inline]
        pub const fn [<unpack_ i $a _u $b>](packed: [< u $packed>]) -> ([< i $a >], [< u $b >]) {
          let (a, b) = [<unpack_ u $a _u $b>](packed);
          ([< zigzag_decode_i $a >](a), b)
        }

        #[doc = "Unpacks `u" $packed "` into `u" $b "` and `i" $a "` values."]
        #[inline]
        pub const fn [<unpack_ u $b _i $a>](packed: [< u $packed>]) -> ([< u $b >], [< i $a >]) {
          let (a, b) = [<unpack_ i $a _u $b>](packed);
          (b, a)
        }
      )*
    }
  };
}

impl_packable_for_primitives!(@self
  8 => 16,
  16 => 32,
  32 => 64,
  64 => 128,
);

impl_packable_for_primitives!(
  (8, 16) => 32,
  (8, 32) => 64,
  (8, 64) => 128,
  (16, 32) => 64,
  (16, 64) => 128,
  (32, 64) => 128,
);

#[cfg(feature = "bnum_0_13")]
#[allow(non_camel_case_types)]
type u256 = ::bnum_0_13::BUintD8<32>;

#[cfg(all(feature = "ruint_1", not(feature = "bnum_0_13")))]
#[allow(non_camel_case_types)]
type u256 = ::ruint_1::aliases::U256;

macro_rules! pack_128_exchange {
  ($($const:ident)? $fg: literal) => {
    /// Packs `u128` and `i128` into a single `u256` value, which suitable for LEB128 encoding/decoding.
    #[cfg(feature = $fg)]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "ruint_1", feature = "bnum_0_13"))))]
    pub $($const)? fn pack_u128_i128(a: u128, b: i128) -> u256 {
      let b = zigzag_encode_i128(b);
      pack_u128(a, b)
    }

    /// Unpacks `u256` into `u128` and `i128` values.
    #[cfg(feature = $fg)]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "ruint_1", feature = "bnum_0_13"))))]
    pub $($const)? fn unpack_u128_i128(value: u256) -> (u128, i128) {
      let (low, high) = unpack_u128(value);
      let high = zigzag_decode_i128(high);
      (low, high)
    }

    /// Packs `i128` and `u128` into a single `u256` value, which suitable for LEB128 encoding/decoding.
    #[cfg(feature = $fg)]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "ruint_1", feature = "bnum_0_13"))))]
    pub $($const)? fn pack_i128_u128(a: i128, b: u128) -> u256 {
      pack_u128_i128(b, a)
    }

    /// Unpacks `u256` into `i128` and `u128` values.
    #[cfg(feature = $fg)]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "ruint_1", feature = "bnum_0_13"))))]
    pub $($const)? fn unpack_i128_u128(value: u256) -> (i128, u128) {
      let (a, b) = unpack_u128_i128(value);
      (b, a)
    }
  };
}

#[cfg(any(feature = "bnum_0_13", feature = "ruint_1"))]
impl_packable_for_primitives!(@self_packable
  128 => 256,
);

#[cfg(feature = "bnum_0_13")]
pack_128_exchange!(const "bnum_0_13");

#[cfg(all(feature = "ruint_1", not(feature = "bnum_0_13")))]
pack_128_exchange!("ruint_1");

/// Pack two `u128`s into a single `u256`
#[cfg(feature = "bnum_0_13")]
#[cfg_attr(docsrs, doc(cfg(any(feature = "ruint_1", feature = "bnum_0_13"))))]
pub const fn pack_u128(low: u128, high: u128) -> u256 {
  let Some(low) = u256::from_le_slice(&low.to_le_bytes()) else {
    unreachable!();
  };

  let Some(high) = u256::from_le_slice(&high.to_le_bytes()) else {
    unreachable!();
  };

  low.bitor(high.shl(128))
}

/// Unpack a single `u256` into two `u128`s
#[cfg(feature = "bnum_0_13")]
#[cfg_attr(docsrs, doc(cfg(any(feature = "ruint_1", feature = "bnum_0_13"))))]
pub const fn unpack_u128(value: u256) -> (u128, u128) {
  use ::bnum_0_13::BUintD8;
  use core::ptr::copy;

  let Some(max_u128) = u256::from_le_slice(&u128::MAX.to_le_bytes()) else {
    unreachable!();
  };
  let low = value.bitand(max_u128);
  let high: BUintD8<32> = value.shr(128).bitand(max_u128);

  let (low, _) = low.digits().split_at(16);
  let mut buf = [0u8; 16];
  // SAFETY: `low` is always 16 bytes long
  unsafe {
    copy(low.as_ptr(), buf.as_mut_ptr(), 16);

    let low = u128::from_le_bytes(buf);
    let (high, _) = high.digits().split_at(16);
    copy(high.as_ptr(), buf.as_mut_ptr(), 16);

    let high = u128::from_le_bytes(buf);
    (low, high)
  }
}

/// Pack two `i128`s into a single `u256`
#[cfg(feature = "bnum_0_13")]
#[cfg_attr(docsrs, doc(cfg(any(feature = "ruint_1", feature = "bnum_0_13"))))]
pub const fn pack_i128(low: i128, high: i128) -> u256 {
  let low = zigzag_encode_i128(low);
  let high = zigzag_encode_i128(high);
  pack_u128(low, high)
}

/// Unpack a single `u256` into two `i128`s
#[cfg(feature = "bnum_0_13")]
#[cfg_attr(docsrs, doc(cfg(any(feature = "ruint_1", feature = "bnum_0_13"))))]
pub const fn unpack_i128(value: u256) -> (i128, i128) {
  let (low, high) = unpack_u128(value);
  let low = zigzag_decode_i128(low);
  let high = zigzag_decode_i128(high);

  (low, high)
}

/// Pack two `u128`s into a single `u256`
#[cfg(all(feature = "ruint_1", not(feature = "bnum_0_13")))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "ruint_1", feature = "bnum_0_13"))))]
pub fn pack_u128(low: u128, high: u128) -> u256 {
  u256::from(low) | (u256::from(high) << 128)
}

/// Unpack a single `u256` into two `u128`s
#[cfg(all(feature = "ruint_1", not(feature = "bnum_0_13")))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "ruint_1", feature = "bnum_0_13"))))]
pub fn unpack_u128(value: u256) -> (u128, u128) {
  let low = value & u256::from(u128::MAX);
  let high: u256 = (value >> 128) & u256::from(u128::MAX);

  (low.to(), high.to())
}

/// Pack two `i128`s into a single `u256`
#[cfg(all(feature = "ruint_1", not(feature = "bnum_0_13")))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "ruint_1", feature = "bnum_0_13"))))]
pub fn pack_i128(low: i128, high: i128) -> u256 {
  let low = zigzag_encode_i128(low);
  let high = zigzag_encode_i128(high);
  pack_u128(low, high)
}

/// Unpack a single `u256` into two `i128`s
#[cfg(all(feature = "ruint_1", not(feature = "bnum_0_13")))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "ruint_1", feature = "bnum_0_13"))))]
pub fn unpack_i128(value: u256) -> (i128, i128) {
  let (low, high) = unpack_u128(value);
  let low = zigzag_decode_i128(low);
  let high = zigzag_decode_i128(high);

  (low, high)
}
