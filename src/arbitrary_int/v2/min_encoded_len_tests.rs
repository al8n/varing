use crate::Varint;
use ::arbitrary_int_2::prelude::*;

// F6: for a zigzag-encoded signed `Int`, `MIN_ENCODED_LEN` is the length of the
// shortest value (`0`, a single byte), not of the numeric minimum which encodes
// longest. The zero value must therefore encode within `MIN_ENCODED_LEN..=MAX`.
#[test]
fn signed_zero_in_range() {
  fn check<T: Varint>(zero: T) {
    let len = zero.encoded_len().get();
    assert_eq!(T::MIN_ENCODED_LEN.get(), 1);
    assert!(len >= T::MIN_ENCODED_LEN.get());
    assert!(len <= T::MAX_ENCODED_LEN.get());
    assert!(T::MIN_ENCODED_LEN.get() <= T::MAX_ENCODED_LEN.get());
  }
  // small width and wide width
  check(Int::<i8, 4>::try_new(0).unwrap());
  check(Int::<i128, 128>::try_new(0).unwrap());
}
