use super::*;

#[derive(Debug, Clone, Copy)]
struct FuzzyF16(f16);

impl quickcheck::Arbitrary for FuzzyF16 {
  fn arbitrary(g: &mut quickcheck::Gen) -> Self {
    loop {
      let val = f16::from_bits(u16::arbitrary(g));
      if !val.is_nan() {
        break Self(val);
      }
    }
  }
}

quickcheck::quickcheck! {
  fn fuzzy_f16_varint(value: FuzzyF16) -> bool {
    let value = value.0;
    let mut buf = [0u8; { f16::MAX_ENCODED_LEN.get() + 1 }];
    let len = value.encoded_len();
    let len2 = value.encode(&mut buf).unwrap();
    assert_eq!(len, len2);
    let (read, value2) = f16::decode(&buf[..len.get()]).unwrap();
    assert_eq!(len, read);
    assert_eq!(value, value2);

    encode_f16_varint(value).as_slice() == &buf[..len.get()]
  }
}

#[cfg(feature = "std")]
mod with_std {
  use super::*;

  quickcheck::quickcheck! {
    fn fuzzy_f16_sequence(value: std::vec::Vec<FuzzyF16>) -> bool {
      let value = value.into_iter().map(|v| v.0).collect::<std::vec::Vec<_>>();
      let encoded_len = encoded_f16_sequence_len(&value);
      let mut buf = std::vec![0; encoded_len];
      let Ok(written) = encode_f16_sequence_to(&value, &mut buf) else { return false; };
      if encoded_len != written {
        return false;
      }

      let (readed, decoded) = crate::decode_sequence::<f16, std::vec::Vec<_>>(&buf).unwrap();
      if encoded_len != readed {
        return false;
      }

      assert_eq!(decoded.len(), value.len());

      for (a, b) in decoded.iter().zip(value.iter()) {
        if a.to_bits() != b.to_bits() {
          return false;
        }
      }

      true
    }
  }
}
