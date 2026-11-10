# serde_cbor_2 — evidence notes

| Field | Value |
|---|---|
| Released version | 0.13.0 (2025-06-14) — source: https://static.crates.io/crates/serde_cbor_2/serde_cbor_2-0.13.0.crate (`.cargo_vcs_info.json` sha `431de98`) |
| Repository | https://github.com/kanidm/cbor — cloned to `vendor/serde_cbor_2` at `b06c72f` (main, 2026-09-14) |
| Released == HEAD? | src yes; HEAD is 30 commits past `431de98` but `git diff --stat 431de98 HEAD` touches only `.github/workflows/ci.yml` and `lint.yml` (dependabot bumps) [ran]. `diff -r` tarball `src/` vs `vendor/serde_cbor_2/src` is empty [ran] |
| License | MIT OR Apache-2.0 |

Citation convention: `[read: crate serde_cbor_2-0.13.0 <path>:<line>]` = published tarball; `[read: vendor/serde_cbor_2@b06c72f <path>:<line>]` = git checkout (src identical to release); `[ran: <command> → <result>]`. "Behaviour harness" = scratch crate `probes/serde-family-behaviour` (host, `default-features = false`).

## Verdict

- A lint-cleanup continuation of serde_cbor 0.11.2: `diff -u` of the two released `src/` trees shows only clippy/rustfmt/doc-name changes plus `half` 1 → `half` 2 (`default-features = false`) [ran: `diff -u serde_cbor-0.11.2/src serde_cbor_2-0.13.0/src`; read: crate serde_cbor_2-0.13.0 Cargo.toml.orig `[dependencies]`]. Docs call it a "'drop in' replacement of `serde_cbor`" [read: vendor/serde_cbor_2@b06c72f src/lib.rs:12].
- Therefore every behaviour in `serde_cbor.md` applies: no_std **without alloc** via `SliceWrite` + `from_slice_with_scratch`, text-keyed structs, zero-copy, unknown keys skipped, depth limit 128, shortest floats, tags need std, no bignums [ran: probe 4/4 + harness `sc2_same_behaviour`].
- Maintenance is passive: no human non-merge commit in the last 12 months; last src change 2025-06-14 [ran].
- Docs still claim RFC 7049 only [read: vendor/serde_cbor_2@b06c72f README.md:7].

## 1. no_std / alloc

- Same gating as serde_cbor: `#![cfg_attr(not(feature = "std"), no_std)]`, `extern crate alloc` only under `alloc` [read: vendor/serde_cbor_2@b06c72f src/lib.rs:326,333-334]; `from_slice`/`to_vec` need `std` or `alloc` [read: src/lib.rs:362-368]; `Value` std-only [read: src/lib.rs:343-344].
- Features: `default = ["std"]`, `alloc = ["serde/alloc"]`, `std = ["serde/std"]`, `unsealed_read_write`, `tags` [read: crate serde_cbor_2-0.13.0 Cargo.toml.orig `[features]`]. `half = { version = "2", default-features = false }` [read: same file `[dependencies]`].
- No-alloc verified: [ran: `cargo build --release --target thumbv7m-none-eabi` in `experiments/probes/serde_cbor_2` (no_main bin, no global allocator) → Finished]; [ran: same for thumbv6m-none-eabi → Finished].
- `cargo tree --target thumbv7m-none-eabi -e features`: `serde_cbor_2 → half v2.7.1 → zerocopy v0.8.63 (features derive, simd) + cfg-if`, `serde v1.0.229` [ran]. `half` 2 pulls `zerocopy` where serde_cbor's `half` 1.8 did not.
- `tags` uses `thread_local!` [read: vendor/serde_cbor_2@b06c72f src/tags.rs:220], so it needs std (same as serde_cbor; serde_cbor variant [ran]).
- Open PR #23 "move Value to alloc feature" (2025-12-21, unmerged) [ran: `gh pr view 23 -R kanidm/cbor`].

## 2. API model

- serde `Serializer`/`Deserializer` + std-only `Value` [read: vendor/serde_cbor_2@b06c72f src/lib.rs:348]. No own derive, no public token API.

## 3. Derive and mapping

Identical code paths to serde_cbor (`diff` shows no behavioural change):

- Text-string keys: yes [read: vendor/serde_cbor_2@b06c72f src/ser.rs:601-611]; [ran: probe `encodes_text_keyed_map_into_slice` → exact 60-byte vector].
- Integer keys: packed format only (field ordinal) [read: src/lib.rs:74-89].
- Struct as array: no serializer option.
- Enum representations: externally tagged map by default, `legacy_enums()` arrays, packed indices [read: src/ser.rs:84-121,390-401].
- `None` omitted: with `skip_serializing_if` (serde) — same serializer code as serde_cbor [ran: serde_cbor harness `sc_none_skipped_definite_len`].
- Unknown keys ignored on decode: yes [ran: probe `decode_ignores_unknown_key_and_accepts_any_key_order` → ok].
- Renames: serde attributes.

## 4. Zero-copy decode

- Yes for definite-length strings [read: vendor/serde_cbor_2@b06c72f src/de.rs:293,349]; [ran: probe `decodes_borrowing_from_input` pointer-equality assert → ok]. Indefinite strings need scratch or `from_mut_slice` [read: src/de.rs:58-90].

## 5. Encode into `&mut [u8]` / decode from `&[u8]` without an allocator

- Yes: `Serializer::new(ser::SliceWrite::new(buf))` / `de::from_slice_with_scratch(buf, &mut [])` [read: vendor/serde_cbor_2@b06c72f src/lib.rs:224-301].
- [ran: `experiments/probes/serde_cbor_2`: `cargo test --lib` → 4 passed; thumbv7m release build → Finished; `cargo clippy --target thumbv7m-none-eabi -- -D warnings` → clean; `cargo clippy --lib --tests -- -D warnings` → clean; `cargo fmt --check` → clean].
- Informational only: [ran: `size` thumbv7m release → text 17822 B; thumbv6m → 20674 B].

## 6. RFC 8949 coverage

Code is the same as serde_cbor 0.11.2 at the same line numbers in `de.rs`, and same logic in `ser.rs`; see `serde_cbor.md` §6 for the full table. Spot-checks run against serde_cbor_2 itself:

| Item | Encode | Decode | Evidence |
|---|---|---|---|
| Indefinite-length items | yes (unknown serde length) | yes | [read: vendor/serde_cbor_2@b06c72f src/ser.rs:197; src/de.rs:653,677,701,725] |
| Half-precision (f16) | yes | yes | [read: src/ser.rs:321; src/de.rs:567] |
| Preferred (shortest) integer/length heads | yes | accepts non-preferred | [read: src/ser.rs:141-182] |
| Preferred (shortest) floats | yes (f64→f32→f16) | n/a | [ran: harness `sc2_same_behaviour`: `1.5f64` → `f9 3e 00`] |
| Tags | `tags` feature, std-only | dropped without `tags` | [read: src/lib.rs:308-310; src/tags.rs:220] |
| Simple values incl. undefined | false/true/null | undefined → unit | [read: src/lib.rs:311-314; src/de.rs:755] |
| Bignums (tags 2/3) | no | no | [read: src/ser.rs:305] |
| Deterministic encoding | serializer: none; `Value::Map` (std) = §4.2.1 bytewise order | no check | [ran: scratch `serde_cbor-value-order` with the RFC 8949 §4.2.1 example keys → `a80af61864f620f6617af6626161f6811864f68120f6f4f6`, identical to §4.2.1 order] |
| Duplicate-key detection | n/a | codec no; serde derive rejects repeated known field | same code as serde_cbor [ran there] |
| Nesting-depth limit / recursion safety | n/a | fixed 128 | [read: src/de.rs:189,431]; [ran: harness `sc2_same_behaviour`: 200 nested arrays → `RecursionLimitExceeded`] |

## 7. RFC claims in docs

- "This crate implements the Concise Binary Object Representation from [RFC 7049]" [read: vendor/serde_cbor_2@b06c72f README.md:7,79]. `Value` ordering doc still cites draft-ietf-cbor-7049bis-04 [read: src/value/mod.rs:16-21]. No RFC 8949 mention.
- Stale docs: README usage snippet says `serde_cbor_2 = "0.11.2"` [read: README.md:17]; lib docs say "supports Rust 1.62" [read: src/lib.rs:5] while `rust-version = "1.81"` [read: crate serde_cbor_2-0.13.0 Cargo.toml.orig].

## 8. Maintenance

- Last commit: `b06c72f` 2026-09-14 (James Hodgkinson merging a dependabot PR) [ran: `git log -1`]; last commit touching `src/`: `059b96b` 2025-06-14 "fix: clippy lint about const refcell" (James Hodgkinson) [ran: `git log -1 -- src`].
- Last release: 0.13.0, 2025-06-14 (previous: 0.12.0-dev, 2022-09-16) [ran: crates.io API].
- Open issues 0, open PRs 3 (#32 dependabot, #25 StreamDeserializer API, #23 Value→alloc); `open_issues_count` 3; issues enabled [ran: `gh api repos/kanidm/cbor`, `gh pr list`].
- Maintainers in last 12 months: non-merge commits — dependabot only (8); including merges — James Hodgkinson 8, dependabot 8 [ran: `git shortlog -sn [--no-merges] --since="12 months ago"`].
- `unsafe` in `src/`: 0 [ran: `grep -rnw unsafe src`].
- Usage: 15.4 M downloads total, 5.1 M recent; 24 reverse dependencies [ran: crates.io API].

## 9. Embedded-relevant open issues

- https://github.com/kanidm/cbor/pull/23 — "feat: move Value to alloc feature" (`Value` currently std-only), open since 2025-12-21.
- No open issues at all (`gh issue list -R kanidm/cbor --state all` → none) [ran]. The upstream serde_cbor issues (stack size #167, recursion limit #166, tags #208) apply unchanged; see `serde_cbor.md` §9.

## Probe

- `experiments/probes/serde_cbor_2/` (same layout and tests as `minicbor-serde`; only the crate path differs from `probes/serde_cbor`).
- Dependency line: `serde_cbor_2 = { version = "=0.13.0", default-features = false }` plus `serde = { version = "1", default-features = false, features = ["derive"] }`, `serde_bytes = { version = "0.11", default-features = false }`.
- Results: [ran: `cargo test --lib` → 4 passed, 0 ignored]; [ran: thumbv7m + thumbv6m release builds → Finished]; [ran: fmt + both clippy invocations → clean]. Toolchain rustc 1.95.0.
