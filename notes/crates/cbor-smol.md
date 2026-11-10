# cbor-smol — evidence notes

| Field | Value |
|---|---|
| Released version | 0.5.1 (crates.io 2025-10-01; CHANGELOG says 2025-09-26) — source: https://static.crates.io/crates/cbor-smol/cbor-smol-0.5.1.crate (`.cargo_vcs_info.json` sha `72242c9`) |
| Repository | https://github.com/trussed-dev/cbor-smol — cloned to `vendor/cbor-smol` at `72242c9` (main, 2025-10-01) |
| Released == HEAD? | yes — HEAD is the release commit; `diff -r` tarball `src/` vs `vendor/cbor-smol/src` is empty [ran] |
| License | Apache-2.0 OR MIT |

Citation convention: `[read: crate cbor-smol-0.5.1 <path>:<line>]` = published tarball; `[read: vendor/cbor-smol@72242c9 <path>:<line>]` = git checkout; `[ran: <command> → <result>]`. "Behaviour harness" = scratch crate `probes/serde-family-behaviour` (host, `cbor-smol = "=0.5.1"`); inputs quoted inline.

## Verdict

- Aimed at embedded (crates.io categories `development-tools`, `embedded`; lives in the trussed-dev org) [read: crate cbor-smol-0.5.1 Cargo.toml.orig]: `no_std` with **no alloc at all** (no `alloc`/`std` feature exists), serde-based, writes into `&mut [u8]` or `heapless` buffers [read: vendor/cbor-smol@72242c9 src/lib.rs:1; Cargo.toml `[features]`]; [ran: thumbv7m no_main bin links, 4/4 tests].
- Smallest of the serde-family probes by far [ran: thumbv7m text 3749 B vs serde_cbor 17810 B, same `Chunk`] — informational only, not a benchmark.
- Deliberately partial CBOR: floats unimplemented (encode **panics** via `todo!`), no tags, no undefined, no bignums, no indefinite-length decode, no `deserialize_any` [read + ran]. Decoder is strict about preferred integer heads (rejects non-minimal) [ran].
- Safety/robustness gaps: unknown values are skipped by unbounded recursion — 10,000 nested arrays in an unknown field overflowed a 64 KiB stack [ran]; `collect_str` is `unreachable!()` [read].
- Does not compile for thumbv6m (no atomics) because of the mandatory `delog` dependency [ran]; open issue #24. Last commit 2025-10-01; no commits in the last 12 months [ran].

## 1. no_std / alloc

- `#![cfg_attr(not(test), no_std)]` unconditionally; no `extern crate alloc` anywhere [read: vendor/cbor-smol@72242c9 src/lib.rs:1]; [ran: `grep -rn "extern crate alloc" src` → none].
- Features are only `heapless-*` / `heapless-bytes-*` writer back-ends and `log-*` levels; there is no `std` or `alloc` feature [read: crate cbor-smol-0.5.1 Cargo.toml.orig `[features]`].
- Dependencies: `serde_core` (default-features = false), `delog` (mandatory), optional `heapless` 0.7/0.8/0.9, `heapless-bytes` 0.3/0.4/0.5 [read: crate cbor-smol-0.5.1 Cargo.toml.orig `[dependencies]`].
- No-alloc verified: [ran: `cargo build --release --target thumbv7m-none-eabi` in `experiments/probes/cbor-smol` (no_main bin, no global allocator) → Finished].
- thumbv6m fails: [ran: `cargo build --release --target thumbv6m-none-eabi` → `error[E0599]: no method named fetch_add found for reference &'static AtomicUsize` … `could not compile delog`]. Matches open issue #24.
- `cargo tree --target thumbv7m-none-eabi -e features`: `cbor-smol v0.5.1 → serde_core v1.0.229`, `delog feature "default" → delog v0.1.8 → log v0.4.35` [ran].

## 2. API model

- serde `Serializer`/`Deserializer` only (via `serde_core`), with free functions `cbor_serialize_to(&T, W: Writer) -> usize`, `cbor_serialize(&T, &mut [u8]) -> &[u8]`, `cbor_deserialize(&[u8]) -> T` [read: vendor/cbor-smol@72242c9 src/lib.rs:20-42].
- Own `Writer` trait implemented for `&mut [u8]` and (feature-gated) `heapless::Vec<u8, N>` / `heapless_bytes::Bytes<N>` [read: vendor/cbor-smol@72242c9 src/ser.rs:11-92].
- No `Value` type; `deserialize_any` returns `Error::WontImplement` ("If you need this, use `serde_cbor`") [read: vendor/cbor-smol@72242c9 src/de.rs:413-421], so self-describing decode, `#[serde(untagged)]`, internally tagged enums and `flatten` are unavailable.

## 3. Derive and mapping

- Text-string keys: yes — struct fields always written as text keys [read: vendor/cbor-smol@72242c9 src/ser.rs:422-425,528-534]; [ran: probe `encodes_text_keyed_map_into_slice` → exact 60-byte vector].
- Integer keys: encode — no (always text); decode — an unsigned-integer key is accepted as the serde field ordinal [read: vendor/cbor-smol@72242c9 src/de.rs:797-814]; [ran: harness `smol_integer_field_key_maps_by_ordinal`: `a1 00 07` → `One { off: 7 }`]. Arbitrary integer keys (CTAP style) are not provided by this crate.
- Struct as array: no option (tuple structs only, serde model).
- Enum representations: fixed "packed" style — unit variant → variant index integer; newtype → `[index, value]`; tuple → `[index, fields…]`; struct variant → `[index, {map}]` [read: vendor/cbor-smol@72242c9 src/ser.rs:330-338,357-372,398-415,428-441]; [ran: harness `smol_unit_variant_as_index`: `Unit::B` → `01`]. Decoder accepts array or uint forms only [read: src/de.rs:770-795].
- `None` omitted on encode: with `#[serde(skip_serializing_if = "Option::is_none")]`; definite map length adjusted [ran: harness `smol_none_skipped_definite_len`: `{a:1,b:None}` → `a1 61 61 01`]. Without it `None` → `f6` [read: src/ser.rs:307-310].
- Unknown keys ignored on decode: yes, via `deserialize_ignored_any` → `ignore()` (added in 0.4.1) [read: vendor/cbor-smol@72242c9 src/de.rs:818-825; CHANGELOG.md 0.4.1]; [ran: probe `decode_ignores_unknown_key_and_accepts_any_key_order` → ok].
- Renames: serde attributes.

## 4. Zero-copy decode

- Yes: `visit_borrowed_bytes` / `visit_borrowed_str` straight from the input slice [read: vendor/cbor-smol@72242c9 src/de.rs:603-607,620-628]; [ran: probe `decodes_borrowing_from_input` pointer-equality assert → ok].
- `deserialize_bytes` also accepts a CBOR array of integers (copied via `visit_seq`) [read: src/de.rs:596-602].

## 5. Encode into `&mut [u8]` / decode from `&[u8]` without an allocator

- Yes — this is the crate's primary mode: `cbor_smol::cbor_serialize_to(chunk, buf)` returns bytes written; `cbor_smol::cbor_deserialize(buf)` [read: vendor/cbor-smol@72242c9 src/lib.rs:20-42]. Overflow → `Error::SerializeBufferFull` [read: src/ser.rs:19-31]; [ran: probe `encode_reports_short_buffer` → ok].
- `cbor_deserialize` does not check for trailing bytes (`take_from_bytes` returns the remainder) [read: src/de.rs:12-30].
- [ran: `experiments/probes/cbor-smol`: `cargo test --lib` → 4 passed; thumbv7m release build → Finished; `cargo clippy --target thumbv7m-none-eabi -- -D warnings` → clean; `cargo clippy --lib --tests -- -D warnings` → clean; `cargo fmt --check` → clean].
- Informational only: [ran: `size` thumbv7m release (opt-level z, LTO) → text 3749 B]. `nm` shows `deserialize_map`, `ignore`, `raw_deserialize_u32`, `deserialize_bytes` present, so the codec was instantiated [ran].

## 6. RFC 8949 coverage

| Item | Encode | Decode | Evidence |
|---|---|---|---|
| Indefinite-length items | yes, when serde gives no length | **no** (lengths parsed with `raw_deserialize_u32`; AI 31 → `DeserializeBadU32`) | [read: vendor/cbor-smol@72242c9 src/ser.rs:180-199,575-583; src/de.rs:150]; [ran: harness `smol_indefinite_seq_encode` → `9f 01 02 03 ff`; `smol_indefinite_map_decode_rejected`: `bf 63 6f 66 66 00 ff` → error] |
| Half-precision (f16) | no | no | [read: src/ser.rs:279-285; src/de.rs:562-574] |
| Preferred (shortest) integer/length heads | yes | **enforced**: non-minimal heads rejected with `DeserializeNonMinimal` | [read: src/ser.rs:136-178; src/de.rs:103,125,135,146,159-191]; [ran: harness `smol_non_minimal_head_rejected`: `{"off": 18 00}` → `DeserializeNonMinimal`] |
| Preferred (shortest) floats | n/a — `serialize_f32`/`f64` are `todo!()` (panic) | `NotYetImplemented` | [read: src/ser.rs:280,284; src/de.rs:566,573]; [ran: harness `smol_float_encode_panics` (should_panic "serialize_f32 not implemented") → ok; `smol_float_decode_unsupported` → error] |
| Tags | no | no for typed values; skipped inside unknown fields | [read: src/de.rs:265-268]; [ran: harness `smol_tagged_value_rejected_for_typed_field`: `c2 41 01` → error; `smol_tagged_unknown_value_skipped` → ok] |
| Simple values incl. undefined | false/true/null only | false/true/null only; undefined constant is commented out | [read: src/consts.rs:12-20; src/de.rs:638-650]; [ran: harness `smol_undefined_not_none`: `{"v": f7}` into `Option<u32>` → error] |
| Bignums (tags 2/3) | no | no | no tag support at all [read: src/ser.rs:344-355 (tag code commented out)] |
| Deterministic encoding | none (declaration order); no sorting code | no check on key order | [ran: `grep -n -i "canonical\|sort\|determin" src/*.rs` → no matches]. Preferred-head enforcement on decode is the only deterministic-encoding-related check |
| Duplicate-key detection | n/a | codec no; serde derive rejects a repeated known field | [ran: harness `smol_duplicate_key_rejected_by_serde_derive`: `{"off":0,"off":1}` → `SerdeDeCustom`] |
| Nesting-depth limit / recursion safety | n/a | **none**: `ignore()` recurses per nesting level with no counter | [read: src/de.rs:226-270]; [ran: `smol_deep` (release, x86_64, 64 KiB thread stack), nested arrays inside an unknown field: depth 1000 → Ok, depth 10000 → "fatal runtime error: stack overflow, aborting"] |

Other limits: byte/text/array/map lengths capped at u32 (AI 27 rejected) [read: src/de.rs:150]; `collect_str` is `unreachable!()` so any type serialized through `Display` panics [read: src/ser.rs:444-449]; `char` decode `NotYetImplemented` [read: src/de.rs:576-583].

## 7. RFC claims in docs

- None. README contains only badges and license text [read: vendor/cbor-smol@72242c9 README.md:1-21]; [ran: `grep -rn -i "rfc\|7049\|8949"` over `*.rs *.md *.toml` → no matches]. Crate description: "Streamlined serde serializer/deserializer for CBOR" [read: crate cbor-smol-0.5.1 Cargo.toml.orig].

## 8. Maintenance

- Last commit (and last `src/` commit): `72242c9` / `32b8ba7`, 2025-10-01 (Sosthène Guédon) [ran: `git log -1`, `git log -1 -- src`].
- Last release: 0.5.1, 2025-10-01 (GitHub release v0.5.1 same day); previous 0.5.0 2024-10-21 [ran: crates.io API; `gh release list`].
- Open issues 1, open PRs 0 [ran: `gh api repos/trussed-dev/cbor-smol` → open_issues_count 1; `gh pr list` → 0].
- Maintainers in last 12 months: 0 commits [ran: `git shortlog -sn --since="12 months ago"` → empty]; last 24 months: Sosthène Guédon 4 non-merge commits [ran]. Org: trussed-dev.
- `unsafe` in `src/`: 0 [ran: `grep -rnw unsafe src` → 0].
- Usage: 100,867 downloads total, 32,808 recent; 10 reverse dependencies [ran: crates.io API].

## 9. Embedded-relevant open issues

- https://github.com/trussed-dev/cbor-smol/issues/24 — "Compilation issues for thumbv6 targets": `delog` needs atomics; reporter asks to feature-gate it (reproduced above).
- Closed for context: https://github.com/trussed-dev/cbor-smol/pull/6 "Add support for `deserialize_ignored_any`" (unknown-key skipping, 0.4.1); https://github.com/trussed-dev/cbor-smol/pull/13 multi-version heapless support; https://github.com/trussed-dev/cbor-smol/pull/15 / #16 / #17 "Never inline map … deserialization" (code-size tuning, one reverted).

## Probe

- `experiments/probes/cbor-smol/` (same `Chunk`/`sample`/60-byte vector/4 tests as `minicbor-serde`; no type adaptations needed).
- Dependency line: `cbor-smol = { version = "=0.5.1", default-features = false }` (no features needed; `default-features = false` is a no-op) plus `serde = { version = "1", default-features = false, features = ["derive"] }`, `serde_bytes = { version = "0.11", default-features = false }`.
- Results: [ran: `cargo test --lib` → 4 passed, 0 ignored]; [ran: thumbv7m release → Finished; thumbv6m → fails in `delog`]; [ran: fmt + both clippy invocations → clean]. Toolchain rustc 1.95.0.
