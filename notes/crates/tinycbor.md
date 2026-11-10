# tinycbor (+ tinycbor-derive) — evidence notes

| Field | Value |
|---|---|
| Released version | tinycbor 0.15.1 (2026-07-31), tinycbor-derive 0.15.4 (2026-07-28). Sources: https://static.crates.io/crates/tinycbor/tinycbor-0.15.1.crate (`.cargo_vcs_info.json` sha `bcedf72`, **not present in the GitHub repo**) and https://static.crates.io/crates/tinycbor-derive/tinycbor-derive-0.15.4.crate (sha `269a82d`, in repo). The probe uses **tinycbor 0.13.0** (sha `68df149`), see §1 |
| Repository | https://github.com/carloskiki/tinycbor, cloned to `vendor/tinycbor` at `a7ec369` (`main`, 2026-07-29, "bump version") |
| Released == HEAD? | **no**. The repo HEAD manifest says `version = "0.15.0"` [read: vendor/tinycbor@a7ec369 Cargo.toml:6], and the published 0.15.1 was cut from an unpushed commit (`bcedf72`: "not our ref"). `src/lib.rs` is the only differing source file: 0.15.1 adds `Memo::bytes()` plus formatting [ran: md5 of every `src/*.rs`, tarball vs HEAD]. The derive source at HEAD is identical to tinycbor-derive 0.15.4 [ran: same check]. Tags are confusing: lib tags stop at `v0.11.2`, and derive tags use derive versions (`v0.15.1-derive`) [ran: `git tag -l`] |
| License | MIT [read: crate tinycbor-0.15.1 Cargo.toml.orig:9] |

Citation convention: `[read: crate <name>-<ver> <path>:<line>]` = published tarball; `[read: vendor/<dir>@<sha7> <path>:<line>]` = git checkout; `[ran: <command> → <result>]`.

## Verdict

- **The latest releases (0.14.0, 0.15.0, 0.15.1) do not compile without `alloc`.**
  - `encode_map!` is defined under `#[cfg(feature = "alloc")]` but invoked unconditionally [read: crate tinycbor-0.15.1 src/container/map.rs:48-49,72].
  - [ran: `cargo build --target thumbv7m-none-eabi` with `default-features = false` → `error: cannot find macro encode_map` for 0.15.1, 0.15.0, and 0.14.0; **0.13.0 builds**.]
  - Upstream's own `no-std` CI job has failed on every commit since `fa893e3` (2026-07-28) [ran: `gh run list --workflow no-std`].
  - The last no-alloc-capable pair is tinycbor 0.13.0 + tinycbor-derive 0.15.3.
- **tinycbor-derive cannot express text-string keys.** Map keys are `#[n(<u64>)]` only. Decoding is deliberately strict: **unknown keys and repeated keys are errors** [read; ran]. A text-keyed struct needs hand-written `Encode`/`Decode` impls; that works with no allocator [ran: `experiments/probes/tinycbor`].
- **Without alloc there is no generic "skip unknown value"**: `Any` and `stream` are `alloc`-only. A no-alloc decoder cannot ignore unknown keys unless it hand-rolls a skipper [read].
- Strongest RFC 8949 encoder of this pair:
  - shortest integers **and shortest floats down to f16**, with built-in conversion and no `half` dependency [read; ran: `1.5f32` → `f9 3e00`];
  - strict `simple` decoding.
  - There is **no key sorting** (derive emits declaration order) and no depth limit [read; ran].
- Bus factor 1: one author, 0 stars, about 3.3 k downloads [ran].

## 1. no_std / alloc

- `#![cfg_attr(not(any(feature = "std", test)), no_std)]`, plus `extern crate alloc` under `feature = "alloc"` [read: crate tinycbor-0.15.1 src/lib.rs:50-53].
- Features: **`default = ["std"]`**, `alloc = ["embedded-io/alloc", "const-hex/alloc"]`, `std = ["alloc", ...]`. There is a hard dependency on `embedded-io = "0.7.1"`, which provides the `Write` sink [read: crate tinycbor-0.15.1 Cargo.toml.orig:25-31], [read: crate tinycbor-0.15.1 src/lib.rs:55]. no_std users must set `default-features = false`.
- **Regression since 0.14.0**: the alloc-gated `encode_map!` is invoked for `[(K, V)]` unconditionally [read: crate tinycbor-0.15.1 src/container/map.rs:48-49,72]; [read: vendor/tinycbor@a7ec369 src/container/map.rs:48-49,72].
  - [ran: scratchpad `probes/tinycbor-nodefault-<v>` (empty `#![no_std]` lib), `cargo build --target thumbv7m-none-eabi`:
    - `=0.15.0` → `error: cannot find macro encode_map in this scope`;
    - `=0.14.0` → same error;
    - `=0.13.0` → Finished.]
  - 0.15.1 fails the same way in the probe project [ran: `cargo test --lib` with `tinycbor = { version = "=0.15.1", default-features = false }` → same error at `container/map.rs:72`].
- CI has a `no-std` job (`cargo check --target thumbv7m-none-eabi --no-default-features`) [read: vendor/tinycbor@a7ec369 .github/workflows/nostd.yml:20,32]. It has been red since commit `fa893e3` "use u64 instead of usize when appropriate" (2026-07-28). The last green run is `68df149`, the 0.13.0 release commit [ran: `gh run list -R carloskiki/tinycbor --workflow no-std`].
- No-alloc link with 0.13.0 [ran: `cargo build --release --target thumbv7m-none-eabi` in `experiments/probes/tinycbor` (no_main bin, no `#[global_allocator]`) → Finished; `size` text = 3513 B (informational)].
- [ran: `cargo tree --target thumbv7m-none-eabi -e features` → `tinycbor v0.13.0` → `embedded-io feature "default"` (embedded-io's default set has no alloc/std)].

## 2. API model

- Own traits `Encode` (`fn encode<W: Write>(&self, &mut Encoder<W>) -> Result<(), W::Error>`, i.e. infallible apart from the sink), `Decode<'b>` (with an associated `Error` type per impl), and `CborLen` [read: crate tinycbor-0.15.1 src/lib.rs:607-623].
- A separate proc-macro crate, `tinycbor-derive`, derives all three. tinycbor does not re-export it, so users depend on both [read: crate tinycbor-0.15.1 src/lib.rs:14], [read: crate tinycbor-0.15.1 Cargo.toml.orig:33-38].
- `Encoder<W>(pub W)` over `embedded_io::Write`; `Decoder<'b>(pub &'b [u8])` (slice only; streaming via `stream`, alloc) [read: crate tinycbor-0.15.1 src/lib.rs:117-119,197]. `ArrayVisitor` / `MapVisitor::with_key` support hand-written heterogeneous decoding [read: crate tinycbor-0.15.1 src/lib.rs:256-300,755-850]. A `Token` iterator is available [read: crate tinycbor-0.15.1 src/lib.rs:380-412].
- Design axioms: simplicity, compile-time correctness, infallible encoding, user-defined decode errors, **strict decoding**, smallest numeric representation [read: crate tinycbor-0.15.1 DESIGN.md:1-40].

## 3. Derive and mapping

- **Text-string keys**: **no**. `#[cbor(map)]` keys are unsigned integers given by `#[n(<u64>)]` [read: crate tinycbor-derive-0.15.4 src/lib.rs:52-61]. The attribute parser takes a `u64` literal only [read: crate tinycbor-derive-0.15.4 src/ast.rs:1037-1066]. [ran: scratchpad `probes/tinycbor-derive-check`, text key `"off"` → `Err(Content(Content(Key(InvalidHeader))))`]. A text-keyed struct requires a hand-written impl (see Probe).
- **Integer keys**: yes, unsigned only (`u64`); no negative keys [read: crate tinycbor-derive-0.15.4 src/ast.rs:1049,1064].
- **Struct as array**: the default. Exact field count is required; a missing or extra element is an error [read: crate tinycbor-derive-0.15.4 src/lib.rs:17-22].
- **Enum representations**: array `[variant_tag: u64, fields...]`; `#[cbor(naked)]` drops the outer array (C-like enum → bare integer) [read: crate tinycbor-derive-0.15.4 src/lib.rs:138-169,294-316].
- **`None` omitted on encode**: only with `#[cbor(optional)]` on a map field. The entry is omitted when the value equals `Default::default()`, and the map count is adjusted [read: crate tinycbor-derive-0.15.4 src/lib.rs:98-103], [read: crate tinycbor-derive-0.15.4 src/ast.rs:829-866]. [ran: derive-check, `len: None` + `optional` → map header `a3` (3 of 4 fields)]. Without `optional`, `None` → `null`.
- **Unknown keys ignored on decode**: **no, by design**. "An unknown or repeated key causes a decoding error" [read: crate tinycbor-derive-0.15.4 src/lib.rs:56-58]. Generated code returns `Surplus` for unmatched keys [read: crate tinycbor-derive-0.15.4 src/ast.rs:692-730]. [ran: derive-check, unknown key 9 → `Err(Content(Surplus))`].
- **Renames**: n/a. Field names are never encoded; indices are explicit.
- Other attributes: `tag(<u64>)` on container or field, `with` / `encode_with` / `decode_with` (type-based adapters), `recursive` (boxes the error, needs alloc), `error = "<Ident>"`, and bound overrides [read: crate tinycbor-derive-0.15.4 src/lib.rs:171-432].

## 4. Zero-copy decode

- `Decode for &'a [u8]` and `&'a str` borrow from the input, **definite length only**: indefinite input → `InvalidHeader` [read: crate tinycbor-0.15.1 src/bytes.rs:9-27], [read: crate tinycbor-0.15.1 src/string.rs:25-44]. `&'a [u8; N]` also borrows [read: crate tinycbor-0.15.1 src/bytes.rs:90-106].
- [ran: `experiments/probes/tinycbor` test `decodes_borrowing_from_input` → `ptr::eq(decoded.sha, input[28..])` ok].

## 5. Encode into `&mut [u8]` / decode from `&[u8]` without an allocator

- Encode: `Encoder(&mut [u8])` uses embedded-io's `impl Write for &mut [u8]`, which advances the slice and returns `SliceWriteError::Full` [read: embedded-io-0.7.1 src/impls/slice_mut.rs:32-52]. Bytes written = `capacity - encoder.0.len()`.
- Decode: `Decoder(&[u8])` directly.
- [ran: `cargo test --lib` in `experiments/probes/tinycbor` (tinycbor 0.13.0, `default-features = false`) → 4 passed, 1 ignored; thumbv7m no-allocator release link OK].
- **Only for tinycbor ≤ 0.13.0** (§1).

## 6. RFC 8949 coverage

| Item | Encode | Decode | Evidence |
|---|---|---|---|
| Indefinite-length items | yes: `begin_array/map/bytes/str` + `end` | `bytes_iter`/`str_iter`/`array_visitor`/`map_visitor` handle both forms; borrowed `&[u8]`/`&str` reject indefinite; `Vec<u8>`/`String` accept them (alloc) | [read: crate tinycbor-0.15.1 src/lib.rs:127-165,204-300], [read: crate tinycbor-0.15.1 src/bytes.rs:20], [read: crate tinycbor-0.15.1 src/bytes.rs:44-56] |
| Half-precision (f16) | yes, built-in conversion (no `half` dep) | yes: `f32`/`f64` decode accept `f9`/`fa`/`fb`; `f32` errors on lossy narrowing from `fb` | [read: crate tinycbor-0.15.1 src/num.rs:523-645,647-717] |
| Preferred (shortest) integer/length heads | yes | accepts non-preferred heads (no strictness check) | [read: crate tinycbor-0.15.1 src/lib.rs:173-190,331-341] |
| Preferred (shortest) floats | **yes**: f64 → f32 → f16 when lossless | n/a | [read: crate tinycbor-0.15.1 src/num.rs:673-685,719-735], [read: crate tinycbor-0.15.1 DESIGN.md:37-40]; [ran: derive-check `1.5f32` → `f9 3e 00`] |
| Tags | yes: `tag::Tagged<T, const N: u64>`, `IanaTag` enum, derive `tag(..)` | yes; a wrong tag → `InvalidTag` | [read: crate tinycbor-0.15.1 src/tag.rs:67-170,179-235] |
| Simple values incl. undefined | yes: `primitive::{Simple, Null, Undefined}` | yes, and **rejects the non-well-formed `f8 <0x20`** two-byte form | [read: crate tinycbor-0.15.1 src/primitive.rs:38-160] |
| Bignums (tags 2/3) | raw only (`Tagged<&[u8], 2>`); no `u128` encode | `u128` decode = native u64 range only; no bignum → integer conversion | [read: crate tinycbor-0.15.1 src/tag.rs:307-311], [read: crate tinycbor-0.15.1 src/num.rs:418-424] |
| Deterministic encoding | **none**: derive emits map fields in declaration order; collection maps are emitted in iteration order (BTreeMap = Rust `Ord`, not CBOR byte order); no §4.2.1 or §4.2.3 sort. The derive docs say "encoding is cannonical whenever possible" | n/a | [read: crate tinycbor-derive-0.15.4 src/lib.rs:4], [read: crate tinycbor-derive-0.15.4 src/ast.rs:855-872], [read: crate tinycbor-0.15.1 src/container/map.rs:48-90]; [ran: derive-check fields `#[n(3)]`,`#[n(1)]`,`#[n(2)]`,`#[n(0)]` → `a3 03 00 01 41 01 00 41 02` (declaration order, not 0,1,3)] |
| Duplicate-key detection | n/a | **yes for derived maps** (the guard `#index if var.is_none()` makes a repeat fall through to `Surplus`); no generic map-level check for collections | [read: crate tinycbor-derive-0.15.4 src/ast/field.rs:334-338]; [ran: derive-check duplicate key 3 → `Err(Content(Surplus))`] |
| Nesting-depth limit / recursion safety | n/a | **no explicit limit**. Decoding is type-directed, so depth is bounded by the static type except recursive types (`#[cbor(recursive)]` + `Box`, alloc). The generic skipper `stream::Any` uses a heap `Vec<Frame>` stack (alloc), not recursion | [read: crate tinycbor-derive-0.15.4 src/lib.rs:213-244], [read: crate tinycbor-0.15.1 src/stream.rs:17-25], [read: crate tinycbor-0.15.1 src/lib.rs:20-26] |

Generic skipping of an unknown item is `Any<'a>`, which is `#[cfg(feature = "alloc")]` [read: crate tinycbor-0.15.1 src/lib.rs:1219-1224] and also alloc-gated in the probe version [read: crate tinycbor-0.13.0 src/lib.rs:1141-1144].

## 7. RFC claims in docs

- README: "A minimal [CBOR](https://tools.ietf.org/html/rfc8949) implementation." [read: crate tinycbor-0.15.1 README.md:3].
- lib docs: "designed from scratch, but is inspired by `minicbor`"; "designed with untrusted inputs in mind, but does not bound allocations to a fixed maximum size" [read: crate tinycbor-0.15.1 src/lib.rs:1-26].
- There is no claim of deterministic-encoding conformance beyond the derive's "cannonical whenever possible" [read: crate tinycbor-derive-0.15.4 src/lib.rs:4].

## 8. Maintenance

- Last commit: `a7ec369` 2026-07-29, Charles Edward Gagnon, "bump version". Last commit touching `src/`: `f833bdc` 2026-07-29, "remove mod" [ran: `git log -1`; `git log -1 -- src derive/src`].
- Last release: tinycbor 0.15.1 (2026-07-31), tinycbor-derive 0.15.4 (2026-07-28). Releases are very frequent: 0.13.0 (07-25), 0.14.0 (07-28), 0.15.0 (07-30), 0.15.1 (07-31) [ran: crates.io API]. Total downloads ≈ 3.3 k (tinycbor), ≈ 2.8 k (derive) [ran: crates.io API].
- GitHub: 0 stars, not archived, pushed 2026-07-29. 0 open issues; 6 open PRs, all dependabot [ran: `gh api repos/carloskiki/tinycbor`; `gh pr list`].
- Contributors in the last 12 months: Charles Edward Gagnon 187 commits; dependabot 5 (bot) [ran: `git shortlog -sne --since="12 months ago" HEAD`]. That is one human maintainer.
- `unsafe`: 7 occurrences, all in `tinycbor`:
  - pointer casts for `#[repr(transparent)]` wrappers `U8`, `Tagged`, `Encoded`;
  - `MaybeUninit` handling in fixed-array decode.
  - The derive has none [ran: `grep -rn unsafe` on the 0.15.1 / derive-0.15.4 tarballs → 7]; [read: crate tinycbor-0.15.1 src/num.rs:249,268,275], [read: crate tinycbor-0.15.1 src/tag.rs:111], [read: crate tinycbor-0.15.1 src/lib.rs:1062], [read: crate tinycbor-0.15.1 src/container/bounded.rs:84,92]. 0.13.0 also has 7 [ran].
  - CI runs a `safety` workflow (sanitizers) [read: vendor/tinycbor@a7ec369 .github/workflows/safety.yml].

## 9. Embedded-relevant open issues

- There are no issues at all (0 open; no hits for no_std, alloc, heapless, embedded, size, canonical, deterministic, derive, or string keys) [ran: `gh search issues --repo carloskiki/tinycbor <kw> --include-prs`].
- Open PRs are dependabot only: #6, #9, #10, #14, #15, #16 [ran: `gh pr list -R carloskiki/tinycbor`].
- The embedded-relevant finding is unreported upstream: **no-default-features build broken since 0.14.0**, with CI `no-std` red since 2026-07-28 [ran].

## Probe

- `experiments/probes/tinycbor/`: hand-written `tinycbor::Encode` / `Decode<'b>` for `Chunk<'a> { off: u32, data: &'a [u8], len: u32, sha: &'a [u8] }`. Decode uses `Decoder::map_visitor().with_key::<&str, ..>` and folds into a `Fields` accumulator, rejecting duplicate and unknown keys. This is **not declarative**: tinycbor-derive has no text keys.
- Dependency lines: `tinycbor = { version = "=0.13.0", default-features = false }`, `embedded-io = { version = "=0.7.1", default-features = false }` (for `SliceWriteError`). **0.14.0+ will not build without alloc.**
- Host tests [ran: `cargo test --lib` → 4 passed, 1 ignored]:
  - passing: exact 60-byte encoding; borrowed decode (`ptr::eq`); any key order accepted with an unknown key rejected (`UnknownKey`); short-buffer error.
  - **ignored**: `decode_ignores_unknown_key_and_accepts_any_key_order`. The library offers no no-alloc way to skip an unknown value (`Any` is alloc-only), so the probe errors on the unknown `"upgrade": true` key, matching tinycbor's strict-decoding axiom [read: crate tinycbor-0.15.1 DESIGN.md:29-35].
- Embedded build [ran: `cargo build --release --target thumbv7m-none-eabi` → Finished with no allocator; text = 3513 B (informational)].
- Lints [ran: `cargo fmt`; `cargo clippy --target thumbv7m-none-eabi -- -D warnings` → clean; `cargo clippy --lib --tests -- -D warnings` → clean].
- Scratchpad-only checks:
  - `probes/tinycbor-derive-check`: tinycbor 0.13.0 + tinycbor-derive 0.15.3, int-keyed `#[cbor(map)]`, host test plus a thumbv7m build. Shows the encode order, `optional` omission, unknown/duplicate/text-key errors, and f16 shortening.
  - `probes/tinycbor-nodefault-*`: the version bisection above.
