# Rust CBOR crates for embedded use: feature matrix

Scope: whether a codec exists that is `no_std` (ideally without `alloc`), small in flash, type-safe and declarative. Conformance is measured against **RFC 8949**, which obsoletes RFC 7049 with an identical wire format (8949 Appendix G, `sources/rfc/rfc8949.txt:3485-3491`). The primary deterministic mode is 8949 §4.2.1 bytewise key order; 7049-style length-first order (8949 §4.2.3) is a legacy extra.

Each row is backed by `notes/crates/<crate>.md`, where every claim carries a `[read: …]` or `[ran: …]` tag. Versions are the latest crates.io releases as of 2026-11-10.

## Executive summary

- **Neither premise holds as stated.**
  - (a) ciborium *is* `no_std`, but it requires `alloc`. Its `ciborium-ll` and `ciborium-io` sub-crates need neither `alloc` nor `std`.
  - (b) minicbor targets RFC 8949, which is wire-identical to 7049. It round-trips all 81 Appendix A vectors.
    - It has one real well-formedness bug: it emits and accepts `f8 14`…`f8 1f` simple values.
    - It omits two optional 8949 features: preferred float serialization and §4.2 deterministic encoding.
- **Declarative + text-keyed + no-alloc + zero-copy exists today only through serde.** Three options qualify:
  - minicbor-serde (maintained);
  - serde_cbor / serde_cbor_2 (archived or passively maintained, about 2× the flash);
  - cbor-smol (smallest, but partial CBOR: float *encode* panics, no tags, no indefinite decode, no thumbv6m).
- **No crate has a non-serde derive that emits text keys.**
  - minicbor-derive keys are integers only.
  - tinycbor-derive keys are integers only.
  - cbor4ii has no derive at all.
  - minicbor has an open maintainer-authored draft PR (#53, `#[s("…")]`) and a contributor PR (#57, `#[cbor(text_keys)]`) for exactly this. That is the most obvious upstream effort to contribute to.
- **Deterministic encoding (8949 §4.2.1) is effectively missing from no-alloc crates.**
  - Only sk-cbor implements it, always on and enforced on decode, and sk-cbor is an alloc `Value`-tree codec.
  - ciborium has only length-first ordering, for `Value` keys.
  - minicbor-derive does sort its *integer* map keys bytewise at compile time.
- **Robustness varies widely.** Fixed recursion limits:
  - serde_cbor: 128
  - ciborium: 256
  - cbor4ii: 256
  - sk-cbor: 127

  Unbounded recursion on hostile input, shown in our runs: cbor-smol, cbor_event, facet-cbor. minicbor's derived and serde `Decode` paths are unbounded too (stated in upstream PR #71).
- **Latest-release breakage.** tinycbor 0.14.0 through 0.15.1 do not compile without `alloc`; 0.13.0 does [ran]. facet-cbor is std-only and was deleted from its repository after its last release.

## Matrix

Legend:
- **y** yes, **n** no, **p** partial, **m** possible only with a hand-written impl over the crate's low-level API, **?** unknown, **—** not applicable.
- `a/b` = encode/decode.
- "no-alloc" means a thumbv7m `#![no_main]` binary links with no `#[global_allocator]` [ran].
- Probe text sizes are from `size` (opt-level z, LTO). They are informational only, not a benchmark.
- "maint. 12 mo" = distinct human commit authors in the last 12 months (`git shortlog -sn --since="12 months ago"`, bots excluded, path-scoped for monorepos). minicbor has 6 authors, of whom 1 is the maintainer.

| Crate (ver) | no_std | API | Text-key struct | Int keys / array | `None` omit / unknown keys skipped | Zero-copy | `&mut [u8]` ↔ `&[u8]`, no alloc [ran] | Indef. | f16 / shortest float | Tags / undef / bignum | Deterministic order | Dup keys / depth limit | RFC cited | Last release / last human src commit | Issues/PRs · maint. 12 mo · `unsafe` |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| minicbor 2.3.1 (+derive 0.19.5) | **no-alloc** | own traits + derive | n (derive is int-only; PRs #53/#57 open) / m | y (i64) / y (default) | y / y¹ | y | m, 6.2 KB | y/y | y/y (feature `half`) / **n** | y / y / y (u128) | none (derive int keys sorted per §4.2.1) | n (last wins) / none² | 8949 | 2026-10-18 / 2026-10-07 | 5/5 · 6 (1 maintainer) · 6 |
| minicbor-serde 0.7.2 | **no-alloc** | serde | **y** | n / serde tuple | y (`skip_serializing_if`) / y¹ | y | **y, 8.3 KB** | y/y | n/y (feature `half`) / **n** | p (`tag` module; bignums) / n (`deserialize_any` rejects) / y | none | serde-derive / none | 8949 | 2026-10-18 / 2026-10-07 | (same repo) |
| ciborium 0.2.2 | alloc | serde | y | n / n | y (serde) / y | **n** | **n** (allocator link error) | y/y | y/y / **y** | y / dec only / y | length-first §4.2.3, `CanonicalValue` keys only | serde-derive / 256 | none (ll: 7049) | 2024-01-24 / 2025-08-01 | 46/11 · 1 · 0 |
| ciborium-ll 0.2.2 | **no-alloc** | header/token | m | m | m | n (copies) | m, 4.5 KB | y/y | y/y / **y** | y / y / tag consts | none | none / — | 7049 | (same) | (same) |
| serde_cbor 0.11.2 | **no-alloc** (v7m, v6m) | serde | **y** | packed ordinals / tuple | y / y | y | **y, 17.8 KB** | y/y | y/y / **y** | std-only / dec→unit / n | none (std `Value`: §4.2.1) | serde-derive / 128 | 7049 | 2021-08-15 / 2020-06-04 (**archived**) | 26/23 · 0 · 0 |
| serde_cbor_2 0.13.0 | **no-alloc** | serde | **y** | as serde_cbor | y / y | y | **y, 17.8 KB** | y/y | y/y / **y** | as serde_cbor | as serde_cbor | serde-derive / 128 | 7049 | 2025-06-14 / 2025-06-14 | 0/3 · 0 (dependabot only) · 0 |
| cbor-smol 0.5.1 | **no-alloc** (v7m only; **v6m fails**) | serde | **y** | dec accepts ordinal / tuple | y / y | y | **y, 3.7 KB** | y/**n** | n/n / floats: **encode panics** | n / n / n | none (decoder rejects non-minimal ints) | serde-derive / **none** (overflow) | none | 2025-10-01 / 2025-10-01 | 1/0 · 0 · 0 |
| cbor4ii 1.2.4 | core **no-alloc**; serde needs alloc | own traits (no derive) + serde | serde only with alloc / m | n / n | serde / y (`IgnoredAny`) | y | m, 4.3 KB | y / p (indefinite strings need alloc) | p (raw `F16` or feature) / **n** | y / y / u128 | none (declined, #13) | n / 256 | 8949 | 2026-10-19 / 2026-10-19 | 7/0 · 1 · 0 |
| tinycbor 0.15.1 (+derive 0.15.4) | **broken** without alloc (0.13.0 OK) | own traits + derive | n (derive u64 keys) / m | y / y | y (`optional`) / **n** (strict) | y (definite only) | m on 0.13.0, 3.5 KB | y/y | y/y / **y** | y / y / raw tag | none (declaration order) | **detected** (derive) / none | 8949 | 2026-07-31 / 2026-07-29 | 0/6 · 1 · 7 |
| sk-cbor 0.2.0 | alloc | `Value` tree + macros | m (macros) | m / — | y / only for canonical input | n | **n** | **n/n** (rejected) | no floats at all | y / y / n | **§4.2.1 bytewise, always on, enforced on decode** | **rejected** / 127 | 8949 | 2026-10-22 / 2026-10-29 | 12/2 repo-wide (1 cbor issue) · 4 · 0 |
| cbor_event 3.3.0 | alloc | token API + own traits | m | m | m (needs `Value`) | n | **n** | y/y | y/y / y (typed) | y / y / n | none | n / **none** (overflow) | 8949 | 2026-08-03 / 2026-08-03 | 3/2 · 2 · 0 |
| facet-cbor 0.3.4 | **std only** | reflection (`Facet`) | y (renames ignored on the wire) | n / n | n (`null`) / y | n | **n** | n/n | n/n / n | n / n / encode only | none | n / **none** (overflow) | 8949 | 2026-05-15 / 2026-04-15 (**removed from repo**) | n/a · 1 · 0 |

Notes:
1. minicbor's `Decoder::skip` without `alloc` errors on an indefinite array or map nested inside a definite one while other items are pending (`[[_ 1], 2]` as an unknown value) [ran: `decode_without_alloc_cannot_skip_nested_indefinite_unknown_value`].
2. `Decoder::skip` is iterative, but derived and serde `Decode` recurse without a limit; PR #71 proposes `max_depth`.
- MSRV: cbor_event requires rustc 1.96.1, newer than the installed 1.95.0.
- RFC 8949 data-model coverage:
  - **Generic, full data model:** minicbor, ciborium(-ll), serde_cbor(_2), cbor4ii, tinycbor, cbor_event. Gaps are per-cell above.
  - **Deliberate subsets:**
    - cbor-smol: no floats, tags or indefinite decode.
    - sk-cbor: CTAP-style canonical subset; no floats or indefinite lengths; strict decoding.
    - facet-cbor: no indefinite lengths, f16, or tag decode.
  - **Docs still cite RFC 7049:** serde_cbor, serde_cbor_2, ciborium-ll.

## Premise check

### (a) "ciborium is not no_std": **false as worded, true in effect for no-alloc targets**

- `ciborium` declares `#![cfg_attr(not(feature = "std"), no_std)]` [read: crate ciborium-0.2.2 src/lib.rs:87]. However, it has an unconditional `extern crate alloc;` [read: same, :93], and it forces `ciborium-io/alloc` and `serde/alloc` [read: crate ciborium-0.2.2 Cargo.toml.orig:25-26].
- With `default-features = false`, a no-allocator thumbv7m binary fails with `error: no global memory allocator found but one is required` [ran: scratchpad `probes/ciborium-noalloc`]. The same binary links once a `#[global_allocator]` is provided [ran: `probes/ciborium-withalloc`].
- The method's control: a bin containing only `extern crate alloc;` produces the same error [ran: `probes/control-alloc`].
- The sources of these three scratch probes are reproduced in `notes/crates/ciborium.md` § Appendix.
- A maintainer confirmed it: "The `ciborium` crate requires `alloc` … The `ciborium-io` and `ciborium-ll` crates already have an `alloc` feature which seems to work as intended" (https://github.com/enarx/ciborium/issues/59, 2024-02-20).
- Issue #90, "no-alloc as option for ciborium" (2023-08), is unanswered. HEAD, 59 commits past the 0.2.2 release, still has the unconditional `extern crate alloc` [read: vendor/ciborium@00279e4 ciborium/src/lib.rs:93].
- `ciborium-ll` ("usable in both `no_std` and `no_alloc` environments" [read: crate ciborium-ll-0.2.2 src/lib.rs:6]) does link without an allocator, and a hand-written text-keyed codec over it passes [ran: `experiments/probes/ciborium-ll`]. But it is a header-level API with no derive and no skip helper, and its decoding copies rather than borrowing.

### (b) "minicbor is not RFC 7049": **only true in name; it implements RFC 8949, which carries the 7049 wire format**

- minicbor's docs cite RFC 8949, not 7049 [read: crate minicbor-2.3.1 README.md:15; src/lib.rs:133; src/decode/decoder.rs:190, 216]. RFC 8949 "formally obsoletes RFC 7049 while keeping full compatibility with the interchange format" [read: sources/rfc/rfc8949.txt:3487-3491].
- minicbor's own test suite is built on "Test vectors of RFC 7049" Appendix A [read: vendor/minicbor@0432e74 minicbor-tests/tests/rfc.rs:6-8]. I did not run it: the sandbox blocked executing repo test code.
- Instead, the crates.io release decodes all **81** RFC 8949 Appendix A examples and re-encodes them byte-identically [ran: `experiments/conformance/appendix-a/minicbor`, `cargo test -- --nocapture` → `vectors=81 failures=0`].
- **One well-formedness defect, which RFC 8949 forbids:** `Encoder::simple(20..=31)` emits two-byte forms (`f8 14` … `f8 1f`), and the decoder accepts `f8 14` and `f8 18`. RFC 8949 §3.3 says: "An encoder MUST NOT issue two-byte sequences that start with 0xf8 … and continue with a byte less than 0x20 … Such sequences are not well-formed" [read: sources/rfc/rfc8949.txt:869-871] [ran: `experiments/conformance/appendix-a/minicbor` asserts both bytes]. RFC 7049's Appendix A had listed `simple(24) = 0xf818`; 8949 dropped that example. `ciborium-ll` also accepts `f8 18` [ran].
- **Optional parts of RFC 8949 that minicbor does not implement.** None of these is a conformance violation:
  - **Preferred serialization of floats.** `Encoder::f64(1.5)` gives `fb 3ff8000000000000` instead of `f9 3e00` [ran]. f16 is available only through the explicit `Encoder::f16`, behind feature `half`, and without `half` the decoder cannot read f16 [read: crate minicbor-2.3.1 src/decode/decoder.rs:288-312]. RFC 8949 §4.1 says the specification "does not put the burden of enforcing this preference on either the encoder or decoder" [read: sources/rfc/rfc8949.txt:1329-1330]. Integer and length heads *are* preferred.
  - **§4.2 deterministic encoding**, which a protocol opts into ("Some protocols may want …" [read: sources/rfc/rfc8949.txt:1374]). There is no key sorting for generic maps (issue #58 is open; the maintainer says "it might take a while") and no 7049 §3.9 canonical mode either. minicbor-derive does sort its *integer* map keys bytewise.
  - **Decode-side strictness.** `1a00000001` decodes as `u8` 1 [ran]. This is correct for a general decoder, since "Full implementations of CBOR decoders are by definition variation tolerant" [read: sources/rfc/rfc8949.txt:1352-1353]. It does mean minicbor cannot *enforce* deterministic input.
- For comparison, `ciborium-ll` decodes all 81 vectors and re-encodes 75 of them byte-identically. The other 6 are the non-preferred f32/f64 Infinity/NaN examples, which it shrinks to f16 by design [ran: `experiments/conformance/appendix-a/ciborium-ll` asserts exactly those six].

## Shortlist for code-size benchmarking

These are the crates that encode and decode the text-keyed struct `{"off": u32, "data": bstr, "len": u32, "sha": bstr}` into `&mut [u8]` and from `&[u8]` on `thumbv7m-none-eabi` with **no allocator**, verified by building a `#![no_std] #![no_main]` binary with no `#[global_allocator]`.

Every probe:
- has a host test asserting the exact 60-byte encoding `a4 63"off" 00 64"data" 44 01020304 63"len" 18 7b 63"sha" 58 20 aa×32`, a borrowed or copying decode, an unknown-key and reordered-key decode, and a short-buffer error;
- also links for `thumbv6m-none-eabi`, except cbor-smol;
- passes `cargo fmt`, plus `cargo clippy -- -D warnings` for the thumbv7m target and for the host tests.

Reproduce with:

```sh
cd experiments/probes/<dir>
cargo test --lib
cargo build --release --target thumbv7m-none-eabi
```

The shared serde struct, used verbatim in probes 1–3:

```rust
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Chunk<'a> {
    pub off: u32,
    #[serde(with = "serde_bytes")]
    pub data: &'a [u8],
    pub len: u32,
    #[serde(with = "serde_bytes")]
    pub sha: &'a [u8],
}
```

`serde_bytes` is needed so `&[u8]` is written as a CBOR byte string rather than an array of u8.

### 1. minicbor-serde 0.7.2: declarative, maintained, zero-copy (`experiments/probes/minicbor-serde/`, text 8268 B)

```toml
minicbor = { version = "=2.3.1", default-features = false }
minicbor-serde = { version = "=0.7.2", default-features = false }
serde = { version = "1", default-features = false, features = ["derive"] }
serde_bytes = { version = "0.11", default-features = false }
```

```rust
pub fn encode_chunk(
    chunk: &Chunk,
    buf: &mut [u8],
) -> Result<usize, minicbor_serde::error::EncodeError<minicbor::encode::write::EndOfSlice>> {
    minicbor_serde::to_slice(chunk, buf)
}

pub fn decode_chunk(buf: &[u8]) -> Result<Chunk<'_>, minicbor_serde::error::DecodeError> {
    minicbor_serde::from_slice(buf)
}
```

### 2. serde_cbor 0.11.2 / serde_cbor_2 0.13.0: declarative, full CBOR data model, archived / passive (`experiments/probes/serde_cbor/`, `experiments/probes/serde_cbor_2/`, text 17810 / 17822 B)

```toml
serde_cbor = { version = "=0.11.2", default-features = false }   # or: serde_cbor_2 = { version = "=0.13.0", default-features = false }
serde = { version = "1", default-features = false, features = ["derive"] }
serde_bytes = { version = "0.11", default-features = false }
```

```rust
pub fn encode_chunk(chunk: &Chunk, buf: &mut [u8]) -> Result<usize, serde_cbor::Error> {
    let mut serializer = serde_cbor::Serializer::new(serde_cbor::ser::SliceWrite::new(buf));
    chunk.serialize(&mut serializer)?;
    Ok(serializer.into_inner().bytes_written())
}

pub fn decode_chunk(buf: &[u8]) -> Result<Chunk<'_>, serde_cbor::Error> {
    serde_cbor::de::from_slice_with_scratch(buf, &mut [])
}
```

serde_cbor_2 is the same code under the crate name `serde_cbor_2` (released `src/` differs only by lint cleanup and `half` 2; see `notes/crates/serde_cbor_2.md`).

### 3. cbor-smol 0.5.1: declarative, smallest, partial CBOR (`experiments/probes/cbor-smol/`, text 3749 B)

```toml
cbor-smol = { version = "=0.5.1", default-features = false }
serde = { version = "1", default-features = false, features = ["derive"] }
serde_bytes = { version = "0.11", default-features = false }
```

```rust
pub fn encode_chunk(chunk: &Chunk, buf: &mut [u8]) -> Result<usize, cbor_smol::Error> {
    cbor_smol::cbor_serialize_to(chunk, buf)
}

pub fn decode_chunk(buf: &[u8]) -> Result<Chunk<'_>, cbor_smol::Error> {
    cbor_smol::cbor_deserialize(buf)
}
```

Caveats:
- `serialize_f32` / `serialize_f64` are `todo!()`, so floats panic.
- It does not build for thumbv6m, because the mandatory `delog` dependency uses atomics [ran].

### 4. minicbor 2.3.1, hand-written `Encode`/`Decode`: no-serde baseline, approximating what a text-key derive would generate (`experiments/probes/minicbor/`, text 6178 B)

```toml
minicbor = { version = "=2.3.1", default-features = false }
```

```rust
impl<C> Encode<C> for Chunk<'_> {
    fn encode<W: Write>(&self, e: &mut Encoder<W>, _: &mut C) -> Result<(), encode::Error<W::Error>> {
        e.map(4)?
            .str("off")?.u32(self.off)?
            .str("data")?.bytes(self.data)?
            .str("len")?.u32(self.len)?
            .str("sha")?.bytes(self.sha)?
            .ok()
    }
}

impl<'b, C> Decode<'b, C> for Chunk<'b> {
    fn decode(d: &mut Decoder<'b>, _: &mut C) -> Result<Self, decode::Error> {
        let entries = d.map()?.ok_or_else(|| decode::Error::message("indefinite map"))?;
        let (off, data, len, sha) = (0..entries).try_fold(
            (None, None, None, None),
            |(off, data, len, sha), _| match d.str()? {
                "off" => Ok((Some(d.u32()?), data, len, sha)),
                "data" => Ok((off, Some(d.bytes()?), len, sha)),
                "len" => Ok((off, data, Some(d.u32()?), sha)),
                "sha" => Ok((off, data, len, Some(d.bytes()?))),
                _ => d.skip().map(|()| (off, data, len, sha)),
            },
        )?;
        Ok(Chunk {
            off: off.ok_or_else(|| decode::Error::missing_value(0))?,
            data: data.ok_or_else(|| decode::Error::missing_value(1))?,
            len: len.ok_or_else(|| decode::Error::missing_value(2))?,
            sha: sha.ok_or_else(|| decode::Error::missing_value(3))?,
        })
    }
}

pub fn encode_chunk(chunk: &Chunk, buf: &mut [u8]) -> Result<usize, encode::Error<EndOfSlice>> {
    let mut cursor = Cursor::new(buf);
    minicbor::encode(chunk, &mut cursor)?;
    Ok(cursor.position())
}

pub fn decode_chunk(buf: &[u8]) -> Result<Chunk<'_>, decode::Error> {
    minicbor::decode(buf)
}
```

The snippet is condensed from the probe. The compiled, rustfmt-formatted version is `experiments/probes/minicbor/src/lib.rs`.

### Also qualify, but only with a hand-written mapping (not declarative)

| Crate | Dependency lines | Probe | text | Caveat |
|---|---|---|---|---|
| cbor4ii 1.2.4 (`core`) | `cbor4ii = { version = "=1.2.4", default-features = false }` | `experiments/probes/cbor4ii/` | 4304 B | User-written `&mut [u8]` writer; its serde route requires alloc |
| tinycbor **0.13.0** | `tinycbor = { version = "=0.13.0", default-features = false }`, `embedded-io = { version = "=0.7.1", default-features = false }` | `experiments/probes/tinycbor/` | 3513 B | 0.14–0.15.1 do not build without alloc; no no-alloc skip, so unknown keys are rejected and that test is `#[ignore]` |
| ciborium-ll 0.2.2 | `ciborium-io = { version = "=0.2.2", default-features = false }`, `ciborium-ll = { version = "=0.2.2", default-features = false }` | `experiments/probes/ciborium-ll/` | 4529 B | Copies into caller scratch; skip loop is probe code |

Not eligible, with the failure shown in each crate note:
- ciborium: needs alloc.
- sk-cbor: needs alloc.
- cbor_event: needs alloc.
- facet-cbor: std-only.

## Evidence index

- Per-crate notes: `notes/crates/{minicbor,ciborium,serde_cbor,serde_cbor_2,cbor-smol,cbor4ii,tinycbor,sk-cbor,cbor_event,facet-cbor}.md`.
- No-alloc probes: `experiments/probes/*/`. Each is a Cargo project with a `no_std` lib, host tests, and a `#![no_main]` thumbv7m bin. `target/` is git-ignored.
- RFC 8949 Appendix A conformance runs: `experiments/conformance/appendix-a/{minicbor,ciborium-ll}/`.
- Scratchpad-only probes (negative and behavioural host checks) are not in the repo. Their commands and results are quoted in the notes, and the ciborium, sk-cbor, cbor_event and facet-cbor notes reproduce their sources inline.
- Upstream checkouts: `vendor/<crate>/`, blobless clones with full history. The commits used are recorded in each note. `vendor/minicbor` is detached at the release commit `0432e74`; the others are on their default-branch HEAD.
- Released sources: `https://static.crates.io/crates/<name>/<name>-<ver>.crate`.
