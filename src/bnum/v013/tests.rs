use super::*;

macro_rules! assert_bnum_width_boundary {
  ($digit_bits:literal) => {{
    let upstream_max_n = u32::MAX as u128 / $digit_bits as u128;
    let max_n = if upstream_max_n > usize::MAX as u128 {
      usize::MAX
    } else {
      upstream_max_n as usize
    };

    let bit_width = match checked_bnum_bit_width(max_n, $digit_bits) {
      Some(bit_width) => bit_width,
      None => panic!("largest supported bnum width was rejected"),
    };
    assert!(bit_width as u128 == max_n as u128 * $digit_bits as u128);

    if max_n < usize::MAX {
      assert!(checked_bnum_bit_width(max_n + 1, $digit_bits).is_none());
    }
  }};
}

const _: () = {
  assert_bnum_width_boundary!(8);
  assert_bnum_width_boundary!(16);
  assert_bnum_width_boundary!(32);
  assert_bnum_width_boundary!(64);

  if usize::BITS >= 32 {
    assert!(checked_bnum_bit_width(usize::MAX, 8).is_none());
    assert!(checked_bnum_bit_width(usize::MAX, 16).is_none());
    assert!(checked_bnum_bit_width(usize::MAX, 32).is_none());
    assert!(checked_bnum_bit_width(usize::MAX, 64).is_none());
  }
};

macro_rules! assert_terminal_boundary {
  ($storage:literal, $unsigned:ident, $signed:ident, $wire:expr) => {{
    paste::paste! {
      let wire = $wire;

      let (read, value) = [< decode_uint_d $storage >]::<1>(&wire).unwrap();
      assert_eq!(read.get(), wire.len());
      assert!(value == $unsigned::<1>::MAX);

      let mut encoded = [0u8; 10];
      let written = [< encode_uint_d $storage _to>]($unsigned::<1>::MAX, &mut encoded).unwrap();
      assert_eq!(&encoded[..written.get()], &wire);

      let (read, value) = [< decode_int_d $storage >]::<1>(&wire).unwrap();
      assert_eq!(read.get(), wire.len());
      assert!(value == $signed::<1>::MIN);

      let mut encoded = [0u8; 10];
      let written = [< encode_int_d $storage _to>]($signed::<1>::MIN, &mut encoded).unwrap();
      assert_eq!(&encoded[..written.get()], &wire);
    }
  }};
}

macro_rules! assert_overlong_zero {
  ($storage:literal, $unsigned:ident) => {{
    paste::paste! {
      let (read, value) = [< decode_ $unsigned:snake _d $storage >]::<1>(&[0x80, 0x00]).unwrap();
      assert_eq!(read.get(), 2);
      assert!(value.is_zero());
    }
  }};
}

macro_rules! assert_zero_width {
  ($storage:literal, $unsigned:ident) => {{
    paste::paste! {
      let unsigned = $unsigned::<0>::ZERO;
      assert_eq!([< encoded_uint_d $storage _len >](&unsigned).get(), 1);
      assert_eq!(<$unsigned<0> as Varint>::MAX_ENCODED_LEN.get(), 1);

      let mut encoded = [0xff];
      assert_eq!([< encode_uint_d $storage _to>](unsigned, &mut encoded).unwrap().get(), 1);
      assert_eq!(encoded, [0]);

      let (read, decoded) = [< decode_uint_d $storage >]::<0>(&[0]).unwrap();
      assert_eq!(read.get(), 1);
      assert!(decoded.is_zero());
      assert!(matches!([< decode_uint_d $storage >]::<0>(&[1]), Err(ConstDecodeError::Overflow)));
    }
  }};
}

#[test]
fn terminal_payload_overflow_is_rejected() {
  let d8 = &[0x80, 0x02];
  assert!(matches!(
    decode_uint_d8::<1>(d8),
    Err(ConstDecodeError::Overflow)
  ));
  assert!(matches!(
    decode_int_d8::<1>(d8),
    Err(ConstDecodeError::Overflow)
  ));

  let d16 = &[0x80, 0x80, 0x04];
  assert!(matches!(
    decode_uint_d16::<1>(d16),
    Err(ConstDecodeError::Overflow)
  ));
  assert!(matches!(
    decode_int_d16::<1>(d16),
    Err(ConstDecodeError::Overflow)
  ));

  let d32 = &[0x80, 0x80, 0x80, 0x80, 0x10];
  assert!(matches!(
    decode_uint_d32::<1>(d32),
    Err(ConstDecodeError::Overflow)
  ));
  assert!(matches!(
    decode_int_d32::<1>(d32),
    Err(ConstDecodeError::Overflow)
  ));

  let d64 = &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x02];
  assert!(matches!(
    decode_uint_d64::<1>(d64),
    Err(ConstDecodeError::Overflow)
  ));
  assert!(matches!(
    decode_int_d64::<1>(d64),
    Err(ConstDecodeError::Overflow)
  ));
}

#[test]
fn terminal_payload_boundaries_round_trip() {
  assert_terminal_boundary!(8, BUintD8, BIntD8, [0xff, 0x01]);
  assert_terminal_boundary!(16, BUintD16, BIntD16, [0xff, 0xff, 0x03]);
  assert_terminal_boundary!(32, BUintD32, BIntD32, [0xff, 0xff, 0xff, 0xff, 0x0f]);
  assert_terminal_boundary!(
    64,
    BUint,
    BInt,
    [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01]
  );
}

#[test]
fn overlong_representable_zero_is_accepted() {
  assert_overlong_zero!(8, uint);
  assert_overlong_zero!(16, uint);
  assert_overlong_zero!(32, uint);
  assert_overlong_zero!(64, uint);
  assert_overlong_zero!(8, int);
  assert_overlong_zero!(16, int);
  assert_overlong_zero!(32, int);
  assert_overlong_zero!(64, int);
}

#[test]
fn zero_width_behavior_is_unchanged() {
  assert_zero_width!(8, BUintD8);
  assert_zero_width!(16, BUintD16);
  assert_zero_width!(32, BUintD32);
  assert_zero_width!(64, BUint);
}

macro_rules! fuzzy {
  ($base:ident($($ty:ident), +$(,)?)) => {
    $(
      paste::paste! {
        #[quickcheck_macros::quickcheck]
        fn [< fuzzy_$base:snake _ $ty:snake >](value: $ty) -> bool {
          let mut buf = [0; <$ty>::MAX_ENCODED_LEN.get()];
          let Ok(encoded_len) = value.encode(&mut buf) else { return false; };
          if encoded_len != value.encoded_len() || !(value.encoded_len() <= <$ty>::MAX_ENCODED_LEN) {
            return false;
          }

          let Some(consumed) = crate::consume_varint_checked(&buf) else {
            return false;
          };
          if consumed != encoded_len {
            return false;
          }

          if let Ok((bytes_read, decoded)) = <$ty>::decode(&buf) {
            value == decoded && encoded_len == bytes_read
          } else {
            false
          }
        }
      }
    )*
  };
}

macro_rules! define_aliases {
  ($sign:ident::$base:ident ($($ty:literal), +$(,)?)) => {
    paste::paste! {
      $(
        type [< $sign:camel $ty >] = $base<$ty>;
      )*
    }
  };
}

macro_rules! fuzzy_mod {
  ($(mod $mod_name:ident ($sign:ident::$base:ident($start:literal..=$end:literal))),+$(,)?) => {
    paste::paste! {
      $(
        mod $mod_name {
          use super::*;

          seq_macro::seq!(
            N in $start..=$end {
              define_aliases!($sign::$base(#(N,)*));

              fuzzy!($base(#([< $sign:camel >]~N,)*));
            }
          );
        }
      )*
    }
  };
}

fuzzy_mod! {
  mod buint_d8 (u::BUintD8(0..=64)),
  mod buint_d16 (u::BUintD16(0..=64)),
  mod buint_d32 (u::BUintD32(0..=64)),
  mod buint(u::BUint(0..=64)),
  mod bint_d8 (i::BIntD8(1..=64)),
  mod bint_d16 (i::BIntD16(1..=64)),
  mod bint_d32 (i::BIntD32(1..=64)),
  mod bint(i::BInt(1..=64)),
}
