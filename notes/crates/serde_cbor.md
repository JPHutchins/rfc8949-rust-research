# serde_cbor — evidence notes

| Field | Value |
|---|---|
| Released version | 0.11.2 (2021-08-15) — source: https://static.crates.io/crates/serde_cbor/serde_cbor-0.11.2.crate (`.cargo_vcs_info.json` sha `347a3f0`) |
| Repository | https://github.com/pyfisch/cbor — cloned to `vendor/serde_cbor` at `347a3f0` (master, 2021-08-15) — **archived** |
| Released == HEAD? | yes — `diff -r` of tarball `src/` vs `vendor/serde_cbor/src` is empty |
| License | MIT/Apache-2.0 |

Citation convention: `[read: crate serde_cbor-0.11.2 <path>:<line>]` = published tarball; `[read: vendor/serde_cbor@347a3f0 <path>:<line>]` = git checkout; `[ran: <command> → <result>]`. "Behaviour harness" = scratch crate `probes/serde-family-behaviour` (host, `serde_cbor = { version = "=0.11.2", default-features = false }`, i.e. the no-alloc configuration); inputs are quoted inline so each check is reproducible.

## Verdict

- Archived since 2021-08-15; README tells new projects to use ciborium or minicbor [read: vendor/serde_cbor@347a3f0 README.md:6-18].
- Genuinely no_std **without alloc** for the serde path: `Serializer::new(SliceWrite)` + `de::from_slice_with_scratch`/`from_mut_slice` [ran: thumbv7m-none-eabi no_main bin with no `#[global_allocator]` links; 4/4 host tests pass].
- Text-keyed struct ↔ CBOR map via serde derive, zero-copy `&[u8]`/`&str` borrow from input, unknown keys skipped, fixed recursion limit 128 [ran].
- Preferred serialization on encode includes shortest-float (f64→f32→f16) [ran]; no deterministic map ordering in the serializer. `Value`'s `BTreeMap` ordering (std-only) produces RFC 8949 §4.2.1 bytewise order [ran].
- Gaps: tags need `std` (thread_local) [ran]; no bignums; `undefined` not accepted for `Option` [ran]; no_std error messages are lost ("Unknown error") [ran].

## 1. no_std / alloc

- `#![cfg_attr(not(feature = "std"), no_std)]`; `alloc` crate linked only under feature `alloc` [read: vendor/serde_cbor@347a3f0 src/lib.rs:318,325-326].
- Features: `default = ["std"]`, `alloc = ["serde/alloc"]`, `std = ["serde/std"]`, `tags`, `unsealed_read_write` [read: crate serde_cbor-0.11.2 Cargo.toml.orig `[features]`].
- Without `alloc`: `from_slice`/`to_vec` are not exported [read: vendor/serde_cbor@347a3f0 src/lib.rs:354-360]; `from_mut_slice` and `from_slice_with_scratch` are always available [read: vendor/serde_cbor@347a3f0 src/de.rs:64,84]; docs show "Serialize an object with `no_std` and without `alloc`" via `SliceWrite` [read: vendor/serde_cbor@347a3f0 src/lib.rs:215-247].
- `Value` is std-only (`#[cfg(feature = "std")] pub mod value`) [read: vendor/serde_cbor@347a3f0 src/lib.rs:335-336].
- No-alloc verified: [ran: `cargo build --release --target thumbv7m-none-eabi` in `experiments/probes/serde_cbor` (no_main bin, no global allocator) → Finished]. Also links for thumbv6m [ran: `cargo build --release --target thumbv6m-none-eabi` → Finished].
- `cargo tree --target thumbv7m-none-eabi -e features`: `serde_cbor v0.11.2 → serde v1.0.229` (no `std`/`alloc` features), `half feature "default" → half v1.8.3` [ran].
- `tags` feature does not build without std: [ran: scratch crate with `features = ["tags"]`, `default-features = false`, `cargo build --target thumbv7m-none-eabi` → `error: cannot find macro thread_local` / `cannot find module or crate std`]; source: `thread_local!(static CBOR_TAG …)` [read: vendor/serde_cbor@347a3f0 src/tags.rs:220].

## 2. API model

- serde `Serializer`/`Deserializer` [read: vendor/serde_cbor@347a3f0 src/lib.rs:339-346]. Optional `Value` enum (std). No own derive, no low-level token API exposed (`read`/`write` modules are private, `Write` trait sealed unless `unsealed_read_write`) [read: vendor/serde_cbor@347a3f0 src/lib.rs:330,333].

## 3. Derive and mapping

- Text-string keys: yes — struct field names are written as text keys (`key.serialize(...)` when not packed) [read: vendor/serde_cbor@347a3f0 src/ser.rs:595-607]; [ran: probe `encodes_text_keyed_map_into_slice` → exact 60-byte `{"off":0,"data":h'01020304',"len":123,"sha":h'aa…'}`].
- Integer keys: only the "packed" format, where struct keys become the field's ordinal index (0,1,2…) — not arbitrary integers [read: vendor/serde_cbor@347a3f0 src/lib.rs:66-81; src/ser.rs:78,602]. Decoder accepts both by default (`accept_packed: true`, `accept_named: true`) [read: vendor/serde_cbor@347a3f0 src/de.rs:186-194].
- Struct as array: not offered by the serializer (open issue #107) — only tuple structs/tuples become arrays (serde model).
- Enum representations: default externally tagged — unit → `"Unit"`, newtype → `{"NewType": v}`; `legacy_enums()` → `["NewType", v]`; packed → unit variant as integer index [read: vendor/serde_cbor@347a3f0 src/ser.rs:84-121,390-401]; [ran: harness `sc_unit_variant_as_text`: `Unit::B` → `61 42`].
- `None` omitted on encode: only with `#[serde(skip_serializing_if = "Option::is_none")]`; the definite map length is adjusted [ran: harness `sc_none_skipped_definite_len`: `{a:1,b:None}` → `a1 61 61 01`]. Without it `None` → `f6` [read: vendor/serde_cbor@347a3f0 src/ser.rs:380-382].
- Unknown keys ignored on decode: yes [ran: probe `decode_ignores_unknown_key_and_accepts_any_key_order` (extra `"upgrade": true`, shuffled order) → ok].
- Renames: serde attributes (`rename`, `rename_all`) — the serializer writes whatever `&'static str` key serde passes [read: vendor/serde_cbor@347a3f0 src/ser.rs:595-601].

## 4. Zero-copy decode

- Yes for definite-length strings: `visit_borrowed_bytes`/`visit_borrowed_str` [read: vendor/serde_cbor@347a3f0 src/de.rs:293,349]; [ran: probe `decodes_borrowing_from_input` asserts `decoded.sha.as_ptr() == input[28..].as_ptr()` → ok, with `from_slice_with_scratch(buf, &mut [])`].
- Indefinite-length strings need either a scratch buffer (copied, not borrowed) or `from_mut_slice` (rearranged in place, then borrowed) [read: vendor/serde_cbor@347a3f0 src/de.rs:58-90,160-166]; [ran: harness `sc_indefinite_bytes_needs_scratch_or_mut_slice`: `5f 41 01 41 02 ff` → error with empty scratch, `[1,2]` borrowed via `from_mut_slice`].

## 5. Encode into `&mut [u8]` / decode from `&[u8]` without an allocator

- Yes. Encode: `Serializer::new(ser::SliceWrite::new(buf))`, then `into_inner().bytes_written()`; decode: `de::from_slice_with_scratch(buf, &mut [])` [read: vendor/serde_cbor@347a3f0 src/lib.rs:215-293].
- [ran: `experiments/probes/serde_cbor`: `cargo test --lib` → 4 passed; `cargo build --release --target thumbv7m-none-eabi` → Finished; `cargo clippy --target thumbv7m-none-eabi -- -D warnings` → clean; `cargo clippy --lib --tests -- -D warnings` → clean; `cargo fmt --check` → clean].
- Short buffer → `Err`, not panic [ran: probe `encode_reports_short_buffer` → ok].
- Informational only: [ran: `size` thumbv7m release (opt-level z, LTO) → text 17810 B; thumbv6m → 20678 B]. Binary contains `core::fmt::num` Display and four `parse_value` monomorphs [ran: `nm -S --size-sort`].

## 6. RFC 8949 coverage

| Item | Encode | Decode | Evidence |
|---|---|---|---|
| Indefinite-length items | yes, when serde gives no length (`serialize_seq(None)`) | yes (bytes, text, array, map) | [read: vendor/serde_cbor@347a3f0 src/ser.rs:194-199; src/de.rs:653,677,701,725]; [ran: harness `sc_indefinite_seq_encode` → `9f 01 02 03 ff`] |
| Half-precision (f16) | yes (as shortest form) | yes | [read: src/ser.rs:321-323; src/de.rs:567-568]; [ran: harness `sc_f16_decode`: `f9 3e 00` → 1.5] |
| Preferred (shortest) integer/length heads | yes | accepts non-preferred | [read: src/ser.rs:140-182]; [ran: harness `sc_non_minimal_head_accepted`: `{"off": 18 00}` → ok] |
| Preferred (shortest) floats | yes: f64→f32→f16 when lossless; NaN → `f9 7e 00` | n/a | [read: src/ser.rs:312-345]; [ran: harness `sc_shortest_float`: 1.5 → `f9 3e 00`, 1000000.5 → `fa 49 74 24 08` (both match RFC 8949 §4.2.1 examples)] |
| Tags | only with `tags` feature, which needs std | silently dropped without `tags` | [read: src/lib.rs:300-302; src/de.rs:404-423]; [ran: harness `sc_tag_silently_ignored_on_decode`: `c2 41 01` → bytes `[1]`]; [ran: tags+no_std build fails] |
| Simple values incl. undefined | false/true/null only (unit → null) | false/true/null; undefined → unit; other simple values → `UnassignedCode` | [read: src/lib.rs:303-306; src/de.rs:755]; [ran: harness `sc_undefined_not_none`: `{"v": f7}` into `Option<u32>` → error] (open PR #218) |
| Bignums (tags 2/3) | no; u128 > u64::MAX → error | no (tag dropped, bytes returned) | [read: src/ser.rs:305]; [ran: harness `sc_u128_over_u64_rejected`] |
| Deterministic encoding | serializer: none (declaration / iteration order). `Value::Map` (`BTreeMap<Value,Value>`, std-only) sorts major type → length → bytes, i.e. §4.2.1 bytewise order | no check | [read: src/value/mod.rs:16-21,76-100]; [ran: scratch `serde_cbor-value-order` (std) with the eight RFC 8949 §4.2.1 example keys → `a8 0a… 1864… 20… 617a… 626161… 811864… 8120… f4…`, identical to §4.2.1 order; doc comment cites draft-ietf-cbor-7049bis-04] |
| Duplicate-key detection | n/a | codec: no; serde-derived structs reject a repeated known field | [ran: harness `sc_duplicate_key_rejected_by_serde_derive`: `{"off":0,"off":1}` → `Err`, Display = "Unknown error" in no_std] |
| Nesting-depth limit / recursion safety | n/a | fixed limit 128 (recursive descent with counter), not configurable | [read: src/de.rs:189,425-436]; [ran: harness `sc_depth_limit`: 100 nested arrays in an unknown field → ok, 200 → `RecursionLimitExceeded`]; open issues #166 (raise limit), #167 (stack size) |

## 7. RFC claims in docs

- "This crate implements the Concise Binary Object Representation from [RFC 7049]" [read: vendor/serde_cbor@347a3f0 README.md:24,29]. Tags/simple-value links point at RFC 7049 [read: src/lib.rs:310,312]. `Value` ordering cites draft-ietf-cbor-7049bis-04 [read: src/value/mod.rs:16-21]. No mention of RFC 8949 (published after the last src commit).

## 8. Maintenance

- Last commit: `347a3f0` 2021-08-15 "Final commit, archive project" (Pyfisch) [ran: `git log -1`]; last commit touching `src/`: `a218403` 2020-06-04 (Steven Roose) [ran: `git log -1 -- src`].
- Last release: 0.11.2, 2021-08-15 (crates.io); repo archived = true, pushed_at 2022-05-22 [ran: `gh api repos/pyfisch/cbor`].
- Open issues 26, open PRs 23 [ran: `gh issue list` / `gh pr list --state open`].
- Maintainers active in last 12 months: 0 [ran: `git shortlog -sn --since="12 months ago"` → empty].
- `unsafe` in `src/`: 0 [ran: `grep -rnw unsafe src` → 0].
- Still widely used: 91.4 M downloads total, 13.2 M recent; 549 reverse dependencies [ran: crates.io API].

## 9. Embedded-relevant open issues

- https://github.com/pyfisch/cbor/issues/167 — "Reduce stack size required for deserialization" (MCU user, `from_slice_with_scratch`).
- https://github.com/pyfisch/cbor/issues/166 — "Allow increasing recursion limit".
- https://github.com/pyfisch/cbor/issues/189 — "Compilation is very slow, especially for release builds".
- https://github.com/pyfisch/cbor/issues/107 — "Serialize structs as array".
- https://github.com/pyfisch/cbor/issues/86 — "(De)serialize all simple values".
- https://github.com/pyfisch/cbor/issues/77 — "(De)serialize 128 bit integers" (no bignum).
- https://github.com/pyfisch/cbor/issues/208 — "Why are tags feature-gated?".
- https://github.com/pyfisch/cbor/pull/218 — "Deserialize `undefined` as `None` for `Option`" (unmerged).
- https://github.com/pyfisch/cbor/pull/198 — "Allow enabling `serde/std` without also requiring `serde_cbor/std`" (unmerged).
- Closed for context: https://github.com/pyfisch/cbor/issues/178 "Support Canonical CBOR" (closed 2020-01-28, no serializer mode added); https://github.com/pyfisch/cbor/issues/79 "no_std support" (closed).

## Probe

- `experiments/probes/serde_cbor/` (lib + `#![no_main]` bin, same `Chunk`/`sample`/60-byte vector/4 tests as `minicbor-serde`).
- Dependency line: `serde_cbor = { version = "=0.11.2", default-features = false }` plus `serde = { version = "1", default-features = false, features = ["derive"] }`, `serde_bytes = { version = "0.11", default-features = false }`.
- Results: [ran: `cargo test --lib` → 4 passed, 0 ignored]; [ran: thumbv7m + thumbv6m release builds → Finished]; [ran: fmt + both clippy invocations → clean]. Toolchain rustc 1.95.0.
