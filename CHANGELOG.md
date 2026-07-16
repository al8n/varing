# RELEASED

## 0.14.0 (Jul 17th, 2026)

Safety and correctness release from a deep audit of the decoders and the
encoded-length contracts. Malformed and non-canonical inputs now always
return a decode error instead of panicking, aliasing a valid value, or being
silently accepted. Valid-value wire encodings, `no_std`/no-alloc operation,
and const-callability are preserved.

### Fixed (soundness)

- `bnum` unequal-width `Packable::pack`/`unpack` performed an out-of-bounds
  read reachable from safe code; the digit copies are now correctly bounded.
- `MapDecoder` (and the sequence/map decode helpers) could construct
  `NonZeroUsize(0)` in release builds from a safe but adversarial `Varint`
  implementation; consumed lengths are now validated with checked arithmetic
  before slicing.

### Fixed (decoders reject malformed input instead of panicking or aliasing)

- The `bnum` and `ruint` decoders now reject excess data bits in a partial
  final byte instead of decoding an over-wide value to a truncated result.
- The core, `chrono`, and `time` `Duration` decoders no longer panic on
  hostile bytes: out-of-range seconds/nanoseconds are rejected, and signed
  durations reject sign-mismatched seconds and subseconds that would
  otherwise normalize to a different value.
- The `time`/`chrono` time, date, and datetime decoders reject non-canonical
  encodings (bits outside the packed layout can no longer alias a valid value).

### Fixed (encoded-length contracts)

- `MIN_ENCODED_LEN` for `Complex`, `Ratio`, and signed `arbitrary-int` types
  was set to a maximum; it is now the true minimum, so `ENCODED_LEN_RANGE`
  holds. For `Ratio` this is the length of the shortest valid value (a zero
  denominator is not representable, so the minimum is `0/1`, not one byte).
- Unsigned `Complex<bnum>` packed into an eightfold-oversized type; it now
  uses the correct `(bits / 8) * 2` digit width, shrinking `MAX_ENCODED_LEN`
  and the packed temporary and rejecting over-wide non-canonical aliases.

### Fixed (encoders)

- The specialized `encode_*_sequence_to` functions returned `Ok` after
  encoding only a prefix when the buffer was too small; they now fail with an
  insufficient-space error (all-or-error).

### Changed

- Timezone decoding (`chrono-tz`) uses an `O(1)` checked direct index instead
  of a linear scan over all variants, preserving the exact accept/reject set.

### Added

- `From<InsufficientData>` and `From<InsufficientSpace>` conversions into
  `std::io::Error` (`UnexpectedEof` and `WriteZero` kinds).

## 0.12.0 (Mar 14th, 2026)

- Extract `InsufficientData` into a standalone error struct (mirrors `InsufficientSpace`)
  - `required` field is now `Option<NonZeroUsize>` to support cases where the exact required size is unknown
  - Add `InsufficientData::new(available)` and `InsufficientData::with_required(required, available)` constructors
  - Add `insufficient_data_with_required` constructors on `ConstDecodeError` and `DecodeError`
- Change `ConstDecodeError::InsufficientData` and `DecodeError::InsufficientData` from struct variants to tuple variants wrapping `InsufficientData`

## 0.10.0 (Aug 12nd, 2025)

- Make `consume_varint` panic
- Add `consume_varint_checked` and `try_consume_varint` for non-panicking version `consume_varint`
- Change fn output from `usize` to `NonZeroUsize`

## 0.9.0 (Aug 9th, 2025)

- Add `ConstEncodeError` and `ConstDecodeError`
- Change `EncodeError::Other` and `DecodeError::Other` from `&'static str` to `Cow<'static, str>` on `alloc` and `std`

## 0.8.0 (Aug 3rd, 2025)

- Change `requested: usize` of `InsufficientSpace` to `requested: NonZeroUsize`
- Change `DecodeError::InsufficientData` to `DecodeError::InsufficientData { available: usize }`

## 0.7.0 (Aug 1st, 2025)

- Change from `*Error::Custom` to `*Error::Other`

## 0.6.1 (Jul 31st, 2025)

- Implement `From<EncodeError>` and `From<DecodeError>` for `std::io::Error`

## 0.6.0 (Jul 31st, 2025)

- **EncodeError**: Renamed `Underflow` variant to `InsufficientSpace`
  - Constructor method: `underflow(required, remaining)` → `insufficient_space(requested, available)`
- **DecodeError**: Renamed `Underflow` variant to `InsufficientData`

## 0.5.3 (Jul 6th, 2025)

- Adapt to `ruint` new changes

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
