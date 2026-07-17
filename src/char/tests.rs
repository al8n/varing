use super::*;

use quickcheck_macros::quickcheck;

#[quickcheck]
fn encode_decode_char(value: char) -> bool {
  let encoded = encode_char(&value);
  if encoded.len() != encoded_char_len(&value).get()
    || (encoded.len() > <char>::MAX_ENCODED_LEN.get())
  {
    return false;
  }

  if let Ok((bytes_read, decoded)) = decode_char(&encoded) {
    value == decoded && encoded.len() == bytes_read.get()
  } else {
    false
  }
}

#[quickcheck]
fn encode_decode_char_varint(value: char) -> bool {
  let mut buf = [0; <char>::MAX_ENCODED_LEN.get()];
  let Ok(encoded_len) = value.encode(&mut buf) else {
    return false;
  };
  if encoded_len != value.encoded_len() || (value.encoded_len() > <char>::MAX_ENCODED_LEN) {
    return false;
  }

  if let Ok((bytes_read, decoded)) = <char>::decode(&buf) {
    value == decoded && encoded_len == bytes_read
  } else {
    false
  }
}
