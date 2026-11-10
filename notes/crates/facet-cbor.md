# facet-cbor — evidence notes

| Field | Value |
|---|---|
| Released version | 0.3.4 (2026-05-15) — source: https://static.crates.io/crates/facet-cbor/facet-cbor-0.3.4.crate (`.cargo_vcs_info.json` sha `27eef57`, `path_in_vcs = rust/facet-cbor`) |
| Repository | https://github.com/bearcove/vox (crate formerly at `rust/facet-cbor`) — cloned to `vendor/facet-cbor` at `4caa675` (main, 2026-08-21) |
| Released == HEAD? | **no — the crate no longer exists at vox HEAD.** It was removed by `4b58849b` (2026-08-08, "repo: restore current Vox workspace from Facet (#381)") [ran: `git log --format='%h %cI %s' --diff-filter=D -1 -- rust/facet-cbor/Cargo.toml` → `4b58849b`; `git ls-tree --name-only HEAD rust/` has no `facet-cbor`, while the parent `2ca38f4` has it]. All citations below are to the published tarball. |
| License | MIT OR Apache-2.0 [read: crate facet-cbor-0.3.4 Cargo.toml:34] |

Citation convention: `[read: crate <name>-<ver> <path>:<line>]` = published tarball; `[read: vendor/<dir>@<sha7> <path>:<line>]` = git checkout; `[ran: <command> → <result>]`.

## Verdict

- **std-only.** The crate has no `#![no_std]`, uses `std::` paths, and pulls `facet-reflect` with default features (`std`, `regex`). A thumbv7m build fails with `can't find crate for std` [ran]. Even its reflection dependency `facet-reflect` hard-requires `alloc`.
- Its API model is reflection (`facet::Facet` derive plus `Peek`/`Partial` at runtime): `to_vec(&T) -> Vec<u8>` and `from_slice::<T: Facet<'static>>`, with owned output only and no zero-copy.
- Structs become text-keyed maps from field names, and unknown keys are skipped. However:
  - `None` is written as `null`, not omitted.
  - `#[facet(rename)]` on a field is **ignored on the wire** [ran].
  - `&[u8]` encodes as an array; only `Vec<u8>` becomes a byte string.
- Spec coverage is thin:
  - no indefinite lengths, no f16, no shortest floats;
  - no tags on decode;
  - bignums on encode only;
  - no canonical ordering, no duplicate detection;
  - an unbounded recursive skip (stack overflow at 1M depth) [ran].
- The crate is **orphaned**: it was removed from its only repo three months after the last release, and I found no successor.

## 1. no_std / alloc

- `src/lib.rs` has no `no_std` attribute; it starts with `#![deny(unsafe_code)]` [read: crate facet-cbor-0.3.4 src/lib.rs:1-11]. It uses `std::borrow::Cow` [read: crate facet-cbor-0.3.4 src/deserialize.rs:175] and `impl std::error::Error` [read: crate facet-cbor-0.3.4 src/error.rs:27].
- Its dependencies are `facet` 0.46 with features `camino` and `reflect`, plus `facet-core` and `facet-reflect` 0.46 with default features [read: crate facet-cbor-0.3.4 Cargo.toml:56-67].
- `facet-reflect` 0.46.5 defaults to `["std", "regex"]` and has an unconditional `extern crate alloc;` [read: ~/.cargo/registry facet-reflect-0.46.5 Cargo.toml:63-66], [read: ~/.cargo/registry facet-reflect-0.46.5 src/lib.rs:25]. `facet-core` gates alloc behind a feature [read: ~/.cargo/registry facet-core-0.46.5 src/lib.rs:22-23].
- The thumbv7m build fails on std [ran: `cargo build --release --target thumbv7m-none-eabi` on a `#![no_std] #![no_main]` bin depending only on `facet-cbor = "=0.3.4"` → `error[E0463]: can't find crate for 'std'` while compiling `regex-syntax`]. The `regex-syntax` pull comes from `regex-syntax ← regex ← facet-reflect ← facet ← facet-cbor` [ran: `cargo tree --target thumbv7m-none-eabi -e normal -i regex-syntax`].
- Verdict: no_std without alloc: **no**. no_std + alloc: **no** as published. std-only: **yes**. Feature flags: **none**.

## 2. API model

The API is reflection-based, built on facet's `Facet` derive and runtime shape reflection: `Peek` for serialization, `Partial` for deserialization.

- Public API: `to_vec<T: Facet>(&T) -> Result<Vec<u8>, CborError>`, `serialize_peek(Peek, &mut Vec<u8>)`, and `from_slice<T: Facet<'static>>(&[u8])` [read: crate facet-cbor-0.3.4 src/lib.rs:9-11], [read: crate facet-cbor-0.3.4 src/serialize.rs:8-14], [read: crate facet-cbor-0.3.4 src/deserialize.rs:8-20].
- Decoding allocates a heap `Partial` (`Partial::alloc_owned::<T>()`) and then materializes it [read: crate facet-cbor-0.3.4 src/deserialize.rs:10-19].
- There is no serde and no low-level public token API. The `decode` and `encode` modules are private (`mod decode; mod encode;`) [read: crate facet-cbor-0.3.4 src/lib.rs:3-7].

## 3. Derive and mapping

- **Text-string keys:** Yes, and they are the default. A struct is written as `map(n)` followed by `field.name` text keys in declaration order [read: crate facet-cbor-0.3.4 src/serialize.rs:120-135]. The `Chunk` struct (with `Vec<u8>` fields) produces exactly the expected 60 bytes and round-trips [ran: `cargo test --lib` → `text_keyed_roundtrip ... ok`].
- **Integer keys:** No struct-field integer keys. Only `HashMap`/`BTreeMap`-style `Def::Map` values can have arbitrary keys [read: crate facet-cbor-0.3.4 src/serialize.rs:82-92].
- **Struct as array:** Only tuple structs [read: crate facet-cbor-0.3.4 src/serialize.rs:136-148].
- **Enum representations:**
  - Externally tagged by default (`{variant_name: payload}`) [read: crate facet-cbor-0.3.4 src/serialize.rs:154-171].
  - Internally tagged with `#[facet(tag = "...")]` [read: crate facet-cbor-0.3.4 src/serialize.rs:164-167].
  - Variant renames are honoured [read: crate facet-cbor-0.3.4 src/serialize.rs:162].
- **`None` omitted on encode:** **No.** `None` is written as `null` [read: crate facet-cbor-0.3.4 src/serialize.rs:29-40]; [ran: `none_encoded_as_null ... ok` → `a2 63 6f6666 01 63 6c656e f6`]. On decode, a missing `Option` field becomes `None` [ran: `missing_option_field_decode` → `Ok(Opt { off: 1, len: None })`].
- **Unknown keys ignored on decode:** Yes. "if unknown, skip the value" via `decode::skip_value` [read: crate facet-cbor-0.3.4 src/deserialize.rs:275-293]. Verified, including reordered keys [ran: `unknown_key_and_reordered ... ok`].
- **Renames:** Field renames are **ignored by the serializer**, which writes `field.name` [read: crate facet-cbor-0.3.4 src/serialize.rs:128]. `#[facet(rename = "o")] off: u32` encoded as `a1 63 6f6666 01`, i.e. key `"off"`, not `"o"` [ran: `rename_used_on_wire` → `[a1, 63, 6f, 66, 66, 01]`].
- **Bytes:** Only `Vec<u8>` becomes a CBOR byte string [read: crate facet-cbor-0.3.4 src/serialize.rs:41-58]. `[u8; N]` and `&[u8]` (`Def::Array` / `Def::Slice`) become arrays of integers [read: crate facet-cbor-0.3.4 src/serialize.rs:71-80].

## 4. Zero-copy decode

No.

- The bound is `T: Facet<'static>` [read: crate facet-cbor-0.3.4 src/deserialize.rs:9].
- `&str` is decoded by copying: the comment says "&str can't be deserialized into owned Partial (would need borrowed)", followed by `s.to_owned()` [read: crate facet-cbor-0.3.4 src/deserialize.rs:167-171].
- Byte strings are `.to_vec()`'d [read: crate facet-cbor-0.3.4 src/deserialize.rs:224-226].

## 5. Encode into `&mut [u8]` / decode from `&[u8]` without an allocator

No. Every encoder primitive appends to `&mut Vec<u8>` [read: crate facet-cbor-0.3.4 src/encode.rs:1-80], and decode heap-allocates a `Partial`. The crate does not build without std (§1).

## 6. RFC 8949 coverage

| Item | Encode | Decode | Evidence |
|---|---|---|---|
| Indefinite-length items | no | no (rejected: "unsupported additional info") | `read_argument` accepts only 0–27 [read: crate facet-cbor-0.3.4 src/decode.rs:16-69]; [ran: `indefinite_rejected ... ok` on `bf … ff`] |
| Half-precision (f16) | no | no (`read_f32` requires `0xfa`, `read_f64` requires `0xfb`) | details below |
| Preferred (shortest) integer/length heads | yes | accepts non-shortest | details below |
| Preferred (shortest) floats | **no** (f32 → `fa`, f64 → `fb` always) | width must match the Rust type exactly | details below |
| Tags | only internally for bignum encode; no user tag API | no (only skipped when unknown) | details below |
| Simple values incl. undefined | null/true/false only | null/bool only; others only skipped | details below |
| Bignums (tags 2/3) | yes for `u128`/`i128` beyond 64 bits | **no** (128-bit decode → `UnsupportedType`) | details below |
| Deterministic encoding (key order) | **none**; declaration order | n/a | struct fields written in index order [read: crate facet-cbor-0.3.4 src/serialize.rs:126-133] |
| Duplicate-key detection | none | none; **last value wins** | [ran: `duplicate_key` → `{off:1, off:2}` decodes to `Ok(Opt { off: 2, .. })`] |
| Nesting-depth limit / recursion safety | none | none; recursive `skip_value` **overflows the stack** | details below |

**f16.** [read: crate facet-cbor-0.3.4 src/decode.rs:179-229]; [ran: `float_widths ... ok` (`f9 3e00` into an `f64` field → Err)]

**Preferred integer and length heads.** `write_major` cites "minimal encoding (RFC 8949 §4.2)" [read: crate facet-cbor-0.3.4 src/encode.rs:56-80]. The decoder does not check minimality [read: crate facet-cbor-0.3.4 src/decode.rs:16-69].

**Preferred floats.** [read: crate facet-cbor-0.3.4 src/encode.rs:44-54]; [ran: `float_widths ... ok` (`1.5f64` → `fb 3ff8000000000000`)]

**Tags.** No tag decode path in `deserialize.rs`; `skip_value` handles tags only when skipping [read: crate facet-cbor-0.3.4 src/decode.rs:290-294].

**Simple values.** `write_null` / `write_bool` [read: crate facet-cbor-0.3.4 src/encode.rs:34-42]; `skip_value` accepts any simple value [read: crate facet-cbor-0.3.4 src/decode.rs:296-326].

**Bignums.** Encode uses `0xc2`/`0xc3` with leading-zero-stripped bytes [read: crate facet-cbor-0.3.4 src/serialize.rs:283-296], [read: crate facet-cbor-0.3.4 src/serialize.rs:328-353] (i128). The scalar decoder has no U128/I128 arm, so it falls through to `UnsupportedType` [read: crate facet-cbor-0.3.4 src/deserialize.rs:178-180].

**Nesting depth.** `skip_value` recurses without a limit [read: crate facet-cbor-0.3.4 src/decode.rs:252-294]. [ran: an unknown key whose value is 1 000 000 nested arrays → `thread 'main' has overflowed its stack`, exit 134; depth 10 000 → ok]

## 7. RFC claims in docs

- No README is published (`readme = false`) [read: crate facet-cbor-0.3.4 Cargo.toml:26].
- The only RFC reference in the source is RFC 8949 §4.2, in the `write_major` doc comment [read: crate facet-cbor-0.3.4 src/encode.rs:56].
- Crate description: "CBOR serialization for Facet types" [read: crate facet-cbor-0.3.4 Cargo.toml:23].

## 8. Maintenance

- Last release: 0.3.4, 2026-05-15. The changelog entry says only "release (#275)" [read: crate facet-cbor-0.3.4 CHANGELOG.md:10-14]. Versions 0.0.0 through 0.3.4 were all published between 2026-03-23 and 2026-05-15 [ran: crates.io API].
- Last human commit touching `rust/facet-cbor`: `24a292b9` 2026-04-15, Amos Wenger, "fix: publish facet-cbor 0.3.3 with facet-core = "0.46"". Later commits are the release bot (`27eef573`) and the removal (`4b58849b`, cool bear bot, 2026-08-08) [ran: `git log --format='%h %cI %an %s' -6 -- rust/facet-cbor`].
- Path-scoped human authors in the 12 months before removal: 1 (Amos Wenger, 16 commits; github-actions[bot] 3 excluded) [ran: `git shortlog -sne --since="12 months ago" 4b58849b^1 -- rust/facet-cbor`].
- Successor search:
  - `facet-rs/facet` main tree: `truncated=false`, 853 entries, no path matching `cbor` [ran: `gh api repos/facet-rs/facet/git/trees/main?recursive=1`].
  - crates.io reverse dependencies: 4, all `vox-*` 0.8.2 [ran: crates.io API `/reverse_dependencies`].
  - vox HEAD has no `cbor` in any `rust/**/Cargo.toml` or `.rs` [ran: `grep -rln cbor --include=Cargo.toml rust` → none].
  - **Successor unknown.**
- Repo-wide (vox): 18 open issues and 12 open PRs (`open_issues_count` = 30); not archived; pushed 2026-11-08 [ran: `gh api repos/bearcove/vox`, `gh issue list`, `gh pr list`]. These numbers no longer concern this crate.
- `unsafe`: 0, and enforced by `#![deny(unsafe_code)]` [read: crate facet-cbor-0.3.4 src/lib.rs:1]; [ran: `grep -rn unsafe src | grep -v deny` → 0].

## 9. Embedded-relevant open issues

- None found for the Rust crate. `gh issue list -R bearcove/vox --state all --search cbor` returns only #254 ("Swift: O(n²) array concatenation in postcard/CBOR encoding") and #282 ("Swift/TS: Use extracted schemas rather than raw facet Shapes"), both about the Swift/TS side [ran].
- The crate is std-only and removed from its repo, so embedded issues are moot.

## Probe

**Not eligible for the shortlist**, because it is std-only. The probes live only in the scratchpad, not in `experiments/`.

No-std check (failing). It uses a `#![no_std] #![no_main]` bin calling `facet_cbor::from_slice::<u32>(&[0x00])`, with dependency `facet-cbor = { version = "=0.3.4" }`:

```text
$ cargo build --release --target thumbv7m-none-eabi
error[E0463]: can't find crate for `std`
error: could not compile `regex-syntax` (lib) due to 1 previous error
```

Host behaviour tests: 8/8 pass, resolving facet 0.46.5 [ran: `cargo test --lib -- --nocapture`]. Types used:

```rust
#[derive(Debug, PartialEq, Facet)]
pub struct Chunk { pub off: u32, pub data: Vec<u8>, pub len: u32, pub sha: Vec<u8> }
// facet_cbor::to_vec(&c)? == expected 60 bytes; facet_cbor::from_slice::<Chunk>(&bytes)? == c
```
