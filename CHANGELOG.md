# RELEASED

## 0.5.2 (Apr 30th, 2025)

- Implement `Varint` for `f32`, `f64` and `half::f16`.

## 0.5.1 (Apr 12th, 2025)

FEATURES

- Add `Packable` and fns to encoding/decoding sequences and maps

## 0.5.0 (Apr 3rd, 2025)

FEATURES

- Cleanup `*Buffer` to const generic `Buffer<N>`
- Add `chrono-tz` support

## 0.4.1 (Mar 29th, 2025)

FEATURES

- Add `chrono` and `time` support
- Add `num-rational`, `num-complex` and `bnum` support
- Rename the crate from `const-varint` to `varing`

## 0.4.0 (Mar 22nd, 2025)

FEATURES

- Implement `Varint` trait for `NonZero*` types
- Add `EncodeError::Custom` and `DecodeError::Custom`
- Add `encode_char`, `encoded_char_len`, `encode_char_to` and `decode_char`

## 0.3.3 (Mar 22nd, 2025)

FEATURES

- Add `arbitrary-int` to support `u1`, `u2`, `u3` .. `u127`
- Add `primitive-types` and `ethereum-types` support

## 0.3.0 (Mar 19th, 2025)

FEATURES

- Add `encode_u8_varint`, `encode_u8_varint_to`, `encoded_u8_varint_len` and `U8VarintBuffer`
- Add `encode_i8_varint`, `encode_i8_varint_to`, `encoded_i8_varint_len` and `I8VarintBuffer`

## 0.2.0 (Feb 5th, 2025)

FEATURES

- Add more information for `EncodeError::Underflow`
