#![no_main]

use bnum::{BInt, BIntD16, BIntD32, BIntD8, BUint, BUintD16, BUintD32, BUintD8};
use libfuzzer_sys::fuzz_target;
use varing::{consume_varint, Varint};

fuzz_target!(|data: Types| {
  data.check();
});

macro_rules! fuzzy {
    ($($ty:ty), +$(,)?) => {
        $(
            paste::paste! {
                fn [<check_ $ty:snake>](value: $ty) {
                    {
                      let mut buf = [0; <$ty>::MAX_ENCODED_LEN];
                      let encoded_len = value.encode(&mut buf).unwrap();
                      assert_eq!(encoded_len, value.encoded_len());
                      assert!(value.encoded_len() <= <$ty>::MAX_ENCODED_LEN);

                      let consumed = crate::consume_varint(&buf).unwrap();
                      assert_eq!(consumed, encoded_len);

                      let (bytes_read, decoded) = <$ty>::decode(&buf).unwrap();
                      assert!(value == decoded && encoded_len == bytes_read);
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

              fuzzy!(#([< $sign:camel >]~N,)*);

              #[derive(Debug, Clone, Copy, arbitrary::Arbitrary)]
              pub enum Number {
                #(
                  [< $sign:camel >]~N([< $sign:camel >]~N),
                )*
              }

              impl Number {
                pub fn check(self) {
                  match self {
                    #(
                      Self::[< $sign:camel >]~N(value) => [< check_ $sign:snake >]~N(value),
                    )*
                  }
                }
              }
            }
          );
        }
      )*
    }
  };
}

fuzzy_mod! {
  mod buint_d8 (u::BUintD8(0..=32)),
  mod buint_d16 (u::BUintD16(0..=32)),
  mod buint_d32 (u::BUintD32(0..=32)),
  mod buint(u::BUint(0..=32)),
  mod bint_d8 (i::BIntD8(1..=32)),
  mod bint_d16 (i::BIntD16(1..=32)),
  mod bint_d32 (i::BIntD32(1..=32)),
  mod bint(i::BInt(1..=32)),
}

#[derive(Copy, Clone, Debug, arbitrary::Arbitrary)]
enum Types {
  BIntD8(bint_d8::Number),
  BIntD16(bint_d16::Number),
  BIntD32(bint_d32::Number),
  BInt(bint::Number),
  BUintD8(buint_d8::Number),
  BUintD16(buint_d16::Number),
  BUintD32(buint_d32::Number),
  BUint(buint::Number),
}

impl Types {
  fn check(self) {
    match self {
      Self::BIntD8(val) => val.check(),
      Self::BIntD16(val) => val.check(),
      Self::BIntD32(val) => val.check(),
      Self::BInt(val) => val.check(),
      Self::BUintD8(val) => val.check(),
      Self::BUintD16(val) => val.check(),
      Self::BUintD32(val) => val.check(),
      Self::BUint(val) => val.check(),
    }
  }
}
