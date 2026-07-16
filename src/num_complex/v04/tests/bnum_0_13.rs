use super::*;

use ::bnum_0_13::*;

fuzzy!(@varint_into(
  ComplexI128(Complex<i128>),
  ComplexU128(Complex<u128>),
));

#[quickcheck_macros::quickcheck]
fn fuzzy_complex_u128(value: ArbitraryComplex<u128>) -> bool {
  let value = ::core::convert::Into::into(value);
  let mut buf = [0; <Complex<u128>>::MAX_ENCODED_LEN.get()];
  let encoded = encode_complex_u128_to(&value, &mut buf).unwrap();
  if encoded != encoded_complex_u128_len(&value) || !(encoded <= <Complex<u128>>::MAX_ENCODED_LEN) {
    return false;
  }

  let Some(consumed) = crate::consume_varint_checked(&buf) else {
    return false;
  };
  if consumed != encoded {
    return false;
  }

  if let Ok((bytes_read, decoded)) = decode_complex_u128(&buf) {
    value == decoded && encoded == bytes_read
  } else {
    false
  }
}

#[quickcheck_macros::quickcheck]
fn fuzzy_complex_i128(value: ArbitraryComplex<i128>) -> bool {
  let value = ::core::convert::Into::into(value);
  let mut buf = [0; <Complex<i128>>::MAX_ENCODED_LEN.get()];
  let encoded = encode_complex_i128_to(&value, &mut buf).unwrap();
  if encoded != encoded_complex_i128_len(&value) || !(encoded <= <Complex<i128>>::MAX_ENCODED_LEN) {
    return false;
  }

  let Some(consumed) = crate::consume_varint_checked(&buf) else {
    return false;
  };
  if consumed != encoded {
    return false;
  }

  if let Ok((bytes_read, decoded)) = decode_complex_i128(&buf) {
    value == decoded && encoded == bytes_read
  } else {
    false
  }
}

macro_rules! complex_bnum_fuzzy {
  (@varint_into ($($ty:ident($target:ty)), +$(,)?)) => {
    $(
      paste::paste! {
        #[quickcheck_macros::quickcheck]
        fn [< fuzzy_ $ty:snake _varint>](value: $ty) -> bool {
          let value: $target = ::core::convert::Into::into(value);
          let mut buf = [0; <$target>::MAX_ENCODED_LEN.get()];
          let Ok(encoded_len) = value.encode(&mut buf) else { return false; };
          if encoded_len != value.encoded_len() || !(value.encoded_len() <= <$target>::MAX_ENCODED_LEN) {
            return false;
          }

          let Some(consumed) = $crate::consume_varint_checked(&buf) else {
            return false;
          };
          if consumed != encoded_len {
            return false;
          }

          if let Ok((bytes_read, decoded)) = <$target>::decode(&buf) {
            value.re == decoded.re && value.im == decoded.im && encoded_len == bytes_read
          } else {
            false
          }
        }
      }
    )*
  };
}

macro_rules! type_aliases {
  ($sign:ident::$storage:literal::$base:ident($($bits:literal), + $(,)?)) => {
    paste::paste! {
      $(
        type [<$sign $bits>] = $base<{ $bits / 8 }>;
      )*
    }
  };
}

macro_rules! test_mod {
  ($($sign:ident::$base:ident::$storage:literal),+$(,)?) => {
    paste::paste! {
      $(
        mod [<$base:snake>] {
          use super::*;
          use ::bnum_0_13::$base;

          type_aliases!(U::$storage::$base(8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192));

          impl_arbitrary_complex!(@bnum (U8, U16, U32, U64, U128, U256, U512, U1024, U2048,));

          complex_bnum_fuzzy!(@varint_into (
            BnumComplexU8(Complex<U8>),
            BnumComplexU16(Complex<U16>),
            BnumComplexU32(Complex<U32>),
            BnumComplexU64(Complex<U64>),
            BnumComplexU128(Complex<U128>),
            BnumComplexU256(Complex<U256>),
            BnumComplexU512(Complex<U512>),
            BnumComplexU1024(Complex<U1024>),
            BnumComplexU2048(Complex<U2048>),
          ));
        }
      )*
    }
  };
}

test_mod!(
  U::BUintD8::8,
  U::BUintD16::16,
  U::BUintD32::32,
  U::BUint::64,
  I::BIntD8::8,
  I::BIntD16::16,
  I::BIntD32::32,
  I::BInt::64,
);

// F6 + F7: the packed `Complex<bnum>` types must advertise a `MIN_ENCODED_LEN` that
// lower-bounds every value's encoded length. `Complex { 0, 0 }` is the shortest.
#[test]
fn min_encoded_len_in_range() {
  fn check<T: crate::Varint>(v: T) {
    let len = v.encoded_len().get();
    assert!(len >= T::MIN_ENCODED_LEN.get());
    assert!(len <= T::MAX_ENCODED_LEN.get());
    assert!(T::MIN_ENCODED_LEN.get() <= T::MAX_ENCODED_LEN.get());
  }
  // unsigned: small (8-bit component) and wide (256-bit component)
  check(Complex {
    re: BUintD8::<1>::ZERO,
    im: BUintD8::<1>::ZERO,
  });
  check(Complex {
    re: BUintD8::<32>::ZERO,
    im: BUintD8::<32>::ZERO,
  });
  // signed: small and wide
  check(Complex {
    re: BIntD8::<1>::ZERO,
    im: BIntD8::<1>::ZERO,
  });
  check(Complex {
    re: BIntD8::<32>::ZERO,
    im: BIntD8::<32>::ZERO,
  });
}

// F7: the unsigned `Complex<bnum>` path previously packed 8-bit components into an
// 8x-oversized `BUintD8<bits*2>`. The corrected `BUintD8<(bits/8)*2>` preserves the
// wire encoding of valid values, gives a tight `MAX_ENCODED_LEN`, and rejects
// over-wide malformed encodings the oversized type used to silently accept.
#[test]
fn f7_unsigned_complex_wire_and_rejection() {
  type C8 = Complex<BUintD8<1>>;

  // MAX_ENCODED_LEN now equals the (bits/8)*2 = `BUintD8<2>` (16-bit) bound of 3,
  // strictly smaller than the pre-fix `BUintD8<16>` (128-bit) bound of 19.
  assert_eq!(
    <C8 as crate::Varint>::MAX_ENCODED_LEN,
    BUintD8::<2>::MAX_ENCODED_LEN,
  );
  assert_eq!(<C8 as crate::Varint>::MAX_ENCODED_LEN.get(), 3);
  assert!(<C8 as crate::Varint>::MAX_ENCODED_LEN.get() < BUintD8::<16>::MAX_ENCODED_LEN.get());

  // Wire preservation: zero, small, and a value exercising the high bits of both
  // components all round-trip through encode/decode.
  let cases = [
    (BUintD8::<1>::ZERO, BUintD8::<1>::ZERO),
    (BUintD8::<1>::ONE, BUintD8::<1>::TWO),
    (BUintD8::<1>::MAX, BUintD8::<1>::MAX),
  ];
  for &(re, im) in cases.iter() {
    let value = Complex { re, im };
    let mut buf = [0u8; <C8 as crate::Varint>::MAX_ENCODED_LEN.get()];
    let n = value.encode(&mut buf).unwrap();
    let (read, decoded) = C8::decode(&buf).unwrap();
    assert_eq!(read, n);
    assert_eq!(decoded.re, re);
    assert_eq!(decoded.im, im);
  }

  // Rejection: `[0x80, 0x80, 0x04]` is the LEB128 encoding of `0x1_0000` (17 bits),
  // which sets a bit above the 16-bit merged width. The corrected merged type
  // overflows and rejects it; the pre-fix 128-bit merged type decoded it to
  // `Complex { 0, 0 }` (an over-wide alias of zero).
  assert!(C8::decode(&[0x80, 0x80, 0x04]).is_err());
}
