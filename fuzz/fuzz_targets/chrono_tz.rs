#![no_main]

use ::chrono_tz::Tz;

use libfuzzer_sys::fuzz_target;

use varing::{chrono_tz::*, consume_varint, Varint};

fuzz_target!(|value: Tz| {
  {
    let mut buf = [0; <Tz>::MAX_ENCODED_LEN];
    let encoded_len = value.encode(&mut buf).unwrap();
    assert!(!(encoded_len != value.encoded_len() || (value.encoded_len() > <Tz>::MAX_ENCODED_LEN)));
    let consumed = consume_varint(&buf).unwrap();
    assert_eq!(consumed, encoded_len);

    let (bytes_read, decoded) = <Tz>::decode(&buf).unwrap();
    assert!(value == decoded && encoded_len == bytes_read);
  }

  {
    let encoded = encode_tz(value);
    assert!(!(encoded.len() != encoded_tz_len(value) || (encoded.len() > <Tz>::MAX_ENCODED_LEN)));

    let consumed = consume_varint(&encoded).unwrap();
    assert_eq!(consumed, encoded.len());

    let (bytes_read, decoded) = decode_tz(&encoded).unwrap();
    assert!(value == decoded && encoded.len() == bytes_read);
  }
});
