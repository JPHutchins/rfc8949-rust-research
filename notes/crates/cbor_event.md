# cbor_event — evidence notes

| Field | Value |
|---|---|
| Released version | 3.3.0 (2026-08-03) — source: https://static.crates.io/crates/cbor_event/cbor_event-3.3.0.crate (`.cargo_vcs_info.json` sha `865473a`) |
| Repository | https://github.com/primetype/cbor_event — cloned to `vendor/cbor_event` at `865473a` (master, 2026-08-03) |
| Released == HEAD? | yes [ran: `diff -rq vendor/cbor_event/src <tarball>/src` → identical; HEAD = release commit `865473a`] |
| License | MIT OR Apache-2.0 [read: crate cbor_event-3.3.0 Cargo.toml.orig:7] |

Citation convention: `[read: crate <name>-<ver> <path>:<line>]` = published tarball; `[read: vendor/<dir>@<sha7> <path>:<line>]` = git checkout; `[ran: <command> → <result>]`.

## Verdict

- `#![no_std]` but **unconditionally requires `alloc`** and has no features. The `Deserializer` owns a `Vec<u8>`, and the `Serializer` writes into a `Vec<u8>`. A no-allocator thumbv7m link fails [ran].
- It is a low-level event/token codec with its own `Serialize` / `Deserialize` traits and no derive. A text-keyed struct needs a hand-written mapping. Decode copies every string (`Vec<u8>` / `String`), so there is no zero-copy.
- The 3.x line (2026-07/08) became very spec-focused:
  - preferred (shortest) floats, including f16, on encode;
  - f16 decode;
  - width-preserving `_sz` APIs for byte-exact round-trips;
  - indefinite lengths in both directions.
- It has **no deterministic map ordering, no duplicate-key detection, and no depth limit**. Decoding 1,000,000 nested arrays into `Value` overflowed the stack [ran].
- MSRV is `rust-version = "1.96.1"`, newer than the installed stable 1.95.0. Building needs `--ignore-rust-version` or a newer toolchain [ran].

## 1. no_std / alloc

- `#![no_std]` [read: crate cbor_event-3.3.0 src/lib.rs:103], then unconditional `extern crate alloc;` [read: crate cbor_event-3.3.0 src/lib.rs:107].
- The manifest has no `[features]` and no `[dependencies]`; there are only dev-deps [read: crate cbor_event-3.3.0 Cargo.toml.orig:1-19].
- `Deserializer { data: Vec<u8>, offset: usize }` [read: crate cbor_event-3.3.0 src/de.rs:235-238]. `Serializer { data: Vec<u8> }` [read: crate cbor_event-3.3.0 src/se.rs:359-361].
- The no-allocator link fails [ran: `cargo build --release --target thumbv7m-none-eabi --ignore-rust-version` on a `#![no_std] #![no_main]` bin depending only on `cbor_event = "=3.3.0"` → `error: no global memory allocator found but one is required`].
- MSRV [ran: same build without `--ignore-rust-version` → `error: rustc 1.95.0 is not supported by the following package: cbor_event@3.3.0 requires rustc 1.96.1`]; [read: crate cbor_event-3.3.0 Cargo.toml.orig:5], [read: crate cbor_event-3.3.0 rust-toolchain.toml:2].
- History: no_std arrived through PR #13 (merged 2024-03-05), and an open PR #15 "Minimize usage of alloc" exists (§9) [ran: `git log v2.4.0..v3.0.0 | grep -i no_std`].
- Verdict: no_std + alloc: **yes**. no_std without alloc: **no**.

## 2. API model

The API is low-level and token/event style, with its own traits.

- `Deserializer` has typed readers: `unsigned_integer`, `bytes`, `text`, `array`, `map`, `tag`, `special`, `float`, … [read: crate cbor_event-3.3.0 src/de.rs:510-1010].
- `Serializer` has the matching `write_*` methods [read: crate cbor_event-3.3.0 src/se.rs:500-903].
- Traits are `Deserialize { fn deserialize(&mut Deserializer) }` [read: crate cbor_event-3.3.0 src/de.rs:57-61] and `Serialize` [read: crate cbor_event-3.3.0 src/se.rs:109-111].
- A dynamic `Value` tree exists [read: crate cbor_event-3.3.0 src/value.rs:61-81], which its docs discourage: "slow and bloated … may not be compatible with the targeted environment" [read: crate cbor_event-3.3.0 src/value.rs:3-10].
- There is no serde, no derive, and no reflection. The only macro is `cbor!` (serialize to Vec) [read: crate cbor_event-3.3.0 src/macros.rs:14-21].

## 3. Derive and mapping

There is no derive; every mapping is hand-written.

- **Text-string keys:** Manual only, with `write_text("off")` and `match d.text()?.as_str()`. A hand-written mapping of the `Chunk` struct round-trips and produces the expected 60 bytes in declaration order [ran: `cargo test --lib --ignore-rust-version` → `manual_text_keyed_roundtrip ... ok`].
- **Integer keys:** Manual.
- **Struct as array:** Manual. Tuple impls write fixed arrays [read: crate cbor_event-3.3.0 src/se.rs:172-197].
- **Enum representations:** None built in.
- **`None` omitted on encode:** Not applicable. The built-in `Option<T>` impl encodes **`[]` / `[x]` (array of 0 or 1)**, not null and not omission [read: crate cbor_event-3.3.0 src/se.rs:199-209], [read: crate cbor_event-3.3.0 src/de.rs:156-166]. Omitting a field is up to the hand-written code.
- **Unknown keys ignored on decode:** Manual, and only by decoding the unknown value into an allocated `Value`, because there is no skip/discard method. That is issue #3 (§9). With `d.deserialize::<Value>()` it works [ran: `unknown_key_skipped_via_value ... ok`].
- **Renames:** Not applicable.

## 4. Zero-copy decode

No.

- `bytes() -> Result<Vec<u8>>` and `text() -> Result<String>` [read: crate cbor_event-3.3.0 src/de.rs:614], [read: crate cbor_event-3.3.0 src/de.rs:671].
- The `Deserializer` must be constructed `From<Vec<u8>>`, so the input itself is copied into an owned buffer [read: crate cbor_event-3.3.0 src/de.rs:239-243].

## 5. Encode into `&mut [u8]` / decode from `&[u8]` without an allocator

No. The only constructors are `Serializer::new_vec()` / `Serializer::new(Vec<u8>)` [read: crate cbor_event-3.3.0 src/se.rs:383-392] and `Deserializer::from(Vec<u8>)`. **No-allocator use is impossible** [ran: see §1].

## 6. RFC 8949 coverage

| Item | Encode | Decode | Evidence |
|---|---|---|---|
| Indefinite-length items | yes (`Len::Indefinite` arrays/maps; chunked strings via `_sz`) | yes (arrays, maps, chunked bytes/text) | details below |
| Half-precision (f16) | yes (`write_float_sz(.., Sz::Two)`; automatic for exact values) | yes | details below |
| Preferred (shortest) integer/length heads | yes (`Sz::canonical`) | accepts non-shortest (`_sz` reports the width) | details below |
| Preferred (shortest) floats | yes for the `f32`/`f64` `Serialize` impls; **no** for `Special::Float` / `Value` (always f64) | any width accepted | details below |
| Tags | yes (`write_tag`, `write_set_tag` = 258) | yes (`tag`, `set_tag`) | [read: crate cbor_event-3.3.0 src/se.rs:812-826]; [read: crate cbor_event-3.3.0 src/de.rs:861-890] |
| Simple values incl. undefined | yes (`Special::Undefined`, `Unassigned(u8)`) | yes; rejects `f8 <0x20` and reserved `fc..fe` | [read: crate cbor_event-3.3.0 src/se.rs:848-862]; [read: crate cbor_event-3.3.0 src/de.rs:903-960] |
| Bignums (tags 2/3) | no dedicated support (generic tag + bytes) | no dedicated support | `grep -i bignum src` → no hits [ran] |
| Deterministic encoding (key order) | **none**; writes in caller order | n/a | details below |
| Duplicate-key detection | none | none (wire order and duplicates preserved) | details below |
| Nesting-depth limit / recursion safety | none | none; **stack overflow** on deep input | details below |

**Indefinite lengths.** Encode: [read: crate cbor_event-3.3.0 src/se.rs:714], [read: crate cbor_event-3.3.0 src/se.rs:778]. Decode: [read: crate cbor_event-3.3.0 src/de.rs:724]. [ran: `indefinite_and_undefined_decode ... ok` on `9f f7 ff`]

**f16.** Encode: [read: crate cbor_event-3.3.0 src/se.rs:880-897]. Decode: [read: crate cbor_event-3.3.0 src/de.rs:937-941] (software `f16_bits_to_f64` [read: crate cbor_event-3.3.0 src/de.rs:13-43]).

**Preferred integer and length heads.** Writer: [read: crate cbor_event-3.3.0 src/se.rs:440-452]. A non-shortest head is still accepted on decode [ran: `non_shortest_int_accepted ... ok` (`18 17` → 23)].

**Preferred floats.**

- The `f32`/`f64` `Serialize` impls pick the smallest exact width [read: crate cbor_event-3.3.0 src/se.rs:99-107], [read: crate cbor_event-3.3.0 src/se.rs:142-161].
- `Special::Float` always serializes as f64 [read: crate cbor_event-3.3.0 src/types.rs:84-87].
- Verified [ran: `f64_serialize_is_preferred_width ... ok`: `1.5f64` → `f9 3e00`, `1000000.5f64` → `fa 49742408`, matching the RFC 8949 §4.2.1 examples].

**Deterministic encoding.** There is no sort anywhere. `Value::Object` is a `Vec<(Value, Value)>` documented as "key-order-preserving … rather than deduplicating or sorting" [read: crate cbor_event-3.3.0 src/value.rs:69-73].

**Duplicate keys.** The `Value::Object` docs cite §5.6 as leaving duplicate-key handling to the protocol [read: crate cbor_event-3.3.0 src/value.rs:69-73]. A map `{100:0, -1:0, 100:1}` round-trips byte-identically [ran: `value_map_keeps_wire_order_and_duplicates ... ok`]. Decoding into `BTreeMap` silently keeps the last value [read: crate cbor_event-3.3.0 src/de.rs:143-154].

**Nesting depth.** `Value` deserialization recurses without a counter [read: crate cbor_event-3.3.0 src/value.rs:126-170]. [ran: host bin decoding `[[[…0…]]]`: depth 10 000 → `ok=true`; depth 1 000 000 → `thread 'main' has overflowed its stack`, exit 134]

## 7. RFC claims in docs

- The crate docs repeatedly cite RFC 8949. Example: encoding "writes the smallest width that preserves the value (RFC 8949 §4.1 preferred serialization)" [read: crate cbor_event-3.3.0 src/lib.rs:31-36]. The `Value` round-trip guarantees are framed against "CBOR's data model (RFC 8949 §2)" [read: crate cbor_event-3.3.0 src/value.rs:39-41].
- Count across `src`: 47 mentions of "RFC 8949" and 50 lowercase "rfc7049" strings, the latter mostly in test-vector comments [ran: `grep -rn "RFC\|rfc" src | grep -o … | sort | uniq -c`].
- The README makes no RFC claim. It describes "a simple, yet efficient CBOR binary parser/serialiser" with "zero dependencies" [read: crate cbor_event-3.3.0 README.md:7-12].

## 8. Maintenance

- Last release: 3.3.0, 2026-08-03. Before that: 3.2.0 on 2026-07-09, 3.1.0 on 2026-07-08, and 3.0.0 on 2026-07-06; the release before those was 2.4.0 on 2023-01-11 [ran: `git tag --sort=-creatordate` + `git log -1 --format=%cI <tag>`; crates.io API].
- Last commit: `865473a` 2026-08-03, Nicolas Di Prima, "chore: Release cbor_event version 3.3.0" (human) [ran: `git log -1`].
- Human authors in the last 12 months: 2 (Sebastien Guillemot 23, Nicolas Di Prima 22) [ran: `git shortlog -sne --since="12 months ago" HEAD`]. The 3.x work came from dcSpark PRs #33 and #34 [ran: `git log -12`].
- GitHub: 3 open issues and 2 open PRs (`open_issues_count` = 5); not archived; pushed 2026-08-03; 11 stars [ran: `gh api repos/primetype/cbor_event`, `gh issue list`, `gh pr list`].
- `unsafe`: 0 occurrences in `src` [ran: `grep -rn unsafe src | wc -l` → 0].

## 9. Embedded-relevant open issues

- https://github.com/primetype/cbor_event/pull/15 — "Minimize usage of alloc" (open PR since 2024-05-29, author SimonIT: "I opened it to get some opinions").
- https://github.com/primetype/cbor_event/issues/3 — "Please provide a method to deserialize and discard one object" (open since 2020-08-24). This blocks skipping unknown keys without allocating a `Value`.
- https://github.com/primetype/cbor_event/pull/18 — "feature: Support f16" (open since 2024-12-07). It is referenced as the future replacement for the vendored f16 code [read: crate cbor_event-3.3.0 src/de.rs:17-19].
- https://github.com/primetype/cbor_event/issues/1 — "Deserializing an indefinite array of bools (or map with bool keys) fails" (open since 2020-08-24).
- https://github.com/primetype/cbor_event/issues/5 — "Fails with `NotEnough` error in arbitrary parsing functions" (open since 2020-09-07).

## Probe

**Not eligible for the shortlist**, because it requires `alloc`. The probes live only in the scratchpad, not in `experiments/`.

No-alloc check (failing). It uses a `#![no_std] #![no_main]` bin calling `Serializer::new_vec().write_unsigned_integer(7)`, with dependency `cbor_event = { version = "=3.3.0" }`:

```text
$ cargo build --release --target thumbv7m-none-eabi
error: rustc 1.95.0 is not supported by the following package:
  cbor_event@3.3.0 requires rustc 1.96.1
$ cargo build --release --target thumbv7m-none-eabi --ignore-rust-version
error: no global memory allocator found but one is required; link to std or add `#[global_allocator]` to a static item that implements the GlobalAlloc trait
```

Host behaviour tests: 6/6 pass [ran: `cargo test --lib --ignore-rust-version`]. Hand-written text-keyed mapping used:

```rust
s.write_map(Len::Len(4))?
    .write_text("off")?.write_unsigned_integer(c.off)?
    .write_text("data")?.write_bytes(&c.data)?
    .write_text("len")?.write_unsigned_integer(c.len)?
    .write_text("sha")?.write_bytes(&c.sha)?;
// decode
let mut d = Deserializer::from(buf.to_vec());
d.map_with(|d| {
    match d.text()?.as_str() {
        "off" => c.off = d.unsigned_integer()?,
        "data" => c.data = d.bytes()?,
        "len" => c.len = d.unsigned_integer()?,
        "sha" => c.sha = d.bytes()?,
        _ => { d.deserialize::<Value>()?; }
    }
    Ok(())
})?;
```
