# ciborium (+ ciborium-ll, ciborium-io) — evidence notes

| Field | Value |
|---|---|
| Released version | ciborium / ciborium-ll / ciborium-io 0.2.2 (2024-01-24). Sources: https://static.crates.io/crates/ciborium/ciborium-0.2.2.crate (and `-ll`, `-io`); `.cargo_vcs_info.json` sha `fdf3ec5` for all three |
| Repository | https://github.com/enarx/ciborium, cloned to `vendor/ciborium` at `00279e4` (`main`, 2026-06-18, a dependabot commit) |
| Released == HEAD? | **no**. There are 59 commits after `fdf3ec5`. Unreleased changes include `canonical_into_writer`/`canonical_into_vec` (`1b60854`, 2025-06-10), an optional `half` dependency (`0ba5b06`), and an MSRV bump. Manifest versions are unchanged at 0.2.2. `extern crate alloc` is still unconditional at HEAD [read: vendor/ciborium@00279e4 ciborium/src/lib.rs:93] |
| License | Apache-2.0 |

Citation convention: `[read: crate <name>-<ver> <path>:<line>]` = published tarball; `[read: vendor/ciborium@00279e4 <path>:<line>]` = git checkout; `[ran: <command> → <result>]`.

## Verdict

- **Premise "ciborium is not no_std" is false as worded.** With `default-features = false`, `ciborium` builds as `#![no_std]`, but it **requires `alloc`**. A no-allocator thumbv7m binary fails to link [ran], and the same binary links once a `#[global_allocator]` is supplied [ran].
- **`ciborium-ll` and `ciborium-io` are no_std and no-alloc.** A hand-written text-keyed struct codec over `ciborium-ll` links on thumbv7m without an allocator [ran]. But that is a token/header API: no derive, no skip helper, and reader-based decoding that copies.
- The serde path has **no zero-copy** decode (`from_reader` needs `DeserializeOwned`) and stages strings and byte strings through a 4096-byte stack scratch buffer.
- Encoding uses preferred serialization for both ints and floats, including f16 [ran]. Released canonical support is limited to `value::CanonicalValue`, which applies **RFC 7049 §3.9 / RFC 8949 §4.2.3 length-first** ordering to `Value` keys. There is no §4.2.1 mode.
- Maintenance is slow: last release 2024-01-24, 46 open issues and 11 open PRs. Issue #90 ("no-alloc as option for ciborium") has been open since 2023-08 with no maintainer reply.

## 1. no_std / alloc

- `ciborium`: `#![cfg_attr(not(feature = "std"), no_std)]` [read: crate ciborium-0.2.2 src/lib.rs:87]; **unconditional** `extern crate alloc;` [read: crate ciborium-0.2.2 src/lib.rs:93]; depends on `ciborium-io` with `features = ["alloc"]` and `serde` with `features = ["alloc", "derive"]` [read: crate ciborium-0.2.2 Cargo.toml.orig:25-26]; `default = ["std"]`, `std = ["ciborium-io/std", "serde/std"]` [read: crate ciborium-0.2.2 Cargo.toml.orig:35-36].
- `ciborium-ll`: "usable in both `no_std` and `no_alloc` environments" [read: crate ciborium-ll-0.2.2 src/lib.rs:6]; features `alloc = []`, `std = ["alloc", "half/std"]`; `half` is non-optional in 0.2.2 [read: crate ciborium-ll-0.2.2 Cargo.toml.orig:24-32].
- `ciborium-io`: `alloc = []`, `std = ["alloc"]` [read: crate ciborium-io-0.2.2 Cargo.toml.orig:24-25]. Without `std` it implements `Read for &[u8]` and `Write for &mut [u8]` [read: crate ciborium-io-0.2.2 src/lib.rs:108-149]. With `std` these become blanket impls over `std::io` [read: same, 52-70].
- [ran: scratchpad `probes/ciborium-noalloc` (no_main bin, `ciborium = { version = "=0.2.2", default-features = false }`), `cargo build --release --target thumbv7m-none-eabi` → `error: no global memory allocator found but one is required`]. `cargo tree -e features -i ciborium-io` shows `ciborium-io feature "alloc"` enabled by `ciborium v0.2.2`.
- [ran: scratchpad `probes/ciborium-withalloc` (the same plus a `#[global_allocator]`) → Finished; text 31118 B (informational)].
- [ran: `experiments/probes/ciborium-ll` (no_main bin, `ciborium-ll`/`ciborium-io` with `default-features = false`) → Finished; text 4529 B (informational)].
- Maintainer statement (2024-02-20, issue #59): "The `ciborium` crate requires `alloc` … The `ciborium-io` and `ciborium-ll` crates already have an `alloc` feature which seems to work as intended." (https://github.com/enarx/ciborium/issues/59)

## 2. API model

- `ciborium`: serde `Serializer`/`Deserializer` behind `into_writer` / `from_reader*`, plus a dynamic `value::Value` [read: crate ciborium-0.2.2 src/lib.rs:5-14; src/ser/mod.rs:490; src/de/mod.rs:825-871]. Serializer and Deserializer types are not public (issues #18 and #62).
- `ciborium-ll`: low-level `Header` enum plus `Encoder::push` / `Decoder::pull`, with segment helpers for chunked bytes/text [read: crate ciborium-ll-0.2.2 src/lib.rs:41-124; src/hdr.rs:14-60].
- No derive of its own; ciborium relies on serde derive.

## 3. Derive and mapping (serde path)

- Text-string keys: yes, as serde field names. [ran: scratchpad `probes/ciborium-host` (no_std crate with `alloc`, host test) → `into_writer(&mut [u8]) wrote 60 bytes: [a4, 63, 6f, 66, 66, 00, 64, 64, 61, 74, 61, 44, …]`, the same bytes as the minicbor-serde probe].
- Integer keys: not with serde derive. Open bug #153 covers flattened maps with integer keys.
- Struct as array: no. Packed encoding is explicitly not implemented ("We might consider adding this in the future") [read: crate ciborium-0.2.2 src/lib.rs:79-85]; issue #93.
- Enum representations: serde defaults (externally tagged `{variant: …}`) [read: crate ciborium-0.2.2 src/ser/mod.rs:218, 265-267].
- `None` omitted: only via serde `skip_serializing_if`. By default `serialize_none` writes `null` [read: crate ciborium-0.2.2 src/ser/mod.rs:171-172]; issue #100.
- Unknown keys ignored: yes. `deserialize_ignored_any` delegates to `deserialize_any` [read: crate ciborium-0.2.2 src/de/mod.rs:512-517]. [ran: `probes/ciborium-host` → `unknown key + reordered -> Ok(Chunk { off: 0, data: [], len: 123, sha: [170] })`].
- Duplicate struct fields: rejected by serde_derive [ran: `duplicate key -> Err(Semantic(None, "duplicate field `off`"))`].
- Renames: serde attributes.

## 4. Zero-copy decode

- **No.** `from_reader<T: de::DeserializeOwned, R: Read>` [read: crate ciborium-0.2.2 src/de/mod.rs:825]. Text and bytes are read into `scratch` and then passed via `visit_str` / `visit_bytes`, not `visit_borrowed_*` [read: crate ciborium-0.2.2 src/de/mod.rs:327-331, 369-371]. Items larger than the scratch buffer, and indefinite text, are built into `String`/`Vec` [read: same, 343-353, 390-400].
- `from_reader` puts `let mut scratch = [0; 4096];` on the stack [read: crate ciborium-0.2.2 src/de/mod.rs:829]. `from_reader_with_buffer` lets the caller size it [read: same, 837-847].
- Open: issue #36 "Request for zero-copy deserializing" (2021) and PR #117 "feat: support zero-copy deserialization" (2024-02, unmerged); PR #170 "Implement `from_slice` deserialization" (2026-02, unmerged).
- `ciborium-ll`: `Decoder<R: Read>` copies bodies out via `read_exact` or segment pulls [read: crate ciborium-ll-0.2.2 src/dec.rs:48-57, 147-170].

## 5. Encode into `&mut [u8]` / decode from `&[u8]` without an allocator

- `ciborium` (serde): **no**. alloc is required (§1). With an allocator, `into_writer(&x, &mut &mut [u8])` and `from_reader(&[u8])` work [ran: `probes/ciborium-host`].
- `ciborium-ll`: **yes, manual**. [ran: `experiments/probes/ciborium-ll`: `cargo test --lib` → 4 passed (exact 60-byte encoding; decode copies `data`/`sha` into a caller scratch slice, with a test asserting the pointer is *not* into the input; an unknown key is skipped by the probe's own skip loop; a short buffer errors); `cargo build --release --target thumbv7m-none-eabi` → Finished; clippy (thumb and host tests) `-D warnings` clean]. Everything above the header level (key matching, skipping, missing-field errors) is hand-written in the probe.

## 6. RFC 8949 coverage

| Item | Encode | Decode | Evidence |
|---|---|---|---|
| Indefinite-length items | serde `serialize_seq/map(None)` → `Header::Array(None)`/`Map(None)`; `ll::Encoder::bytes/text(_, Some(seg))` emits chunked strings | yes | [read: crate ciborium-0.2.2 src/ser/mod.rs:226-227, 278; crate ciborium-ll-0.2.2 src/enc.rs:66-120] [ran: `experiments/conformance/appendix-a/ciborium-ll` → all indefinite vectors round-trip] |
| Half-precision (f16) | yes, automatically when lossless | yes | [read: crate ciborium-ll-0.2.2 src/hdr.rs:111, 145-158] |
| Preferred (shortest) integer/length heads | yes ("Always Serialize Numeric Values to the Smallest Size") | accepts non-preferred | [read: crate ciborium-0.2.2 src/lib.rs:17-33] [ran: `experiments/conformance/appendix-a/ciborium-ll` asserts `1a00000001` → `Header::Positive(1)`] |
| Preferred (shortest) floats | **yes** (f64 → f16 / f32 when bit-exact) | n/a | [read: crate ciborium-ll-0.2.2 src/hdr.rs:145-158] [ran: `probes/ciborium-host` → `f64 1.5 -> [f9, 3e, 00]`; Appendix A f32/f64 ±Inf and NaN vectors re-encode as `f97c00`/`f97e00`/`f9fc00`] |
| Tags | yes (`tag::Required<V, TAG>`, `Accepted`, `Captured`) | yes | [read: crate ciborium-0.2.2 src/tag.rs:15-72] |
| Simple values incl. undefined | ll: `Header::Simple(u8)`; serde: none is `null`, no undefined producer (PR #67 open) | `undefined` → `None`/unit; other simple values error in serde (issue #60); ll accepts the not-well-formed `f8 18`/`f8 14` | [read: crate ciborium-0.2.2 src/de/mod.rs:187-191, 522-536] [ran: conformance test asserts `f8 18` → `Header::Simple(24)`] |
| Bignums (tags 2/3) | yes, `i128`/`u128` → tag 2/3 only when outside 64-bit | yes, up to 16 bytes | [read: crate ciborium-0.2.2 src/ser/mod.rs:78-86, 124-137; src/de/mod.rs:84-85, 166] |
| Deterministic encoding | released: `value::CanonicalValue` uses **length-first** order (RFC 7049 §3.9 / RFC 8949 §4.2.3) for `Value` map keys only; no §4.2.1 bytewise mode; struct fields are emitted in declaration order. HEAD (unreleased) adds `canonical_into_writer`, still length-first | no validation | [read: crate ciborium-0.2.2 src/value/canonical.rs:26-34, 63-67; crate ciborium-0.2.2 src/value/mod.rs:12; crate ciborium-0.2.2 src/value/integer.rs:71-92; vendor/ciborium@00279e4 ciborium/src/value/canonical.rs:11-25] |
| Duplicate-key detection | n/a | none in the codec (serde_derive rejects duplicate struct fields) | [ran: `probes/ciborium-host` duplicate key → serde_derive error] |
| Nesting-depth limit / recursion safety | n/a | yes: default recursion limit 256, configurable through `from_reader_with_recursion_limit` | [read: crate ciborium-0.2.2 src/de/mod.rs:59-71, 847, 859-871] |

All 81 RFC 8949 Appendix A vectors decode with `ciborium-ll`. Re-encoding differs only for the 6 non-preferred float examples (f32/f64 Infinity/NaN/−Infinity), which ciborium shrinks to f16 [ran: `cd experiments/conformance/appendix-a/ciborium-ll && cargo test -- --nocapture` → `vectors=81 failures=6`; the test asserts that the failures are exactly these six `REENCODE-DIFF … -> f9…` entries, and that `f8 18` decodes as `Simple(24)`, which is not well-formed per RFC 8949 §3.3].

## 7. RFC claims in docs

- `ciborium-ll` docs: "For more information on the CBOR format, see [RFC 7049]" [read: crate ciborium-ll-0.2.2 src/lib.rs:46-47; README.md:52].
- `ciborium` README and lib docs name no RFC; they describe the design as "Robustness Principle … liberal in what we accept" [read: crate ciborium-0.2.2 src/lib.rs:36-47].
- `CanonicalValue` docs cite "RFC 7049 Section 3.9 (regarding key sorting) and RFC 8949 4.2.3 (as errata)" [read: crate ciborium-0.2.2 src/value/canonical.rs:26, 63].
- Open issue #182 (2026-09-14) is a third-party "spec compliance misses" report against 0.2.2.

## 8. Maintenance

- Last commit: `00279e4` 2026-06-18 by dependabot. Last human commit touching `*/src`: `7434241` 2025-11-21 Richard Zak "ci: add license header check". Last functional source change: `1a8190d` 2025-08-01 [ran: `git log -3 --no-merges -- ciborium/src ciborium-ll/src ciborium-io/src`].
- Last release: 0.2.2 on 2024-01-24, with 59 commits unreleased since [ran: crates.io API; `git log fdf3ec5..HEAD | wc -l`].
- Open: 46 issues and 11 PRs (`open_issues_count` 57); not archived; 399 stars; pushed 2026-10-21 [ran: `gh api repos/enarx/ciborium`, `gh issue list`, `gh pr list`].
- Maintainers, last 12 months: 1 human (Richard Zak, 3 non-merge commits), plus dependabot (4) [ran: `git shortlog -sn --since="12 months ago" --no-merges HEAD`].
- `unsafe`: 0 occurrences across all three crates [ran: `grep -rn unsafe` on the three tarballs' `src/`].
- Downloads: about 294 M total (ciborium), 82 M recent [ran: crates.io API].

## 9. Embedded-relevant open issues

- https://github.com/enarx/ciborium/issues/90: "no-alloc as option for ciborium" (chrysn, 2023-08-24; no comments).
- https://github.com/enarx/ciborium/issues/59: "Make sure no_std mode works". The maintainer confirms ciborium requires alloc.
- https://github.com/enarx/ciborium/issues/66: "Optionally implement Writer for heapless::Vec".
- https://github.com/enarx/ciborium/issues/172: "Support for `embedded-io-async`".
- https://github.com/enarx/ciborium/issues/36 and https://github.com/enarx/ciborium/pull/117: zero-copy deserialization (request and unmerged PR).
- https://github.com/enarx/ciborium/pull/170: "Implement `from_slice` deserialization …" (unmerged).
- https://github.com/enarx/ciborium/issues/47: "improve memory usage".
- https://github.com/enarx/ciborium/issues/93: "Packed Encoding" (integer keys / arrays).
- https://github.com/enarx/ciborium/issues/154 and https://github.com/enarx/ciborium/pull/143: deterministic encoding request, and "Canonicalization rfc8949 and rfc7049 implementation for structs" (unmerged, 2024-11).
- https://github.com/enarx/ciborium/issues/161: "Predict serialized len".

## Probe

- `experiments/probes/ciborium-ll/`: manual codec over `ciborium-ll` (not declarative). Deps: `ciborium-io = { version = "=0.2.2", default-features = false }`, `ciborium-ll = { version = "=0.2.2", default-features = false }`. 4 host tests pass; thumbv7m no-alloc bin links. Decoding copies into caller scratch (no zero-copy).
- `ciborium` (serde) is **not** shortlisted: it cannot link without an allocator. The failing probe stays in the scratchpad (`probes/ciborium-noalloc`); its error is quoted in §1.
- `experiments/conformance/appendix-a/ciborium-ll/`: RFC 8949 Appendix A header round-trip (host only).

## Appendix: scratch probe sources (not committed; reproduce by recreating them)

These probes sat in the session scratchpad and were built with `cargo build --release --target thumbv7m-none-eabi`.

`probes/ciborium-noalloc` → `error: no global memory allocator found but one is required; link to std or add #[global_allocator] …`

```toml
[package]
name = "ciborium-noalloc"
version = "0.0.0"
edition = "2024"
[dependencies]
ciborium = { version = "=0.2.2", default-features = false }
serde = { version = "1", default-features = false, features = ["derive"] }
[profile.dev]
panic = "abort"
[profile.release]
panic = "abort"
opt-level = "z"
lto = true
```

```rust
#![no_std]
#![no_main]
use core::hint::black_box;
#[derive(serde::Serialize, serde::Deserialize)]
struct Chunk {
    off: u32,
    len: u32,
}
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! { loop { core::hint::spin_loop() } }
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let mut buf = [0u8; 64];
    let ok = ciborium::into_writer(&black_box(Chunk { off: 0, len: 123 }), &mut buf[..]).is_ok();
    let back: Result<Chunk, _> = ciborium::from_reader(black_box(&buf[..]));
    black_box((ok, back.is_ok()));
    loop { core::hint::spin_loop() }
}
```

`probes/ciborium-withalloc` is the same project with the lines below added after the `use`. Result: `Finished`.

```rust
struct NullAlloc;
unsafe impl core::alloc::GlobalAlloc for NullAlloc {
    unsafe fn alloc(&self, _: core::alloc::Layout) -> *mut u8 { core::ptr::null_mut() }
    unsafe fn dealloc(&self, _: *mut u8, _: core::alloc::Layout) {}
}
#[global_allocator]
static ALLOC: NullAlloc = NullAlloc;
```

`probes/control-alloc` is the method control: no dependencies, same profiles. Result: the same "no global memory allocator found" error, which shows that the mere presence of `alloc` in the crate graph trips the check.

```rust
#![no_std]
#![no_main]
extern crate alloc;
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! { loop { core::hint::spin_loop() } }
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! { loop { core::hint::spin_loop() } }
```

`probes/ciborium-host` (host `cargo test`, `ciborium` with `default-features = false`, `serde_bytes` with `alloc`) serialized an owned-`Vec<u8>` variant of `Chunk` into `&mut [u8]` and decoded the unknown-key, duplicate-key and `f64 1.5` cases quoted in §3 and §6.
