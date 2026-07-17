use crate::{Varint, consume_varint_checked};
use core::num::*;

macro_rules! fuzzy {
  ($($ty:ty), +$(,)?) => {
    $(
      paste::paste! {
        #[quickcheck_macros::quickcheck]
        fn [< fuzzy_non_zero_ $ty _varint>](value: $ty) -> bool {
          if value == 0 {
            return true;
          }

          let value = [< NonZero $ty:camel >]::new(value).unwrap();

          let mut buf = [0; { <[< NonZero $ty:camel >]>::MAX_ENCODED_LEN.get() }];
          let Ok(encoded_len) = value.encode(&mut buf) else { return false; };
          if encoded_len != value.encoded_len() || !(value.encoded_len() <= <[< NonZero $ty:camel >]>::MAX_ENCODED_LEN) {
            return false;
          }

          let Some(consumed) = consume_varint_checked(&buf) else {
            return false;
          };
          if consumed != encoded_len {
            return false;
          }

          if let Ok((bytes_read, decoded)) = <[< NonZero $ty:camel >]>::decode(&buf) {
            value == decoded && encoded_len == bytes_read
          } else {
            false
          }
        }
      }
    )*
  };
}

fuzzy!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128,);
