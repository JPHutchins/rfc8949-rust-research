# sk-cbor — evidence notes

| Field | Value |
|---|---|
| Released version | 0.2.0 (2026-10-22) — source: https://static.crates.io/crates/sk-cbor/sk-cbor-0.2.0.crate (`.cargo_vcs_info.json` sha `118d612`, `path_in_vcs = libraries/cbor`) |
| Repository | https://github.com/google/OpenSK (crate at `libraries/cbor`) — cloned to `vendor/sk-cbor` at `f53621c` (develop, 2026-11-09) |
| Released == HEAD? | Non-test code identical; two later commits (`90cea20`, `c9dec1f`) only touch `#[cfg(test)]` code (`core::i64::MAX` → `i64::MAX` lint cleanups) and the fuzz manifest [ran: `diff -r <tarball>/src vendor/sk-cbor/libraries/cbor/src` → differences only inside `mod test` blocks] |
| License | Apache-2.0 [read: crate sk-cbor-0.2.0 Cargo.toml.orig:4] |

Citation convention: `[read: crate <name>-<ver> <path>:<line>]` = published tarball; `[read: vendor/<dir>@<sha7> <path>:<line>]` = git checkout; `[ran: <command> → <result>]`.

## Verdict

- `no_std` but **requires `alloc` unconditionally**; there are no Cargo features at all. A no-allocator thumbv7m link fails [ran: see Probe].
- Not a typed or declarative codec. It is a dynamic `Value` tree (`Vec`/`String`/`Box`) plus `cbor_map!` and `destructure_cbor_map!` macros. Text and integer keys work through those macros. There is no derive, no zero-copy, and no slice writer.
- **Strict canonical codec.** The writer sorts map keys in RFC 8949 §4.2.1 bytewise order. I verified this with the 8949 mixed-type example. It is not RFC 7049 length-first order. The reader **rejects** unsorted or duplicate keys, non-shortest heads, indefinite lengths, all floats, and simple values other than false/true/null/undefined.
- It has a nesting limit (127 by default) on both encode and decode, and no `unsafe`.
- The strict decoder has a consequence for embedded peers: a map that arrives in declaration order (`off, data, len, sha`) is rejected with `OutOfOrderKey` [ran].

## 1. no_std / alloc

- `#![no_std]` + unconditional `extern crate alloc;` [read: crate sk-cbor-0.2.0 src/lib.rs:15], [read: crate sk-cbor-0.2.0 src/lib.rs:17].
- `[dependencies]` is empty and there is no `[features]` table, so alloc cannot be turned off [read: crate sk-cbor-0.2.0 Cargo.toml.orig:15-16].
- The `Value` representation is built from `Vec<u8>`, `String`, `Vec<Value>`, `Vec<(Value, Value)>`, and `Box<Value>` [read: crate sk-cbor-0.2.0 src/values.rs:28-45].
- The no-allocator link fails [ran: `cargo build --release --target thumbv7m-none-eabi` on a `#![no_std] #![no_main]` bin depending only on `sk-cbor = "=0.2.0"` → `error: no global memory allocator found but one is required`].
- With a `#[global_allocator]` (a null-returning stub) it links for thumbv7m [ran: `cargo build --release --target thumbv7m-none-eabi --bin withalloc` → Finished; `size` text = 11387 B, informational only].
- Verdict: no_std + alloc: **yes**. no_std without alloc: **no**. Feature flags: **none**.

## 2. API model

The API is a low-level dynamic value model. `sk_cbor::read(&[u8]) -> Result<Value, DecoderError>` and `sk_cbor::write(Value, &mut Vec<u8>) -> Result<(), EncoderError>` [read: crate sk-cbor-0.2.0 src/reader.rs:43-45], [read: crate sk-cbor-0.2.0 src/writer.rs:30-32].

There are construction macros (`cbor_map!`, `cbor_map_options!`, `cbor_array!`, `cbor_bytes!`, `cbor_text!`, `cbor_tagged!`, …) and the extraction macro `destructure_cbor_map!` [read: crate sk-cbor-0.2.0 src/macros.rs:22-78]. There is no serde, no derive, no reflection, and no streaming token API.

## 3. Derive and mapping

There is no derive. "Mapping" here means hand-written code over `cbor_map_options!` and `destructure_cbor_map!`.

- **Text-string keys:** Yes, through the macros: `"off" => …` builds `Value::TextString` keys [read: crate sk-cbor-0.2.0 src/macros.rs:134-153]. My text-keyed `Chunk` round-trips [ran: `cargo test --lib` → `text_keyed_roundtrip_and_sorted_output ... ok`]. The output order is canonical: `len, off, sha, data`, not the declaration order.
- **Integer keys:** Yes. Integer keys are the CTAP2 use case, e.g. `1 => x` [read: crate sk-cbor-0.2.0 src/macros.rs:45-50].
- **Struct as array:** Manual only, with `cbor_array!` [read: crate sk-cbor-0.2.0 src/macros.rs:227-253].
- **Enum representations:** None (no derive).
- **`None` omitted on encode:** Yes, with `cbor_map_options!`: "The map entry is included iff the Value is not an Option or Option is Some" [read: crate sk-cbor-0.2.0 src/macros.rs:175-181]. Verified [ran: `none_fields_omitted ... ok`, which yields `a2 63 6f6666 01 64 64617461 40`].
- **Unknown keys ignored on decode:** Yes. `destructure_cbor_map!` only extracts the requested keys and skips the others during a linear scan [read: crate sk-cbor-0.2.0 src/macros.rs:91-109]. Verified [ran: `unknown_key_ignored_when_input_sorted ... ok`]. Two caveats:
  - The macro's keys **must be listed in sorted order**, with no runtime check (only `cfg(test)` asserts) [read: crate sk-cbor-0.2.0 src/macros.rs:29-35].
  - The input must already be canonical, because `read` rejects out-of-order maps first [ran: `declaration_order_input_rejected_as_out_of_order ... ok` → `Err(OutOfOrderKey)`].
- **Renames:** Not applicable. Keys are literal expressions.

## 4. Zero-copy decode

No. Byte and text strings are copied into `Vec<u8>` / `String` (`cbor_bytes_lit!` → `.to_vec()`, `cbor_text!` → `.into()`) [read: crate sk-cbor-0.2.0 src/reader.rs:151-172], [read: crate sk-cbor-0.2.0 src/macros.rs:354-358]. The extractors return owned values (`extract_byte_string() -> Option<Vec<u8>>`) [read: crate sk-cbor-0.2.0 src/values.rs:170-175].

## 5. Encode into `&mut [u8]` / decode from `&[u8]` without an allocator

- Decode takes `&[u8]` but builds an allocated `Value` tree [read: crate sk-cbor-0.2.0 src/reader.rs:43-57].
- Encode only targets `&mut Vec<u8>` [read: crate sk-cbor-0.2.0 src/writer.rs:30-52]. There is no slice writer.
- **No-allocator use is impossible** [ran: see §1].

## 6. RFC 8949 coverage

| Item | Encode | Decode | Evidence |
|---|---|---|---|
| Indefinite-length items | no | no (rejected, `UnknownAdditionalInfo`) | additional-info 31 is not in the accepted set [read: crate sk-cbor-0.2.0 src/reader.rs:110-118]; [ran: `indefinite_rejected ... ok` on `9f ff`] |
| Half-precision (f16) | no | no | no float variant in `ValueImpl` [read: crate sk-cbor-0.2.0 src/values.rs:28-45] |
| Preferred (shortest) integer/length heads | yes | **enforced** (`NonMinimalCborEncoding`) | writer `start_item` [read: crate sk-cbor-0.2.0 src/writer.rs:97-111]; reader check [read: crate sk-cbor-0.2.0 src/reader.rs:126-130]; [ran: `non_minimal_rejected ... ok` on `18 17`] |
| Preferred (shortest) floats | n/a (no floats) | rejects all floats (`UnsupportedFloatingPointValue`) | [read: crate sk-cbor-0.2.0 src/reader.rs:222-228]; [ran: `floats_rejected ... ok` on `f9 3e00`] |
| Tags | yes (any u64) | yes (any u64) | `Value::tag` [read: crate sk-cbor-0.2.0 src/values.rs:125-128]; [read: crate sk-cbor-0.2.0 src/reader.rs:208-215] |
| Simple values incl. undefined | false/true/null/undefined only | same; other simple values → `UnsupportedSimpleValue` | [read: crate sk-cbor-0.2.0 src/values.rs:49-54]; [read: crate sk-cbor-0.2.0 src/reader.rs:229-232]; [ran: `undefined_and_tag_roundtrip ... ok` (`f7` ok, `f8 20` rejected)] |
| Bignums (tags 2/3) | generic tag + bytes only | generic tag only | no bignum-specific code; `Unsigned(u64)` / `Negative(i64)` only [read: crate sk-cbor-0.2.0 src/values.rs:29-32] |
| Deterministic encoding (key order) | **8949 §4.2.1 bytewise** (always on) | **enforced** (strictly increasing keys) | details below |
| Duplicate-key detection | `Value::map` **panics** on duplicates | rejected (`OutOfOrderKey`) | details below |
| Nesting-depth limit / recursion safety | 127 (`TooMuchNesting`) | 127 (`TooMuchNesting`) | details below |

**Deterministic key order.** The `Ord` for `ValueImpl` sorts as follows [read: crate sk-cbor-0.2.0 src/values.rs:246-293]:

1. Major type first.
2. Unsigned keys by numeric value.
3. Negative keys by reversed value.
4. Byte and text strings by length, then by bytes.
5. Arrays and maps by element count, then element-wise.
6. Tags by tag number, then by value.

This matches bytewise comparison of the shortest-form encodings. `Value::map` sorts the entries [read: crate sk-cbor-0.2.0 src/values.rs:115-123]. I confirmed the order with the RFC 8949 §4.2.1 example keys `10, 100, -1, "z", "aa", [100], [-1], false`. The writer emits them in exactly that order, not in the §4.2.3 length-first order `10, -1, false, 100, "z", [-1], "aa", [100]` [ran: `writer_sorts_map_keys_in_rfc8949_bytewise_order ... ok`]. On decode, the map reader requires each key to be strictly greater than the previous one [read: crate sk-cbor-0.2.0 src/reader.rs:195-199].

Two notes on CTAP2 compatibility:

- For scalar keys (ints and strings) of the same major type, bytewise order and length-first order agree. CTAP2 maps use int or text keys, so the two rules coincide in practice.
- I did not check CTAP2's own rule text.

**Duplicate keys.** `Value::map` sorts, dedups, and `panic!()`s if anything was removed [read: crate sk-cbor-0.2.0 src/values.rs:115-123]. `EncoderError::DuplicateMapKey` is declared but never constructed [read: crate sk-cbor-0.2.0 src/writer.rs:22-25] (grep finds no other use). That contradicts the `cbor_map_options!` doc, which says "writing these values fails" [read: crate sk-cbor-0.2.0 src/macros.rs:180]. On decode, duplicates are rejected as out-of-order [ran: `duplicate_key_rejected ... ok` on `a2 01 00 01 00`].

**Nesting limit.** `read` and `write` both default to `Some(i8::MAX)` [read: crate sk-cbor-0.2.0 src/reader.rs:40-45], [read: crate sk-cbor-0.2.0 src/writer.rs:27-32]. The recursion is depth-checked [read: crate sk-cbor-0.2.0 src/reader.rs:74-76]. I confirmed the limit fires [ran: `nesting_limit ... ok` (200 nested arrays → `TooMuchNesting`)].

## 7. RFC claims in docs

README: "This crate implements Concise Binary Object Representation (CBOR) from [RFC 8949]" [read: crate sk-cbor-0.2.0 README.md:9-10]. Note that the crate supports only a deliberately restricted subset of RFC 8949: no floats, no indefinite lengths, and canonical-only input (§6 above).

## 8. Maintenance

- Last release: 0.2.0, 2026-10-22. The previous release was 0.1.2 on 2021-07-14, a 5-year gap [ran: crates.io API `/api/v1/crates/sk-cbor`].
- Last commit touching `libraries/cbor`: `c9dec1f` 2026-10-29, kaczmarczyck, "Unifies compiled syn versions onto 2.0.117 (#829)". Last commit touching `libraries/cbor/src`: `90cea20` 2026-10-29, kaczmarczyck [ran: `git log -1 -- libraries/cbor` / `-- libraries/cbor/src`].
- Path-scoped human authors in the last 12 months: 4 (kaczmarczyck 4, Frederic Ruget 1, Ivo Matijašević 1, VAMSI KRISHNA 1) [ran: `git shortlog -sne --since="12 months ago" HEAD -- libraries/cbor`].
- Recent substantive library work: "Reject CBOR string lengths that do not fit in usize (#788)" on 2026-10-06, and "Add sk-cbor boundary and ordering tests (#800)" on 2026-10-15 [ran: `git log -8 -- libraries/cbor`].
- The crate badge says `maintenance = { status = "passively-maintained" }` [read: crate sk-cbor-0.2.0 Cargo.toml.orig:12-13].
- Repo-wide (OpenSK): 12 open issues and 2 open PRs; `open_issues_count` = 14; not archived; pushed 2026-11-09 [ran: `gh api repos/google/OpenSK`, `gh issue list`, `gh pr list`]. A keyword filter `--search cbor` → 2 open issues, one of them relevant (§9).
- `unsafe`: 0 occurrences in `libraries/cbor/src` [ran: `grep -rn unsafe libraries/cbor/src | wc -l` → 0].

## 9. Embedded-relevant open issues

- https://github.com/google/OpenSK/issues/128 — "#cfg in macros": requests `#[cfg]` support inside `cbor_map!` (open since 2020-07-08).
- https://github.com/google/OpenSK/issues/420 (closed 2023-08-11) — "Ordering invariant does not hold between `cbor_map!` and `destructure_cbor_map!`". This is historical context for the sorted-keys precondition.
- The `--search cbor` keyword hit #348 is a Python `fido2` traceback from a dongle bring-up, not a library issue.
- I found no open issues about alloc, no_std, or code size.

## Probe

**Not eligible for the shortlist**, because it requires `alloc`. The probes live only in the scratchpad, not in `experiments/`.

No-alloc check (failing). It uses a `#![no_std] #![no_main]` bin with `#[panic_handler]` and `_start`, calling `sk_cbor::read(black_box(&[0x00]))`, with dependency `sk-cbor = { version = "=0.2.0" }`:

```text
$ cargo build --release --target thumbv7m-none-eabi
error: no global memory allocator found but one is required; link to std or add `#[global_allocator]` to a static item that implements the GlobalAlloc trait
```

Host behaviour tests: 11/11 pass [ran: `cargo test --lib`]. The text-keyed encode test expects the canonical order:

```text
a4 63 6c656e 18 7b | 63 6f6666 00 | 63 736861 58 20 aa…aa | 64 64617461 44 01020304
```

Mapping used (manual, alloc-based):

```rust
let value = cbor_map_options! {
    "off" => cbor_unsigned!(chunk.off),
    "data" => cbor_bytes!(chunk.data),
    "len" => chunk.len.map(|l| cbor_unsigned!(l)),
    "sha" => chunk.sha.map(|s| cbor_bytes!(s)),
};
sk_cbor::write(value, out)
// decode: keys must be listed in canonical order
destructure_cbor_map! { let { "len" => len, "off" => off, "sha" => sha, "data" => data, } = map; }
```
