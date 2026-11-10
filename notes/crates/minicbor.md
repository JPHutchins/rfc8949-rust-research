# minicbor (+ minicbor-derive, minicbor-serde) — evidence notes

| Field | Value |
|---|---|
| Released version | minicbor 2.3.1 (2026-10-18), minicbor-derive 0.19.5 (2026-07-20), minicbor-serde 0.7.2 (2026-10-18) — sources: https://static.crates.io/crates/minicbor/minicbor-2.3.1.crate (`.cargo_vcs_info.json` sha `0432e74`), https://static.crates.io/crates/minicbor-derive/minicbor-derive-0.19.5.crate (sha `d1fad46`), https://static.crates.io/crates/minicbor-serde/minicbor-serde-0.7.2.crate (sha `0432e74`) |
| Repository | https://github.com/twittner/minicbor — cloned to `vendor/minicbor`, checked out at `0432e74` (= `origin/master`, the 2.3.1 / serde 0.7.2 release commit, 2026-10-18). Default branch is `develop` (`000f8bf`); `git diff 0432e74 000f8bf` is empty |
| Released == HEAD? | yes for minicbor / minicbor-serde; minicbor-derive 0.19.5 was cut from `d1fad46`, and `minicbor-derive/src/encode.rs` differs by 4 lines at HEAD |
| License | BlueOak-1.0.0 [read: crate minicbor-2.3.1 Cargo.toml.orig:5] |

Citation convention: `[read: crate <name>-<ver> <path>:<line>]` = published tarball; `[read: vendor/minicbor@0432e74 <path>:<line>]` = git checkout; `[ran: <command> → <result>]`.

## Verdict

- **no_std without alloc: yes**, for `minicbor` core and for `minicbor-serde`. A thumbv7m `no_main` binary with no `#[global_allocator]` links for both [ran: probes below].
- **minicbor-derive cannot express text-string keys.** Map keys are numeric `#[n(i64)]` indices only. Text keys work through `minicbor-serde` (serde derive) or a hand-written `Encode`/`Decode` impl. Upstream has a maintainer-authored draft PR (#53) and a contributor PR (#57, `#[cbor(text_keys)]`) adding string keys. Neither is merged.
- **Zero-copy decode** of `&str` and `&[u8]` from the input works in both the derive and serde paths [ran].
- **RFC 8949 data model.** The docs cite RFC 8949, and all 81 RFC 8949 Appendix A vectors round-trip [ran].
  - One well-formedness defect: `simple(20..=31)` is emitted as `f8 xx` and accepted on decode. RFC 8949 §3.3 says this is not well-formed.
  - Optional 8949 features that are not implemented (not conformance violations):
    - preferred float serialization: floats are written at their declared width, while integers and lengths are shortest;
    - §4.2 deterministic encoding: there is no generic map-key sorting (issue #58 is open);
    - decode-side enforcement of preferred encoding.
- Without `alloc`, `Decoder::skip`, which is used to ignore unknown keys, errors on an indefinite array or map nested inside a definite one when other items are still pending [ran].

## 1. no_std / alloc

- `#![cfg_attr(not(feature = "std"), no_std)]` [read: crate minicbor-2.3.1 src/lib.rs:138]; `extern crate alloc` only under `feature = "alloc"` [read: crate minicbor-2.3.1 src/lib.rs:140-141].
- Features: `alloc = []`, `std = ["alloc"]`, `derive = ["minicbor-derive"]`, `half` (optional dep `half` with `default-features = false`), `full = ["std","derive","half"]`. Defaults are empty, so a plain dependency is no_std/no-alloc [read: crate minicbor-2.3.1 Cargo.toml.orig:15-23].
- minicbor-serde: `alloc = ["minicbor/alloc","serde/alloc"]`, `std = [...]`, `half`; no defaults; `serde` with `default-features = false` [read: crate minicbor-serde-0.7.2 Cargo.toml.orig:15-24].
- [ran: `cargo build --release --target thumbv7m-none-eabi` in `experiments/probes/minicbor` (no_main bin, no global allocator) → Finished; `size` text = 6178 B (informational)].
- [ran: same in `experiments/probes/minicbor-serde` → Finished; text = 8268 B (informational)].
- Negative control proving the method: a no_main bin with only `extern crate alloc;` fails with "no global memory allocator found but one is required" [ran: scratchpad `probes/control-alloc`, `cargo build --release --target thumbv7m-none-eabi` → error].

## 2. API model

- Own traits `Encode<C>` / `Decode<'b, C>` (context-parameterised), with type-directed `Encoder<W>` / `Decoder<'b>` methods, a `Tokenizer`, and `CborLen` for computing encoded size [read: crate minicbor-2.3.1 src/lib.rs:3-22].
- Derive macros for `Encode`, `Decode` and `CborLen` behind feature `derive` [read: crate minicbor-derive-0.19.5 src/lib.rs:1-2].
- serde `Serializer` / `Deserializer` in the separate crate `minicbor-serde` [read: crate minicbor-serde-0.7.2 src/lib.rs:1-7, 51-52].
- `encode::Write` is minicbor's own trait (not `embedded-io`). Slice writing goes through `encode::write::Cursor` / `EndOfSlice` [read: crate minicbor-2.3.1 src/encode/write.rs:160].

## 3. Derive and mapping

minicbor-derive (`#[derive(Encode, Decode)]`):
- Text-string keys: **no**. "The encoding does not contain any names … every field and every constructor needs to be annotated with an index number" [read: crate minicbor-derive-0.19.5 src/lib.rs:16-18]. The index type is `Idx::N(i64) | Idx::B(i64)` [read: crate minicbor-derive-0.19.5 src/attrs/idx.rs:7-12]. Open issue #52 asks for string indices. The maintainer points to minicbor-serde and opened draft PR #53 (`#[s("...")]`). Contributor PR #57 builds `#[cbor(text_keys)]` on top of it (see §9).
- Integer keys: yes, `#[cbor(map)]` with `#[n(k)]`, and negative indices since 0.16.0 [read: crate minicbor-derive-0.19.5 src/lib.rs:471-486; read: vendor/minicbor@0432e74 CHANGELOG.md:545-549].
- Struct as array: yes; this is the default [read: crate minicbor-derive-0.19.5 src/lib.rs:143-144, 454-469].
- Enum representations: `[index, value]` (default), `#[cbor(index_only)]`, and `#[cbor(flat)]` [read: crate minicbor-derive-0.19.5 src/lib.rs:488-509].
- `None` omitted on encode: yes for map encoding ("Optional fields whose value is `None` are not encoded"); `null` under array encoding [read: crate minicbor-derive-0.19.5 src/lib.rs:457-460, 486]; also `#[cbor(skip_if = ...)]` [read: same, 181-197].
- Unknown keys ignored on decode: yes, via `Decoder::skip`. Without `alloc`, an indefinite map or array nested in a regular one cannot always be skipped [read: crate minicbor-derive-0.19.5 src/lib.rs:20, 532-536; read: crate minicbor-derive-0.19.5 src/decode.rs:425-431]. There is no "deny unknown" option (issue #60 is open).
- Renames: not applicable, since names are never encoded.
- Duplicate keys: derived map decode assigns on each match (`Ok(__v777) => #name = #value`), so the last value wins and nothing is detected [read: crate minicbor-derive-0.19.5 src/decode.rs:368-372, 422-431].
- Derived map fields are emitted in bytewise-lexicographic order of their integer keys. They are sorted at macro time with `(val < 0, |val|)`, which matches RFC 8949 §4.2.1 for integer keys [read: crate minicbor-derive-0.19.5 src/attrs/idx.rs:40-43; src/fields.rs:76].

minicbor-serde (serde derive):
- Text-string keys: yes. Field names are encoded as strings: "nor does it use integers instead of strings for struct field names" [read: crate minicbor-serde-0.7.2 src/lib.rs:3-7]. `serialize_struct` writes a definite `map(len)` [read: crate minicbor-serde-0.7.2 src/ser.rs:277-284]. [ran: `experiments/probes/minicbor-serde` `cargo test --lib` → `encodes_text_keyed_map_into_slice` ok (exact 60-byte vector)].
- Integer keys: not via serde derive (issue #60 notes this).
- `None` omitted: through serde's `skip_serializing_if`; serde_derive computes the definite length.
- Unknown keys ignored: yes, through `deserialize_ignored_any → decoder.skip()` [read: crate minicbor-serde-0.7.2 src/de.rs:321-324]. [ran: `decode_ignores_unknown_key_and_accepts_any_key_order` ok]. The same no-alloc skip limit applies [ran: `decode_without_alloc_cannot_skip_nested_indefinite_unknown_value` ok, meaning the error is returned].
- Tags, undefined and bignums in `deserialize_any`: tags 2/3 map to `u128`/`i128`, any other tag is an error, and `undefined`/simple values are errors [read: crate minicbor-serde-0.7.2 src/de.rs:126-140]. Custom tags go through the `tag` module [read: crate minicbor-serde-0.7.2 src/tag.rs:1].
- Renames and enum representations: standard serde attributes. `serialize_struct_variant` produces `{variant: {...}}` [read: crate minicbor-serde-0.7.2 src/ser.rs:286-296].
- Duplicate struct-field keys: rejected, but by serde_derive's generated visitor rather than by the codec [ran: `duplicate_field_key_rejected_by_serde_derive` ok].

## 4. Zero-copy decode

- `Decoder::str() -> &'b str` and `Decoder::bytes() -> &'b [u8]` borrow from the input [read: crate minicbor-2.3.1 src/decode/decoder.rs:326, 364].
- In derive, `&str`, `&ByteSlice` and their `Option`s borrow implicitly. `&[u8]` needs `with = "minicbor::bytes"` [read: crate minicbor-derive-0.19.5 src/lib.rs:402-445].
- In serde, `visit_borrowed_str` / `visit_borrowed_bytes` [read: crate minicbor-serde-0.7.2 src/de.rs:85-86, 200-209]. [ran: `decodes_borrowing_from_input` asserts `decoded.sha.as_ptr()` points into the input → ok, in both probes].
- Indefinite (chunked) strings cannot be borrowed as one slice. `bytes()` / `str()` reject them; `bytes_iter` / `str_iter` handle them [read: crate minicbor-2.3.1 src/decode/decoder.rs:325, 343, 363, 382].

## 5. Encode into `&mut [u8]` / decode from `&[u8]` without an allocator

- Core: `minicbor::encode(x, &mut Cursor::new(buf))` + `cursor.position()`, and `minicbor::decode(&[u8])` [read: crate minicbor-2.3.1 src/lib.rs:173-195]. [ran: `experiments/probes/minicbor`: `cargo test --lib` → 5 passed; `cargo build --release --target thumbv7m-none-eabi` → Finished; `cargo clippy --target thumbv7m-none-eabi -- -D warnings` clean].
- serde: `minicbor_serde::to_slice(val, &mut [u8]) -> Result<usize, _>` (added 2026-09, PR by Dan Nixon) and `from_slice(&[u8])` [read: crate minicbor-serde-0.7.2 src/ser.rs:25-30; src/de.rs:17-19]. [ran: `experiments/probes/minicbor-serde`: `cargo test --lib` → 6 passed; thumbv7m release build → Finished; clippy (thumb and host tests) clean].
- A short buffer returns an error, not a panic [ran: `encode_reports_short_buffer` ok in both].

## 6. RFC 8949 coverage

| Item | Encode | Decode | Evidence |
|---|---|---|---|
| Indefinite-length items | yes (`begin_array/map/bytes/str` + `end`) | yes (`array()/map()` return `None`; `bytes_iter`/`str_iter`) | [read: crate minicbor-2.3.1 src/encode/encoder.rs:293-322; src/decode/decoder.rs:343, 382, 404, 449] [ran: Appendix A `(_ …)`, `[_ …]`, `{_ …}` vectors round-trip] |
| Half-precision (f16) | yes, explicit `Encoder::f16`, feature `half` | yes, feature `half`; without `half`, `0xf9` is a type error | [read: crate minicbor-2.3.1 src/encode/encoder.rs:206-227; src/decode/decoder.rs:276-312] |
| Preferred (shortest) integer/length heads | yes | accepts non-preferred (`1a00000001` → `u8` 1) | [read: crate minicbor-2.3.1 src/encode/encoder.rs:62-130, 347-356] [ran: `experiments/conformance/appendix-a/minicbor` test `decoder_accepts_non_preferred_integer_head` ok]. This is permitted: "Full implementations of CBOR decoders are by definition variation tolerant" [read: sources/rfc/rfc8949.txt:1352-1353] |
| Preferred (shortest) floats | **no**: `f32` is always `fa…` and `f64` is always `fb…` | n/a (widens f16/f32 to f64) | [read: crate minicbor-2.3.1 src/encode/encoder.rs:229-237] [ran: test `floats_are_encoded_at_declared_width` asserts `f64(1.5)` → `fb3ff8000000000000` and `f32(1.5)` → `fa3fc00000`; the preferred form is `f9 3e00`]. Optional per RFC 8949 §4.1 [read: sources/rfc/rfc8949.txt:1329-1330] |
| Tags | yes (`tag()`, `IanaTag`, derive `#[cbor(tag(n))]`) | yes | [read: crate minicbor-2.3.1 src/encode/encoder.rs:250; src/decode/decoder.rs:492] |
| Simple values incl. undefined | yes (`undefined()`, `simple()`). `simple(20..=31)` is emitted as `f8 14..1f`, which RFC 8949 §3.3 calls not well-formed | yes. Also accepts the not-well-formed `f8 14`, `f8 18` | [read: crate minicbor-2.3.1 src/encode/encoder.rs:193-204; src/decode/decoder.rs:515-535; sources/rfc/rfc8949.txt:869-871] [ran: test `simple_values_20_to_31_use_not_well_formed_two_byte_form` asserts encode `[f8,14,f8,18]` and decode `f8 18` → 24, `f8 14` → 20] |
| Bignums (tags 2/3) | yes for `u128`/`i128` beyond 64 bits | yes into `u128`/`i128` | [read: crate minicbor-2.3.1 src/encode/encoder.rs:132-170; src/decode/decoder.rs:186-245] |
| Deterministic encoding | **none** for generic maps (`BTreeMap` is emitted in Rust `Ord` order); derive int-key maps are statically sorted bytewise (§4.2.1-compatible for integer keys) | no validation | [read: crate minicbor-derive-0.19.5 src/fields.rs:76; src/attrs/idx.rs:40-43]; open issue #58 "Producing and enforcing deterministically encoded CBOR" (maintainer, 2026-05-23: "I think this can be done, but it might take a while") |
| Duplicate-key detection | n/a | no (derive: last wins; serde path: serde_derive rejects duplicate struct fields) | [read: crate minicbor-derive-0.19.5 src/decode.rs:368-372] [ran: `duplicate_field_key_rejected_by_serde_derive` ok] |
| Nesting-depth limit / recursion safety | n/a | `Decoder::skip` is iterative, so it has no recursion. Derived and serde `Decode` recurse without a limit (PR #71 proposes `max_depth`) | [read: crate minicbor-2.3.1 src/decode/decoder.rs:548-662, 664-708]; PR #71 text: "minicbor has no bound on that today" |

All 81 RFC 8949 Appendix A examples decode through `Decoder::tokens()` and re-encode byte-identically [ran: `cd experiments/conformance/appendix-a/minicbor && cargo test -- --nocapture` → `vectors=81 failures=0`; the test asserts `failures.is_empty()`]. This needs feature `half` for the `Token` API.

## 7. RFC claims in docs

- README: "A small CBOR codec suitable for `no_std` environments", where CBOR links to `https://tools.ietf.org/html/rfc8949` [read: crate minicbor-2.3.1 README.md:3, 15].
- Crate docs link `[CBOR]` to `https://datatracker.ietf.org/doc/html/rfc8949` [read: crate minicbor-2.3.1 src/lib.rs:133]. Bignum docs cite "RFC 8949 §3.4.3" [read: crate minicbor-2.3.1 src/decode/decoder.rs:190, 216].
- The repo's own test suite runs "Test vectors of RFC 7049" (Appendix A) [read: vendor/minicbor@0432e74 minicbor-tests/tests/rfc.rs:6-8]. I did not run the upstream suite, because the sandbox denied running repo test code; the equivalent Appendix A run above uses the crates.io release.
- The docs make no claim of deterministic/canonical encoding.

## 8. Maintenance

- Last commit: `000f8bf` 2026-10-18 Toralf Wittner "Update CHANGELOG and increment versions." [ran: `git log -1` on `develop`]. Last code commit by another author: `9f0547d` 2026-10-07 Stan Bondi [ran: `git log -5 -- minicbor minicbor-derive minicbor-serde`].
- Last release: minicbor 2.3.1 / minicbor-serde 0.7.2 on 2026-10-18; minicbor-derive 0.19.5 on 2026-07-20 [ran: crates.io API].
- Open: 5 issues, 5 PRs; not archived; 114 stars; pushed 2026-10-18 [ran: `gh api repos/twittner/minicbor`, `gh issue list`, `gh pr list`].
- Maintainers, last 12 months: 1 maintainer (Toralf Wittner, 59 commits) plus 5 contributors (Stan Bondi 8, Panav Munshi 4, David Tolnay 2, Chris Bandy 1, Dan Nixon 1) [ran: `git shortlog -sne --since="12 months ago" HEAD`].
- `unsafe`: 6 blocks in minicbor. 4 are `repr(transparent)` casts for `ByteSlice` [read: crate minicbor-2.3.1 src/bytes.rs:38-64], and 2 are in an internal `ArrayVec` used for decoding `[T; N]` [read: crate minicbor-2.3.1 src/decode.rs:477, 504]. minicbor-derive and minicbor-serde have 0 [ran: `grep -rn unsafe`].
- Downloads: 19.5 M total, 7.0 M recent [ran: crates.io API].

## 9. Embedded-relevant open issues

- https://github.com/twittner/minicbor/issues/52: "minicbor_derive: strings as indecies?" (2026-02-11). The maintainer started draft PR #53; a commenter is running the branch in production.
- https://github.com/twittner/minicbor/pull/53: "Add support for string indices." Draft by the maintainer: `#[s("...")]`, map encoding only.
- https://github.com/twittner/minicbor/pull/57: "Add `#[cbor(text_keys)]` for automatic string-keyed map encoding" (contributor, 2026-03-14, builds on #53).
- https://github.com/twittner/minicbor/pull/56: "Add MaxCborLen trait and derive macro for compile-time max CBOR size", motivated by fixed-size stack buffers in no_std without alloc.
- https://github.com/twittner/minicbor/issues/58: "Producing and enforcing deterministically encoded CBOR" (RFC 8949 §4.2). The maintainer says it can be done, without a timeline.
- https://github.com/twittner/minicbor/issues/60: "Allow forbidding unknown fields".
- https://github.com/twittner/minicbor/pull/71: "Add a max_depth attribute to the Decode derive", covering unbounded recursion on hostile input.
- https://github.com/twittner/minicbor/issues/68: "Consider choosing dynamically between array and map struct encoding".

## Probe

- `experiments/probes/minicbor-serde/`: serde derive, text keys, `to_slice` / `from_slice`. Deps: `minicbor = { version = "=2.3.1", default-features = false }`, `minicbor-serde = { version = "=0.7.2", default-features = false }`, `serde = { version = "1", default-features = false, features = ["derive"] }`, `serde_bytes = { version = "0.11", default-features = false }`. `serde_bytes` is needed so `&[u8]` encodes as a byte string rather than an array. 6 host tests pass; thumbv7m no-alloc bin links; text 8268 B.
- `experiments/probes/minicbor/`: hand-written `Encode` / `Decode` over the core API (manual, not declarative). Dep: `minicbor = { version = "=2.3.1", default-features = false }`. 5 host tests pass; thumbv7m no-alloc bin links; text 6178 B.
- `experiments/conformance/appendix-a/minicbor/`: RFC 8949 Appendix A token round-trip plus well-formedness spot checks (host only, feature `half`).
