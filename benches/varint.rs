//! Criterion benchmark suite comparing `varing`'s const LEB128 varint
//! encode/decode against other mature Rust varint crates.
//!
//! `varing` uses plain (unsigned) LEB128 for unsigned integers and
//! zigzag + LEB128 for signed integers -- the same scheme as `integer-encoding`.
//! This suite benchmarks that scheme against:
//!
//! - `integer-encoding` (`VarInt::encode_var` / `VarInt::decode_var`): the closest match,
//!   same LEB128 + zigzag scheme. Only implements `VarInt` for `u8/u16/u32/u64/usize` and
//!   `i8/i16/i32/i64/isize` (no 128-bit support), so it is excluded from the `u128` groups.
//! - `leb128` (`leb128::write::unsigned` / `leb128::read::unsigned`): raw unsigned LEB128.
//!   Its public API is hard-coded to `u64`/`i64`, so it only appears in the `u64` unsigned
//!   groups. `leb128`'s *signed* encoding is true signed-LEB128 (sign-extension based), a
//!   different wire format from zigzag, so it is deliberately excluded from the `i64` groups
//!   to avoid presenting an apples-to-oranges number as if it were an equivalent comparison.
//! - `unsigned-varint` (`unsigned_varint::encode` / `unsigned_varint::decode`): unsigned-only,
//!   but it does implement `u16`/`u32`/`u64`/`u128`, so it appears in every unsigned group.
//!
//! Each case is benchmarked against four value distributions to reflect realistic workloads:
//! a 1-byte-encodable small value, a mid-range multi-byte value, the type maximum, and a
//! fixed (seeded, reproducible) pseudo-random sweep of ~1000 values. The sweep uses a small
//! SplitMix64 generator with a fixed seed -- not `rand`/OS randomness -- so the exact same
//! values are used on every run and every machine.
//!
//! Groups are named `"<encode|decode>/<type>/<distribution>"` (e.g. `"encode/u64/small"`),
//! and within each group every comparison library appears as its own named bench
//! (`"varing"`, `"integer-encoding"`, `"unsigned-varint"`, `"leb128"`), so `cargo bench`
//! output and the criterion HTML report show the libraries side by side for the same case.

use criterion::{Criterion, criterion_group, criterion_main};
use integer_encoding::VarInt;
use std::hint::black_box;
use unsigned_varint::{decode as uvi_decode, encode as uvi_encode};

/// Number of values in each "mixed" pseudo-random sweep.
const MIXED_LEN: usize = 1000;

/// Fixed seed for the SplitMix64 generator backing the "mixed" distributions.
///
/// This is a constant, not an OS-seeded RNG: every run of this benchmark suite, on every
/// machine, sweeps the exact same sequence of values.
const MIXED_SEED: u64 = 0x0DDB_1A5E_5EED_C0DE;

const U16_MAX_LEN: usize = <u16 as varing::Varint>::MAX_ENCODED_LEN.get();
const U32_MAX_LEN: usize = <u32 as varing::Varint>::MAX_ENCODED_LEN.get();
const U64_MAX_LEN: usize = <u64 as varing::Varint>::MAX_ENCODED_LEN.get();
const U128_MAX_LEN: usize = <u128 as varing::Varint>::MAX_ENCODED_LEN.get();
const I64_MAX_LEN: usize = <i64 as varing::Varint>::MAX_ENCODED_LEN.get();

/// A single SplitMix64 step: deterministic, fast, decent-quality bit mixing.
///
/// Used only to build fixed, reproducible pseudo-random value arrays for the "mixed"
/// benchmarks -- this is not meant to be a cryptographic or statistical-quality PRNG.
fn splitmix64(state: &mut u64) -> u64 {
  *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
  let mut z = *state;
  z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
  z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
  z ^ (z >> 31)
}

/// Generates a value whose bit-length is uniform over `1..=max_bits`, then fills that many
/// low bits with fresh random bits.
///
/// A naive full-width uniform sample (e.g. plain `splitmix64() as u64`) is a poor stand-in
/// for a "mixed" workload: each extra bit doubles the range, so roughly *half* of a
/// full-range uniform u64 sample would already need the maximum 10 LEB128 bytes, drowning
/// out the small/short encodings we want this distribution to exercise. Sampling the
/// bit-length uniformly first instead gives a roughly even spread across every encoded
/// byte-length the type can produce.
fn random_magnitude(state: &mut u64, max_bits: u32) -> u128 {
  let bits = (splitmix64(state) as u32 % max_bits) + 1;
  let hi = splitmix64(state) as u128;
  let lo = splitmix64(state) as u128;
  let raw = (hi << 64) | lo;
  if bits >= 128 {
    raw
  } else {
    raw & ((1u128 << bits) - 1)
  }
}

fn mixed_u16() -> Vec<u16> {
  let mut state = MIXED_SEED;
  (0..MIXED_LEN)
    .map(|_| random_magnitude(&mut state, 16) as u16)
    .collect()
}

fn mixed_u32() -> Vec<u32> {
  let mut state = MIXED_SEED;
  (0..MIXED_LEN)
    .map(|_| random_magnitude(&mut state, 32) as u32)
    .collect()
}

fn mixed_u64() -> Vec<u64> {
  let mut state = MIXED_SEED;
  (0..MIXED_LEN)
    .map(|_| random_magnitude(&mut state, 64) as u64)
    .collect()
}

fn mixed_u128() -> Vec<u128> {
  let mut state = MIXED_SEED;
  (0..MIXED_LEN)
    .map(|_| random_magnitude(&mut state, 128))
    .collect()
}

fn mixed_i64() -> Vec<i64> {
  let mut state = MIXED_SEED;
  (0..MIXED_LEN)
    .map(|_| {
      let magnitude = random_magnitude(&mut state, 63) as i64;
      if splitmix64(&mut state) & 1 == 1 {
        -magnitude
      } else {
        magnitude
      }
    })
    .collect()
}

/// Encodes `values` back-to-back into a freshly allocated buffer using `encode_one`,
/// truncated to the bytes actually written. Used to pre-build a realistic mixed-length
/// "stream" once, outside the timed decode loop.
fn build_stream<T: Copy>(
  values: &[T],
  max_len: usize,
  mut encode_one: impl FnMut(T, &mut [u8]) -> usize,
) -> Vec<u8> {
  let mut buf = vec![0u8; values.len() * max_len];
  let mut cursor = 0;
  for &value in values {
    cursor += encode_one(value, &mut buf[cursor..]);
  }
  buf.truncate(cursor);
  buf
}

/// Encodes every value in `values` into `scratch` (reused, overwritten from the start on
/// every call), returning the number of bytes written. Meant to be called once per
/// criterion iteration so the timed cost covers encoding the whole sweep.
fn sweep_encode<T: Copy>(
  values: &[T],
  scratch: &mut [u8],
  mut encode_one: impl FnMut(T, &mut [u8]) -> usize,
) -> usize {
  let mut cursor = 0;
  for &value in values {
    cursor += encode_one(black_box(value), &mut scratch[cursor..]);
  }
  cursor
}

/// Decodes `count` consecutive values from `stream` using `decode_one`, which returns the
/// number of bytes consumed and the decoded value. Meant to be called once per criterion
/// iteration so the timed cost covers decoding the whole sweep.
fn sweep_decode<T>(stream: &[u8], count: usize, mut decode_one: impl FnMut(&[u8]) -> (usize, T)) {
  let mut pos = 0;
  for _ in 0..count {
    let (n, v) = decode_one(black_box(&stream[pos..]));
    pos += n;
    black_box(v);
  }
}

// ---------------------------------------------------------------------------------------
// u16
// ---------------------------------------------------------------------------------------

fn bench_encode_u16(c: &mut Criterion) {
  const CASES: [(&str, u16); 3] = [("small", 5), ("mid", 12_345), ("max", u16::MAX)];

  for (label, value) in CASES {
    let mut group = c.benchmark_group(format!("encode/u16/{label}"));

    let mut buf = [0u8; U16_MAX_LEN];
    group.bench_function("varing", |b| {
      b.iter(|| {
        let n = varing::encode_u16_varint_to(black_box(value), &mut buf).unwrap();
        black_box(&buf[..n.get()]);
      })
    });

    let mut buf = [0u8; U16_MAX_LEN];
    group.bench_function("integer-encoding", |b| {
      b.iter(|| {
        let n = black_box(value).encode_var(&mut buf);
        black_box(&buf[..n]);
      })
    });

    let mut uvi_buf = uvi_encode::u16_buffer();
    group.bench_function("unsigned-varint", |b| {
      b.iter(|| {
        let written = uvi_encode::u16(black_box(value), &mut uvi_buf);
        black_box(written);
      })
    });

    group.finish();
  }

  let values = mixed_u16();
  let mut group = c.benchmark_group("encode/u16/mixed");

  let mut scratch = vec![0u8; MIXED_LEN * U16_MAX_LEN];
  group.bench_function("varing", |b| {
    b.iter(|| {
      let n = sweep_encode(&values, &mut scratch, |v, buf| {
        varing::encode_u16_varint_to(v, buf).unwrap().get()
      });
      black_box(&scratch[..n]);
    })
  });

  let mut scratch = vec![0u8; MIXED_LEN * U16_MAX_LEN];
  group.bench_function("integer-encoding", |b| {
    b.iter(|| {
      let n = sweep_encode(&values, &mut scratch, |v, buf| v.encode_var(buf));
      black_box(&scratch[..n]);
    })
  });

  let mut scratch = vec![0u8; MIXED_LEN * U16_MAX_LEN];
  group.bench_function("unsigned-varint", |b| {
    b.iter(|| {
      let n = sweep_encode(&values, &mut scratch, |v, buf| {
        let mut small = uvi_encode::u16_buffer();
        let written = uvi_encode::u16(v, &mut small);
        buf[..written.len()].copy_from_slice(written);
        written.len()
      });
      black_box(&scratch[..n]);
    })
  });

  group.finish();
}

fn bench_decode_u16(c: &mut Criterion) {
  const CASES: [(&str, u16); 3] = [("small", 5), ("mid", 12_345), ("max", u16::MAX)];

  for (label, value) in CASES {
    let mut group = c.benchmark_group(format!("decode/u16/{label}"));

    let mut buf = [0u8; U16_MAX_LEN];
    let n = varing::encode_u16_varint_to(value, &mut buf).unwrap();
    let encoded = &buf[..n.get()];

    group.bench_function("varing", |b| {
      b.iter(|| {
        let (_, v) = varing::decode_u16_varint(black_box(encoded)).unwrap();
        black_box(v);
      })
    });

    group.bench_function("integer-encoding", |b| {
      b.iter(|| {
        let (v, _) = <u16 as VarInt>::decode_var(black_box(encoded)).unwrap();
        black_box(v);
      })
    });

    group.bench_function("unsigned-varint", |b| {
      b.iter(|| {
        let (v, _) = uvi_decode::u16(black_box(encoded)).unwrap();
        black_box(v);
      })
    });

    group.finish();
  }

  let values = mixed_u16();
  let mut group = c.benchmark_group("decode/u16/mixed");

  let varing_stream = build_stream(&values, U16_MAX_LEN, |v, buf| {
    varing::encode_u16_varint_to(v, buf).unwrap().get()
  });
  group.bench_function("varing", |b| {
    b.iter(|| {
      sweep_decode(&varing_stream, MIXED_LEN, |buf| {
        let (n, v) = varing::decode_u16_varint(buf).unwrap();
        (n.get(), v)
      })
    })
  });

  let ie_stream = build_stream(&values, U16_MAX_LEN, |v, buf| v.encode_var(buf));
  group.bench_function("integer-encoding", |b| {
    b.iter(|| {
      sweep_decode(&ie_stream, MIXED_LEN, |buf| {
        let (v, n) = <u16 as VarInt>::decode_var(buf).unwrap();
        (n, v)
      })
    })
  });

  let uvi_stream = build_stream(&values, U16_MAX_LEN, |v, buf| {
    let mut small = uvi_encode::u16_buffer();
    let written = uvi_encode::u16(v, &mut small);
    buf[..written.len()].copy_from_slice(written);
    written.len()
  });
  group.bench_function("unsigned-varint", |b| {
    b.iter(|| {
      sweep_decode(&uvi_stream, MIXED_LEN, |buf| {
        let (v, rest) = uvi_decode::u16(buf).unwrap();
        (buf.len() - rest.len(), v)
      })
    })
  });

  group.finish();
}

// ---------------------------------------------------------------------------------------
// u32
// ---------------------------------------------------------------------------------------

fn bench_encode_u32(c: &mut Criterion) {
  const CASES: [(&str, u32); 3] = [("small", 5), ("mid", 1_000_000), ("max", u32::MAX)];

  for (label, value) in CASES {
    let mut group = c.benchmark_group(format!("encode/u32/{label}"));

    let mut buf = [0u8; U32_MAX_LEN];
    group.bench_function("varing", |b| {
      b.iter(|| {
        let n = varing::encode_u32_varint_to(black_box(value), &mut buf).unwrap();
        black_box(&buf[..n.get()]);
      })
    });

    let mut buf = [0u8; U32_MAX_LEN];
    group.bench_function("integer-encoding", |b| {
      b.iter(|| {
        let n = black_box(value).encode_var(&mut buf);
        black_box(&buf[..n]);
      })
    });

    let mut uvi_buf = uvi_encode::u32_buffer();
    group.bench_function("unsigned-varint", |b| {
      b.iter(|| {
        let written = uvi_encode::u32(black_box(value), &mut uvi_buf);
        black_box(written);
      })
    });

    group.finish();
  }

  let values = mixed_u32();
  let mut group = c.benchmark_group("encode/u32/mixed");

  let mut scratch = vec![0u8; MIXED_LEN * U32_MAX_LEN];
  group.bench_function("varing", |b| {
    b.iter(|| {
      let n = sweep_encode(&values, &mut scratch, |v, buf| {
        varing::encode_u32_varint_to(v, buf).unwrap().get()
      });
      black_box(&scratch[..n]);
    })
  });

  let mut scratch = vec![0u8; MIXED_LEN * U32_MAX_LEN];
  group.bench_function("integer-encoding", |b| {
    b.iter(|| {
      let n = sweep_encode(&values, &mut scratch, |v, buf| v.encode_var(buf));
      black_box(&scratch[..n]);
    })
  });

  let mut scratch = vec![0u8; MIXED_LEN * U32_MAX_LEN];
  group.bench_function("unsigned-varint", |b| {
    b.iter(|| {
      let n = sweep_encode(&values, &mut scratch, |v, buf| {
        let mut small = uvi_encode::u32_buffer();
        let written = uvi_encode::u32(v, &mut small);
        buf[..written.len()].copy_from_slice(written);
        written.len()
      });
      black_box(&scratch[..n]);
    })
  });

  group.finish();
}

fn bench_decode_u32(c: &mut Criterion) {
  const CASES: [(&str, u32); 3] = [("small", 5), ("mid", 1_000_000), ("max", u32::MAX)];

  for (label, value) in CASES {
    let mut group = c.benchmark_group(format!("decode/u32/{label}"));

    let mut buf = [0u8; U32_MAX_LEN];
    let n = varing::encode_u32_varint_to(value, &mut buf).unwrap();
    let encoded = &buf[..n.get()];

    group.bench_function("varing", |b| {
      b.iter(|| {
        let (_, v) = varing::decode_u32_varint(black_box(encoded)).unwrap();
        black_box(v);
      })
    });

    group.bench_function("integer-encoding", |b| {
      b.iter(|| {
        let (v, _) = <u32 as VarInt>::decode_var(black_box(encoded)).unwrap();
        black_box(v);
      })
    });

    group.bench_function("unsigned-varint", |b| {
      b.iter(|| {
        let (v, _) = uvi_decode::u32(black_box(encoded)).unwrap();
        black_box(v);
      })
    });

    group.finish();
  }

  let values = mixed_u32();
  let mut group = c.benchmark_group("decode/u32/mixed");

  let varing_stream = build_stream(&values, U32_MAX_LEN, |v, buf| {
    varing::encode_u32_varint_to(v, buf).unwrap().get()
  });
  group.bench_function("varing", |b| {
    b.iter(|| {
      sweep_decode(&varing_stream, MIXED_LEN, |buf| {
        let (n, v) = varing::decode_u32_varint(buf).unwrap();
        (n.get(), v)
      })
    })
  });

  let ie_stream = build_stream(&values, U32_MAX_LEN, |v, buf| v.encode_var(buf));
  group.bench_function("integer-encoding", |b| {
    b.iter(|| {
      sweep_decode(&ie_stream, MIXED_LEN, |buf| {
        let (v, n) = <u32 as VarInt>::decode_var(buf).unwrap();
        (n, v)
      })
    })
  });

  let uvi_stream = build_stream(&values, U32_MAX_LEN, |v, buf| {
    let mut small = uvi_encode::u32_buffer();
    let written = uvi_encode::u32(v, &mut small);
    buf[..written.len()].copy_from_slice(written);
    written.len()
  });
  group.bench_function("unsigned-varint", |b| {
    b.iter(|| {
      sweep_decode(&uvi_stream, MIXED_LEN, |buf| {
        let (v, rest) = uvi_decode::u32(buf).unwrap();
        (buf.len() - rest.len(), v)
      })
    })
  });

  group.finish();
}

// ---------------------------------------------------------------------------------------
// u64 (also includes `leb128`, which is hard-coded to u64/i64)
// ---------------------------------------------------------------------------------------

fn bench_encode_u64(c: &mut Criterion) {
  const CASES: [(&str, u64); 3] = [("small", 5), ("mid", 1_000_000_000_000), ("max", u64::MAX)];

  for (label, value) in CASES {
    let mut group = c.benchmark_group(format!("encode/u64/{label}"));

    let mut buf = [0u8; U64_MAX_LEN];
    group.bench_function("varing", |b| {
      b.iter(|| {
        let n = varing::encode_u64_varint_to(black_box(value), &mut buf).unwrap();
        black_box(&buf[..n.get()]);
      })
    });

    let mut buf = [0u8; U64_MAX_LEN];
    group.bench_function("integer-encoding", |b| {
      b.iter(|| {
        let n = black_box(value).encode_var(&mut buf);
        black_box(&buf[..n]);
      })
    });

    let mut uvi_buf = uvi_encode::u64_buffer();
    group.bench_function("unsigned-varint", |b| {
      b.iter(|| {
        let written = uvi_encode::u64(black_box(value), &mut uvi_buf);
        black_box(written);
      })
    });

    let mut buf = [0u8; U64_MAX_LEN];
    group.bench_function("leb128", |b| {
      b.iter(|| {
        let mut w: &mut [u8] = &mut buf;
        let n = leb128::write::unsigned(&mut w, black_box(value)).unwrap();
        black_box(&buf[..n]);
      })
    });

    group.finish();
  }

  let values = mixed_u64();
  let mut group = c.benchmark_group("encode/u64/mixed");

  let mut scratch = vec![0u8; MIXED_LEN * U64_MAX_LEN];
  group.bench_function("varing", |b| {
    b.iter(|| {
      let n = sweep_encode(&values, &mut scratch, |v, buf| {
        varing::encode_u64_varint_to(v, buf).unwrap().get()
      });
      black_box(&scratch[..n]);
    })
  });

  let mut scratch = vec![0u8; MIXED_LEN * U64_MAX_LEN];
  group.bench_function("integer-encoding", |b| {
    b.iter(|| {
      let n = sweep_encode(&values, &mut scratch, |v, buf| v.encode_var(buf));
      black_box(&scratch[..n]);
    })
  });

  let mut scratch = vec![0u8; MIXED_LEN * U64_MAX_LEN];
  group.bench_function("unsigned-varint", |b| {
    b.iter(|| {
      let n = sweep_encode(&values, &mut scratch, |v, buf| {
        let mut small = uvi_encode::u64_buffer();
        let written = uvi_encode::u64(v, &mut small);
        buf[..written.len()].copy_from_slice(written);
        written.len()
      });
      black_box(&scratch[..n]);
    })
  });

  let mut scratch = vec![0u8; MIXED_LEN * U64_MAX_LEN];
  group.bench_function("leb128", |b| {
    b.iter(|| {
      let n = sweep_encode(&values, &mut scratch, |v, mut buf: &mut [u8]| {
        leb128::write::unsigned(&mut buf, v).unwrap()
      });
      black_box(&scratch[..n]);
    })
  });

  group.finish();
}

fn bench_decode_u64(c: &mut Criterion) {
  const CASES: [(&str, u64); 3] = [("small", 5), ("mid", 1_000_000_000_000), ("max", u64::MAX)];

  for (label, value) in CASES {
    let mut group = c.benchmark_group(format!("decode/u64/{label}"));

    let mut buf = [0u8; U64_MAX_LEN];
    let n = varing::encode_u64_varint_to(value, &mut buf).unwrap();
    let encoded = &buf[..n.get()];

    group.bench_function("varing", |b| {
      b.iter(|| {
        let (_, v) = varing::decode_u64_varint(black_box(encoded)).unwrap();
        black_box(v);
      })
    });

    group.bench_function("integer-encoding", |b| {
      b.iter(|| {
        let (v, _) = <u64 as VarInt>::decode_var(black_box(encoded)).unwrap();
        black_box(v);
      })
    });

    group.bench_function("unsigned-varint", |b| {
      b.iter(|| {
        let (v, _) = uvi_decode::u64(black_box(encoded)).unwrap();
        black_box(v);
      })
    });

    group.bench_function("leb128", |b| {
      b.iter(|| {
        let mut r: &[u8] = black_box(encoded);
        let v = leb128::read::unsigned(&mut r).unwrap();
        black_box(v);
      })
    });

    group.finish();
  }

  let values = mixed_u64();
  let mut group = c.benchmark_group("decode/u64/mixed");

  let varing_stream = build_stream(&values, U64_MAX_LEN, |v, buf| {
    varing::encode_u64_varint_to(v, buf).unwrap().get()
  });
  group.bench_function("varing", |b| {
    b.iter(|| {
      sweep_decode(&varing_stream, MIXED_LEN, |buf| {
        let (n, v) = varing::decode_u64_varint(buf).unwrap();
        (n.get(), v)
      })
    })
  });

  let ie_stream = build_stream(&values, U64_MAX_LEN, |v, buf| v.encode_var(buf));
  group.bench_function("integer-encoding", |b| {
    b.iter(|| {
      sweep_decode(&ie_stream, MIXED_LEN, |buf| {
        let (v, n) = <u64 as VarInt>::decode_var(buf).unwrap();
        (n, v)
      })
    })
  });

  let uvi_stream = build_stream(&values, U64_MAX_LEN, |v, buf| {
    let mut small = uvi_encode::u64_buffer();
    let written = uvi_encode::u64(v, &mut small);
    buf[..written.len()].copy_from_slice(written);
    written.len()
  });
  group.bench_function("unsigned-varint", |b| {
    b.iter(|| {
      sweep_decode(&uvi_stream, MIXED_LEN, |buf| {
        let (v, rest) = uvi_decode::u64(buf).unwrap();
        (buf.len() - rest.len(), v)
      })
    })
  });

  let leb128_stream = build_stream(&values, U64_MAX_LEN, |v, mut buf: &mut [u8]| {
    leb128::write::unsigned(&mut buf, v).unwrap()
  });
  group.bench_function("leb128", |b| {
    b.iter(|| {
      sweep_decode(&leb128_stream, MIXED_LEN, |buf| {
        let mut r = buf;
        let before = r.len();
        let v = leb128::read::unsigned(&mut r).unwrap();
        (before - r.len(), v)
      })
    })
  });

  group.finish();
}

// ---------------------------------------------------------------------------------------
// u128 (integer-encoding and leb128 do not support 128-bit integers -- see module docs)
// ---------------------------------------------------------------------------------------

fn bench_encode_u128(c: &mut Criterion) {
  const CASES: [(&str, u128); 3] = [("small", 5), ("mid", u64::MAX as u128), ("max", u128::MAX)];

  for (label, value) in CASES {
    let mut group = c.benchmark_group(format!("encode/u128/{label}"));

    let mut buf = [0u8; U128_MAX_LEN];
    group.bench_function("varing", |b| {
      b.iter(|| {
        let n = varing::encode_u128_varint_to(black_box(value), &mut buf).unwrap();
        black_box(&buf[..n.get()]);
      })
    });

    let mut uvi_buf = uvi_encode::u128_buffer();
    group.bench_function("unsigned-varint", |b| {
      b.iter(|| {
        let written = uvi_encode::u128(black_box(value), &mut uvi_buf);
        black_box(written);
      })
    });

    group.finish();
  }

  let values = mixed_u128();
  let mut group = c.benchmark_group("encode/u128/mixed");

  let mut scratch = vec![0u8; MIXED_LEN * U128_MAX_LEN];
  group.bench_function("varing", |b| {
    b.iter(|| {
      let n = sweep_encode(&values, &mut scratch, |v, buf| {
        varing::encode_u128_varint_to(v, buf).unwrap().get()
      });
      black_box(&scratch[..n]);
    })
  });

  let mut scratch = vec![0u8; MIXED_LEN * U128_MAX_LEN];
  group.bench_function("unsigned-varint", |b| {
    b.iter(|| {
      let n = sweep_encode(&values, &mut scratch, |v, buf| {
        let mut small = uvi_encode::u128_buffer();
        let written = uvi_encode::u128(v, &mut small);
        buf[..written.len()].copy_from_slice(written);
        written.len()
      });
      black_box(&scratch[..n]);
    })
  });

  group.finish();
}

fn bench_decode_u128(c: &mut Criterion) {
  const CASES: [(&str, u128); 3] = [("small", 5), ("mid", u64::MAX as u128), ("max", u128::MAX)];

  for (label, value) in CASES {
    let mut group = c.benchmark_group(format!("decode/u128/{label}"));

    let mut buf = [0u8; U128_MAX_LEN];
    let n = varing::encode_u128_varint_to(value, &mut buf).unwrap();
    let encoded = &buf[..n.get()];

    group.bench_function("varing", |b| {
      b.iter(|| {
        let (_, v) = varing::decode_u128_varint(black_box(encoded)).unwrap();
        black_box(v);
      })
    });

    group.bench_function("unsigned-varint", |b| {
      b.iter(|| {
        let (v, _) = uvi_decode::u128(black_box(encoded)).unwrap();
        black_box(v);
      })
    });

    group.finish();
  }

  let values = mixed_u128();
  let mut group = c.benchmark_group("decode/u128/mixed");

  let varing_stream = build_stream(&values, U128_MAX_LEN, |v, buf| {
    varing::encode_u128_varint_to(v, buf).unwrap().get()
  });
  group.bench_function("varing", |b| {
    b.iter(|| {
      sweep_decode(&varing_stream, MIXED_LEN, |buf| {
        let (n, v) = varing::decode_u128_varint(buf).unwrap();
        (n.get(), v)
      })
    })
  });

  let uvi_stream = build_stream(&values, U128_MAX_LEN, |v, buf| {
    let mut small = uvi_encode::u128_buffer();
    let written = uvi_encode::u128(v, &mut small);
    buf[..written.len()].copy_from_slice(written);
    written.len()
  });
  group.bench_function("unsigned-varint", |b| {
    b.iter(|| {
      sweep_decode(&uvi_stream, MIXED_LEN, |buf| {
        let (v, rest) = uvi_decode::u128(buf).unwrap();
        (buf.len() - rest.len(), v)
      })
    })
  });

  group.finish();
}

// ---------------------------------------------------------------------------------------
// i64, zigzag + LEB128 (leb128's `signed` is true signed-LEB128, a different scheme --
// deliberately excluded here; see module docs)
// ---------------------------------------------------------------------------------------

fn bench_encode_i64(c: &mut Criterion) {
  const CASES: [(&str, i64); 3] = [("small", -5), ("mid", -1_000_000_000), ("max", i64::MAX)];

  for (label, value) in CASES {
    let mut group = c.benchmark_group(format!("encode/i64/{label}"));

    let mut buf = [0u8; I64_MAX_LEN];
    group.bench_function("varing", |b| {
      b.iter(|| {
        let n = varing::encode_i64_varint_to(black_box(value), &mut buf).unwrap();
        black_box(&buf[..n.get()]);
      })
    });

    let mut buf = [0u8; I64_MAX_LEN];
    group.bench_function("integer-encoding", |b| {
      b.iter(|| {
        let n = black_box(value).encode_var(&mut buf);
        black_box(&buf[..n]);
      })
    });

    group.finish();
  }

  let values = mixed_i64();
  let mut group = c.benchmark_group("encode/i64/mixed");

  let mut scratch = vec![0u8; MIXED_LEN * I64_MAX_LEN];
  group.bench_function("varing", |b| {
    b.iter(|| {
      let n = sweep_encode(&values, &mut scratch, |v, buf| {
        varing::encode_i64_varint_to(v, buf).unwrap().get()
      });
      black_box(&scratch[..n]);
    })
  });

  let mut scratch = vec![0u8; MIXED_LEN * I64_MAX_LEN];
  group.bench_function("integer-encoding", |b| {
    b.iter(|| {
      let n = sweep_encode(&values, &mut scratch, |v, buf| v.encode_var(buf));
      black_box(&scratch[..n]);
    })
  });

  group.finish();
}

fn bench_decode_i64(c: &mut Criterion) {
  const CASES: [(&str, i64); 3] = [("small", -5), ("mid", -1_000_000_000), ("max", i64::MAX)];

  for (label, value) in CASES {
    let mut group = c.benchmark_group(format!("decode/i64/{label}"));

    let mut buf = [0u8; I64_MAX_LEN];
    let n = varing::encode_i64_varint_to(value, &mut buf).unwrap();
    let encoded = &buf[..n.get()];

    group.bench_function("varing", |b| {
      b.iter(|| {
        let (_, v) = varing::decode_i64_varint(black_box(encoded)).unwrap();
        black_box(v);
      })
    });

    group.bench_function("integer-encoding", |b| {
      b.iter(|| {
        let (v, _) = <i64 as VarInt>::decode_var(black_box(encoded)).unwrap();
        black_box(v);
      })
    });

    group.finish();
  }

  let values = mixed_i64();
  let mut group = c.benchmark_group("decode/i64/mixed");

  let varing_stream = build_stream(&values, I64_MAX_LEN, |v, buf| {
    varing::encode_i64_varint_to(v, buf).unwrap().get()
  });
  group.bench_function("varing", |b| {
    b.iter(|| {
      sweep_decode(&varing_stream, MIXED_LEN, |buf| {
        let (n, v) = varing::decode_i64_varint(buf).unwrap();
        (n.get(), v)
      })
    })
  });

  let ie_stream = build_stream(&values, I64_MAX_LEN, |v, buf| v.encode_var(buf));
  group.bench_function("integer-encoding", |b| {
    b.iter(|| {
      sweep_decode(&ie_stream, MIXED_LEN, |buf| {
        let (v, n) = <i64 as VarInt>::decode_var(buf).unwrap();
        (n, v)
      })
    })
  });

  group.finish();
}

criterion_group!(
  benches,
  bench_encode_u16,
  bench_decode_u16,
  bench_encode_u32,
  bench_decode_u32,
  bench_encode_u64,
  bench_decode_u64,
  bench_encode_u128,
  bench_decode_u128,
  bench_encode_i64,
  bench_decode_i64,
);
criterion_main!(benches);
