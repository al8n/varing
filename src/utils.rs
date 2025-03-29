/// Zigzag encode `i8` value.
#[inline]
pub const fn zigzag_encode_i8(value: i8) -> u8 {
  ((value << 1) ^ (value >> 7)) as u8
}

/// Zigzag encode `i16` value.
#[inline]
pub const fn zigzag_encode_i16(value: i16) -> u16 {
  ((value << 1) ^ (value >> 15)) as u16
}

/// Zigzag encode `i32` value.
#[inline]
pub const fn zigzag_encode_i32(value: i32) -> u32 {
  ((value << 1) ^ (value >> 31)) as u32
}

/// Zigzag encode `i64` value.
#[inline]
pub const fn zigzag_encode_i64(value: i64) -> u64 {
  ((value << 1) ^ (value >> 63)) as u64
}

/// Zigzag encode `i128` value.
#[inline]
pub const fn zigzag_encode_i128(value: i128) -> u128 {
  ((value << 1) ^ (value >> 127)) as u128
}

/// Zigzag decode `i8` value.
#[inline]
pub const fn zigzag_decode_i8(value: u8) -> i8 {
  ((value >> 1) as i8) ^ (-((value & 1) as i8))
}

/// Zigzag decode `i16` value.
#[inline]
pub const fn zigzag_decode_i16(value: u16) -> i16 {
  ((value >> 1) as i16) ^ (-((value & 1) as i16))
}

/// Zigzag decode `i32` value.
#[inline]
pub const fn zigzag_decode_i32(value: u32) -> i32 {
  ((value >> 1) as i32) ^ (-((value & 1) as i32))
}

/// Zigzag decode `i64` value.
#[inline]
pub const fn zigzag_decode_i64(value: u64) -> i64 {
  ((value >> 1) as i64) ^ (-((value & 1) as i64))
}

/// Zigzag decode `i128` value.
#[inline]
pub const fn zigzag_decode_i128(value: u128) -> i128 {
  ((value >> 1) as i128) ^ (-((value & 1) as i128))
}

macro_rules! pack_unpack {
  (@unsigned $($bits:literal($merged_ty:ident)),+$(,)?) => {
    paste::paste! {
      $(
        /// Pack two unsigned integers into a single value.
        #[inline]
        pub const fn [< pack_u $bits >](low: [<u $bits>], high: [<u $bits>]) -> $merged_ty {
          low as $merged_ty | (high as $merged_ty) << $bits
        }

        /// Unpack a single value into two unsigned integers.
        #[inline]
        pub const fn [< unpack_u $bits >](value: $merged_ty) -> ([<u $bits>], [<u $bits>]) {
          let low = value as [< u $bits >];
          let high = (value >> $bits) as [< u $bits >];
          (low, high)
        }
      )*
    }
  };
  (@signed $($bits:literal($merged_ty:ident)),+$(,)?) => {
    paste::paste! {
      $(
        /// Pack two signed integers into a single value.
        #[inline]
        pub const fn [< pack_i $bits >](low: [<i $bits>], high: [<i $bits>]) -> $merged_ty {
          let low = $crate::utils::[< zigzag_encode_i $bits>](low);
          let high = $crate::utils::[< zigzag_encode_i $bits>](high);
          [< pack_u $bits>](low, high)
        }

        /// Unpack a single value into two unsigned integers.
        #[inline]
        pub const fn [< unpack_i $bits >](value: $merged_ty) -> ([<i $bits>], [<i $bits>]) {
          let (low, high) = [< unpack_u $bits>](value);
          let low = $crate::utils::[< zigzag_decode_i $bits>](low);
          let high = $crate::utils::[< zigzag_decode_i $bits>](high);

          (low, high)
        }
      )*
    }
  };
}

pack_unpack!(@unsigned 8(u16), 16(u32), 32(u64), 64(u128));
pack_unpack!(@signed 8(u16), 16(u32), 32(u64), 64(u128));

/// Pack two `u128`s into a single `U256`
#[cfg(feature = "bnum_0_13")]
#[cfg_attr(docsrs, doc(cfg(any(feature = "ruint_1", feature = "bnum_0_13"))))]
pub const fn pack_u128(low: u128, high: u128) -> ::bnum_0_13::BUintD8<32> {
  use ::bnum_0_13::BUintD8;

  // low.copy_from_slice(low.to_le_bytes().as_slice());
  let Some(low) = BUintD8::<32>::from_le_slice(&low.to_le_bytes()) else {
    unreachable!();
  };

  let Some(high) = BUintD8::<32>::from_le_slice(&high.to_le_bytes()) else {
    unreachable!();
  };

  low.bitor(high.shl(128))
}

/// Unpack a single `U256` into two `u128`s
#[cfg(feature = "bnum_0_13")]
#[cfg_attr(docsrs, doc(cfg(any(feature = "ruint_1", feature = "bnum_0_13"))))]
pub const fn unpack_u128(value: ::bnum_0_13::BUintD8<32>) -> (u128, u128) {
  use ::bnum_0_13::BUintD8;
  use core::ptr::copy;

  let Some(max_u128) = ::bnum_0_13::BUintD8::from_le_slice(&u128::MAX.to_le_bytes()) else {
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

/// Pack two `i128`s into a single `U256`
#[cfg(feature = "bnum_0_13")]
#[cfg_attr(docsrs, doc(cfg(any(feature = "ruint_1", feature = "bnum_0_13"))))]
pub const fn pack_i128(low: i128, high: i128) -> ::bnum_0_13::BUintD8<32> {
  let low = zigzag_encode_i128(low);
  let high = zigzag_encode_i128(high);
  pack_u128(low, high)
}

/// Unpack a single `U256` into two `i128`s
#[cfg(feature = "bnum_0_13")]
#[cfg_attr(docsrs, doc(cfg(any(feature = "ruint_1", feature = "bnum_0_13"))))]
pub const fn unpack_i128(value: ::bnum_0_13::BUintD8<32>) -> (i128, i128) {
  let (low, high) = unpack_u128(value);
  let low = zigzag_decode_i128(low);
  let high = zigzag_decode_i128(high);

  (low, high)
}

/// Pack two `u128`s into a single `U256`
#[cfg(all(feature = "ruint_1", not(feature = "bnum_0_13")))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "ruint_1", feature = "bnum_0_13"))))]
pub fn pack_u128(low: u128, high: u128) -> ::ruint_1::aliases::U256 {
  use ::ruint_1::aliases::U256;

  U256::from(low) | (U256::from(high) << 128)
}

/// Unpack a single `U256` into two `u128`s
#[cfg(all(feature = "ruint_1", not(feature = "bnum_0_13")))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "ruint_1", feature = "bnum_0_13"))))]
pub fn unpack_u128(value: ::ruint_1::aliases::U256) -> (u128, u128) {
  use ::ruint_1::aliases::U256;
  let low = value & U256::from(u128::MAX);
  let high: U256 = (value >> 128) & U256::from(u128::MAX);

  (low.to(), high.to())
}

/// Pack two `i128`s into a single `U256`
#[cfg(all(feature = "ruint_1", not(feature = "bnum_0_13")))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "ruint_1", feature = "bnum_0_13"))))]
pub fn pack_i128(low: i128, high: i128) -> ::ruint_1::aliases::U256 {
  let low = zigzag_encode_i128(low);
  let high = zigzag_encode_i128(high);
  pack_u128(low, high)
}

/// Unpack a single `U256` into two `i128`s
#[cfg(all(feature = "ruint_1", not(feature = "bnum_0_13")))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "ruint_1", feature = "bnum_0_13"))))]
pub fn unpack_i128(value: ::ruint_1::aliases::U256) -> (i128, i128) {
  let (low, high) = unpack_u128(value);
  let low = zigzag_decode_i128(low);
  let high = zigzag_decode_i128(high);

  (low, high)
}
