# cbor4ii — evidence notes

| Field | Value |
|---|---|
| Released version | 1.2.4 (2026-10-19). Source: https://static.crates.io/crates/cbor4ii/cbor4ii-1.2.4.crate (`.cargo_vcs_info.json` sha `4e6b37a`) |
| Repository | https://github.com/quininer/cbor4ii, cloned to `vendor/cbor4ii` at `4e6b37a` (`master`, 2026-10-19) |
| Released == HEAD? | yes: the HEAD commit is the release commit `4e6b37a` [read: crate cbor4ii-1.2.4 .cargo_vcs_info.json] |
| License | MIT [read: crate cbor4ii-1.2.4 Cargo.toml.orig:8] |

Citation convention: `[read: crate <name>-<ver> <path>:<line>]` = published tarball; `[read: vendor/<dir>@<sha7> <path>:<line>]` = git checkout; `[ran: <command> → <result>]`.

## Verdict

- **Two layers with different allocation stories.**
  - The `core` module (own `Encode`/`Decode` traits, no derive) is **no_std and no-alloc** with default features.
  - The `serde` module is gated on feature `serde1`, which **forces `alloc`**: `serde1 = ["serde/alloc", "use_alloc"]`, and its error types hold `Box<str>`. A thumbv7m link with no allocator fails [ran].
- **The only declarative route (serde) needs an allocator.** A text-keyed struct into `&mut [u8]` with no allocator needs hand-written `core::enc::Encode` / `core::dec::Decode` impls plus a user-written slice writer, because the crate ships no `&mut [u8]` writer. That works and borrows from the input [ran: `experiments/probes/cbor4ii`, 4/4 tests, thumbv7m link OK].
- Zero-copy `&str` / `&[u8]` decode works in both `core` and `serde` [ran]. Unknown map keys can be skipped without alloc via `core::dec::IgnoredAny`. That skip is recursive but depth-limited at 256 by `SliceReader` [read].
- **RFC 8949 coverage is partial.**
  - Preferred (shortest) integer and length heads: yes.
  - Floats are always written at their declared width, and the decoders reject a narrower float for `f64`/`f32` [ran].
  - No deterministic map ordering; the maintainer declined it as out of scope in #13.
  - No duplicate-key detection in the codec.

## 1. no_std / alloc

- `#![cfg_attr(not(feature = "use_std"), no_std)]`; `extern crate alloc` only when `use_alloc` is on and `use_std` is off [read: crate cbor4ii-1.2.4 src/lib.rs:2-8].
- Features: `use_std = ["use_alloc"]`, `use_alloc = []`, `half-f16 = ["half"]`, `serde1 = ["serde/alloc", "use_alloc"]`. There is no `default` feature list, so a plain dependency is core-only, no_std, no-alloc [read: crate cbor4ii-1.2.4 Cargo.toml.orig:12-16].
- The `serde` module is compiled only with `serde1` [read: crate cbor4ii-1.2.4 src/lib.rs:13-14]. Its error enums carry `Custom(crate::alloc::boxed::Box<str>)` unconditionally [read: crate cbor4ii-1.2.4 src/serde/error.rs:6-9,45-48]. The deserializer imports `alloc::borrow::Cow` [read: crate cbor4ii-1.2.4 src/serde/de.rs:2].
- No-alloc core path links [ran: `cargo build --release --target thumbv7m-none-eabi` in `experiments/probes/cbor4ii` (no_main bin, no `#[global_allocator]`) → Finished; `size` text = 4304 B (informational)].
- serde path needs an allocator [ran: scratchpad `probes/cbor4ii-serde-noalloc`, `cbor4ii = { version = "=1.2.4", default-features = false, features = ["serde1"] }`, `cargo build --release --target thumbv7m-none-eabi` → `error: no global memory allocator found but one is required`].
- [ran: `cargo tree --target thumbv7m-none-eabi -e features` in the scratchpad serde probe → `serde_core feature "alloc"` ← `serde feature "alloc"` ← `cbor4ii feature "serde1"`].
- [ran: `cargo tree --target thumbv7m-none-eabi -e features` in `experiments/probes/cbor4ii` → only `cbor4ii v1.2.4`, no features, no dependencies].
- CI checks `cargo check --all` (default = core only) on the host. There is no embedded-target job [read: vendor/cbor4ii@4e6b37a .github/workflows/ci.yml:22-26].

## 2. API model

- **core**: own traits `enc::Encode` (`fn encode<W: Write>(&self, &mut W)`) and `dec::Decode<'de>` (`fn decode<R: Read<'de>>(&mut R)`). Marker newtypes `types::{Bytes, Map, Array, Tag, Simple, Null, Undefined, F16, Maybe, Negative, UncheckedStr}` select the CBOR shape [read: crate cbor4ii-1.2.4 src/core/enc.rs:14-25], [read: crate cbor4ii-1.2.4 src/core/dec.rs:17-64], [read: crate cbor4ii-1.2.4 src/core/types.rs:1-25].
  - There is **no derive macro**. Issue #46 notes the maintainer "tried to make a derive proc macro" and found design problems, and says 2.0.0 is not imminent [read: https://github.com/quininer/cbor4ii/issues/46].
  - There is also a dynamic `Value` (alloc) and a `RawValue<'de>` [read: crate cbor4ii-1.2.4 src/core.rs:48-61], [read: crate cbor4ii-1.2.4 src/core/raw_value.rs:4-5].
- **serde**: `serde::{Serializer, Deserializer, to_vec, from_slice}` (+ `to_writer`/`from_reader` with `use_std`). `to_vec` requires `Vec`; `Serializer::new` is generic over any `enc::Write` [read: crate cbor4ii-1.2.4 src/serde.rs:28-91], [read: crate cbor4ii-1.2.4 src/serde/ser.rs:8-15].

## 3. Derive and mapping

- **Text-string keys**:
  - serde: yes, struct field names become text keys; `serialize_struct` → definite map [read: crate cbor4ii-1.2.4 src/serde/ser.rs:236-242], [read: crate cbor4ii-1.2.4 src/serde/ser.rs:392-407]. [ran: scratchpad `probes/cbor4ii-serde-host` → `a3 63 6f6666 00 64 64617461 42 0102 63 6c656e f6`].
  - core: hand-written only [ran: `experiments/probes/cbor4ii`].
- **Integer keys**: there is no struct-level mechanism in serde (field names are strings). core can do it by hand. No derive.
- **Struct as array**: serde tuple structs → array [read: crate cbor4ii-1.2.4 src/serde/ser.rs:192-205]. There is no packed or array mode for named structs; the README says packed mode is not supported [read: crate cbor4ii-1.2.4 README.md:29-31].
- **Enum representations (serde)**:
  - unit variant → text string;
  - newtype, tuple, and struct variants → 1-entry map `{variant: value}` (externally tagged);
  - unit `()` → empty array, not null [read: crate cbor4ii-1.2.4 src/serde/ser.rs:132-135,145-175,244-256], [read: crate cbor4ii-1.2.4 README.md:26-28].
  - `deserialize_any` does not support untagged enums with tags [read: crate cbor4ii-1.2.4 src/serde/de.rs:78-84].
- **`None` omitted on encode**: no by default; `None` → `null` [read: crate cbor4ii-1.2.4 src/serde/ser.rs:119-122]. With serde's `skip_serializing_if`, serde_derive passes the reduced `len`, so the map stays definite [ran: same host probe → map header `a3` with the skipped field absent].
  - On decode, `null` and `undefined` both map to `None` [read: crate cbor4ii-1.2.4 src/core/dec.rs:750-760], [read: crate cbor4ii-1.2.4 src/serde/de.rs:175-183].
- **Unknown keys ignored on decode**:
  - serde: yes, `deserialize_ignored_any` → `IgnoredAny` [read: crate cbor4ii-1.2.4 src/serde/de.rs:307-313]. [ran: host probe, extra key `"x": [1,2]` → `Ok(..)`].
  - core: up to the hand-written impl; `IgnoredAny` is no-alloc [read: crate cbor4ii-1.2.4 src/core/dec.rs:851-924]. [ran: `experiments/probes/cbor4ii` test `decode_ignores_unknown_key_and_accepts_any_key_order` → ok].
- **Renames**: via serde attributes (`#[serde(rename)]`); nothing crate-specific.

## 4. Zero-copy decode

- core: `Decode for &'de str` and `types::Bytes<&'de [u8]>` borrow through `Reference::Long`. They fail with `RequireBorrowed` if the reader cannot lend, and they reject indefinite-length strings (definite only) [read: crate cbor4ii-1.2.4 src/core/dec.rs:427-444,541-547,567-574].
- serde: `deserialize_bytes`/`deserialize_str` call `visit_borrowed_bytes`/`visit_borrowed_str` when the input is a slice. They fall back to owned `Cow` for indefinite strings [read: crate cbor4ii-1.2.4 src/serde/de.rs:139-165].
- [ran: host probe `ptr::eq(decoded.data.as_ptr(), input[12..].as_ptr())` → true]; [ran: `experiments/probes/cbor4ii` test `decodes_borrowing_from_input` → ok].

## 5. Encode into `&mut [u8]` / decode from `&[u8]` without an allocator

- Decode: `core::utils::SliceReader::new(&[u8])` is no-alloc, with a depth limit of 256 [read: crate cbor4ii-1.2.4 src/core/utils.rs:46-87].
- Encode: **the crate provides no slice writer**; only `BufWriter(Vec<u8>)` (alloc) and `IoWriter` (std) exist [read: crate cbor4ii-1.2.4 src/core/utils.rs:7-44,89-112]. Implementing `enc::Write` for a `&mut [u8]` cursor takes about 12 lines (see probe `SliceWriter`).
- Working no-alloc encode and decode of the text-keyed struct via hand-written core impls [ran: `cargo test --lib` in `experiments/probes/cbor4ii` → 4 passed; thumbv7m release link OK].
- serde over a slice writer is not possible without alloc, because `serde1` forces alloc (§1).

## 6. RFC 8949 coverage

| Item | Encode | Decode | Evidence |
|---|---|---|---|
| Indefinite-length items | yes (`Array/Map/Bytes/UncheckedStr::unbounded` + `end`) | arrays/maps yes (`Map::len` → `None`); indefinite bytes/str only into owned types (alloc); `&str`/`&[u8]` reject them | [read: crate cbor4ii-1.2.4 src/core/enc.rs:265-300], [read: crate cbor4ii-1.2.4 src/core/dec.rs:412-424,427-444,474-508] |
| Half-precision (f16) | yes: raw `types::F16(u16)`, or `half::f16` with `half-f16` | yes (same); serde `deserialize_any` visits f16 only with `half-f16` | [read: crate cbor4ii-1.2.4 src/core/enc.rs:351-370], [read: crate cbor4ii-1.2.4 src/core/dec.rs:775-799], [read: crate cbor4ii-1.2.4 src/serde/de.rs:99-103] |
| Preferred (shortest) integer/length heads | yes (`TypeNum` narrows u64→u32→u16→u8) | accepts non-preferred | [read: crate cbor4ii-1.2.4 src/core/enc.rs:55-109]; [ran: host probe `from_slice::<u32>(&[0x1a,0,0,0,5])` → `Ok(5)`] |
| Preferred (shortest) floats | **no**: f32 → `fa`, f64 → `fb`, always | **strict width**: `f64::decode` accepts only `0xfb`, `f32::decode` only `0xfa` | [read: crate cbor4ii-1.2.4 src/core/enc.rs:372-390], [read: crate cbor4ii-1.2.4 src/core/dec.rs:801-833]; [ran: `to_vec(&1.5f64)` → `fb 3ff8…`; `from_slice::<f64>(fa…)` → `Mismatch{found:250}`; `from_slice::<f64>(f9 3e00)` → `Mismatch{found:249}`] |
| Tags | yes (`types::Tag<T>`) in core; serde cannot serialize `Value::Tag` | core yes; serde `deserialize_any` handles only tags 2/3 | [read: crate cbor4ii-1.2.4 src/core/enc.rs:302-308], [read: crate cbor4ii-1.2.4 src/core/dec.rs:713-725], [read: crate cbor4ii-1.2.4 src/core.rs:175-176], [read: crate cbor4ii-1.2.4 src/serde/de.rs:78-84] |
| Simple values incl. undefined | yes (`types::Simple`, `types::Undefined`) | `types::Simple` (does not reject the non-well-formed `f8 <0x20` form); `undefined` decodes as `None`/`Null`; `IgnoredAny` errors on simple values other than false/true/null/undefined/floats | [read: crate cbor4ii-1.2.4 src/core/enc.rs:310-342], [read: crate cbor4ii-1.2.4 src/core/dec.rs:184-193,727-733,750-760,905-917] |
| Bignums (tags 2/3) | `u128`/`i128` beyond 64 bits → tag 2/3 + byte string | `u128`/`i128` accept tag 2/3 (≤16 bytes) | [read: crate cbor4ii-1.2.4 src/core/enc.rs:119-153], [read: crate cbor4ii-1.2.4 src/core/dec.rs:344-398]; [ran: `to_vec(&(u64::MAX as u128 + 1))` → `c2 49 01 00…`]. Note: the README says bignum "will not be implemented" [read: crate cbor4ii-1.2.4 README.md:15-17], which presumably refers to arbitrary-precision bignums |
| Deterministic encoding | **none**: map entries are written in iteration order; no §4.2.1 or §4.2.3 mode | n/a | [read: crate cbor4ii-1.2.4 src/core/enc.rs:253-263]; the maintainer called RFC 7049 canonical ordering "a bit beyond the scope of this crate" [read: https://github.com/quininer/cbor4ii/issues/13] |
| Duplicate-key detection | n/a | **none** in the codec. serde_derive rejects duplicate *known* fields itself (`duplicate field`), not cbor4ii | [ran: host probe `{"off":0,"off":1}` → `Err(Custom("duplicate field \`off\`"))`]; no check in `types::Map` decode [read: crate cbor4ii-1.2.4 src/core/dec.rs:679-709] |
| Nesting-depth limit / recursion safety | n/a | yes: `Read::step_in/step_out`, limit 256 in `SliceReader`/`IoReader`. Used by `Value`, `Vec`, `Map`, `IgnoredAny`, and the serde deserializer. Hand-written core impls must call it themselves | [read: crate cbor4ii-1.2.4 src/core/dec.rs:32-47], [read: crate cbor4ii-1.2.4 src/core/utils.rs:54,73-86], [read: crate cbor4ii-1.2.4 src/serde/de.rs:25-32]; [ran: 300 nested arrays → `Err(DepthOverflow{name:"any"})`] |

The crate's own tests include the RFC 8949 Appendix A encode vectors [read: crate cbor4ii-1.2.4 src/core/enc.rs:415-543].

## 7. RFC claims in docs

- "see [rfc8949]" [read: crate cbor4ii-1.2.4 README.md:11].
- "The `core` mod should be fully compatible with rfc8949, but some extensions will not be implemented in this crate, such as `datetime`, `bignum`, `bigfloat`." [read: crate cbor4ii-1.2.4 README.md:15-17].
- "The `serde` mod defines how Rust types should be expressed in CBOR, which is not any standard" [read: crate cbor4ii-1.2.4 README.md:19-21].
- The README says it is fuzz tested and "should not crash or panic during the decoding process" [read: crate cbor4ii-1.2.4 README.md:43-47].

## 8. Maintenance

- Last commit: `4e6b37a` 2026-10-19, quininer, "fix: inf-bytes and string decode break byte fail (#61)". It touches `src/` [ran: `git log -1` in vendor/cbor4ii].
- Last release: 1.2.4 on 2026-10-19. Previous releases: 1.2.3 (2026-09-07), 1.2.2 (2025-11-30) [ran: crates.io API]. ~6.7 M total downloads [ran: crates.io API].
- GitHub: 90 stars, not archived, pushed 2026-10-19. 7 open issues, 0 open PRs [ran: `gh api repos/quininer/cbor4ii`; `gh pr list`; `gh issue list`].
- Contributors in the last 12 months: quininer 6, Volker Mische 2, David R. MacIver 1. There are no bot commits [ran: `git shortlog -sne --since="12 months ago" HEAD`]. That is effectively one maintainer.
- `unsafe`: 0 occurrences in `src/` [ran: `grep -rn unsafe src | wc -l` on the tarball → 0].

## 9. Embedded-relevant open issues

- https://github.com/quininer/cbor4ii/issues/46 — "Prepare 2.0.0". The maintainer lists API flaws found while attempting a derive macro, and says 2.0.0 won't "happen anytime soon".
- None open for no_std, alloc, heapless, embedded, or size [ran: `gh search issues --repo quininer/cbor4ii` for each keyword → no open hits].
- Closed and relevant: https://github.com/quininer/cbor4ii/issues/13 — "Support for RFC-7049 Canonical CBOR key ordering", declined as out of scope; the maintainer suggested a wrapper crate.

## Probe

- `experiments/probes/cbor4ii/`: hand-written `core::enc::Encode` / `core::dec::Decode` for `Chunk<'a> { off: u32, data: &'a [u8], len: u32, sha: &'a [u8] }`, plus a 12-line `SliceWriter` implementing `enc::Write`. This is **not declarative**: cbor4ii has no derive, and its serde path needs alloc.
- Dependency line: `cbor4ii = { version = "=1.2.4", default-features = false }`.
- Host tests [ran: `cargo test --lib` → 4 passed]: exact 60-byte encoding, borrowed decode (`ptr::eq`), unknown key `"upgrade": true` skipped plus any key order, short-buffer error.
- Embedded build [ran: `cargo build --release --target thumbv7m-none-eabi` → Finished, no allocator; text = 4304 B (informational)].
- Lints [ran: `cargo fmt`; `cargo clippy --target thumbv7m-none-eabi -- -D warnings` → clean; `cargo clippy --lib --tests -- -D warnings` → clean].
- Negative probe, scratchpad only: `probes/cbor4ii-serde-noalloc` → "no global memory allocator found".
- Host behaviour probe, scratchpad only: `probes/cbor4ii-serde-host` (std).
