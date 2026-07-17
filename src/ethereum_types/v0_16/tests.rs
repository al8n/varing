use crate::Varint;

use super::*;

type AU128 = ArbitraryType<BU128>;
type AU256 = ArbitraryType<BU256>;
type AU512 = ArbitraryType<BU512>;

type BU128 = bnum_0_13::types::U128;
type BU256 = bnum_0_13::types::U256;
type BU512 = bnum_0_13::types::U512;

impl From<AU128> for U128 {
  fn from(arbitrary: AU128) -> Self {
    Self(arbitrary.0.into())
  }
}

impl From<AU256> for U256 {
  fn from(arbitrary: AU256) -> Self {
    Self(arbitrary.0.into())
  }
}

impl From<AU512> for U512 {
  fn from(arbitrary: AU512) -> Self {
    Self(arbitrary.0.into())
  }
}

fuzzy_test!(128, 256, 512);
