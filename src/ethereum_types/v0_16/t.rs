use super::*;
use crate::Varint;

type AU64 = ArbitraryType<BU64>;

impl From<AU64> for U64 {
  fn from(arbitrary: AU64) -> Self {
    Self(arbitrary.0.into())
  }
}

fuzzy_test!(64);
