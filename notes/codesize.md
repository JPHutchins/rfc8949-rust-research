# Flash cost per message type — thumbv7m, no allocator

## Summary

- **Serde itself isn't the waste; per-type code generation is.**
  - With enums and nested structs, minicbor-serde costs about the same per type as hand-written minicbor impls: **1,100 vs 1,163 B** at opt-level z, and 1,399 vs 1,295 B at s.
  - It also beats minicbor's own derive: 1,453 / 1,764 B.
  - On flat structs serde costs 17% more than hand-written at z and 45% more at s.
- **The serde backend's implementation dominates.** serde_cbor / serde_cbor_2 cost **4.8 KB per flat type** (6.2 KB at s): 7.5× minicbor-serde at z and 6.2× at s, because `Deserializer::parse_value` is monomorphized per visitor at 1.2–2.2 KB each.
- **Every CBOR crate generates code per type, and every per-type approach measured lands in the same band** (roughly 0.9–1.8 KB per enum/nested type). Only a table-driven design changes the slope: Munns measured −38% total `.text` (−59% without enums) for postcard-forth vs serde. No CBOR crate has tried it, so the CBOR gain is **unmeasured**.
- **Fixed cost** (fit intercept, opt-level z): cbor-smol about 3.3 KB (partial CBOR); minicbor hand-written about 6.7–7.2 KB; minicbor-derive about 7.7–8.5 KB; minicbor-serde about 9.4–9.5 KB; serde_cbor about 17.6 KB. In minicbor the fixed cost is mostly decoder primitives (`type_of`, `unsigned`, `str`, `bytes`, `map`: about 1.5 KB), `memcpy` (744 B) and UTF-8 validation (`from_utf8` 368 B plus a 256 B table).

## Why serde backends differ: two multiplying factors

Per-type flash ≈ (format-method instantiations per type, **set by serde_derive**) × (bytes per instantiation, **set by the format crate**). Munns, dtolnay and this measurement each address a different factor. They do not disagree.

| Factor | Who addresses it | Evidence |
|---|---|---|
| Instantiations per type (derive side) | **dtolnay**: serde_derive's per-struct `__Field` type "instantiat[es] the `Deserializer`'s methods with it". One non-generic `DeserializeSeed` "would reduce the number of instantiations of `Deserializer` methods by 50%". Ultimately he wants "type-erased bytecode", making impls "basically a single `static`". | `sources/munns/serde-pr2250-dtolnay-review-2023-08-06.txt`; serde_derive 1.0.229 still emits `__Field` (`src/de/struct_.rs`, `identifier.rs`) |
| Bytes per instantiation (format side) | **this repo**: serde_cbor's `Deserializer::parse_value` is generic over the visitor, giving 5 copies at 1,450–2,252 B in the flat N=4 binary. minicbor-serde's largest symbols are non-generic primitives compiled once (`Decoder::skip` 2,096, `type_of` 362, `unsigned` 316, `str` 240 B), with thin generic shims inlined into callers. | `nm -S --size-sort` on `cs-serde_cbor-n4` / `cs-minicbor-serde-n4` |
| Format-side erasure was tried | **dtolnay**, serde-rs/json#687 "Optimize compile time of Deserializer trait methods": "instantiate the control flow in these methods once per Read type rather than once per Visitor type" (−44% compile time). Closed unmerged. In the #2250 review he calls this "unsafe manual erasing" that formats should not need. | `gh pr view 687 -R serde-rs/json` |
| Remaining per-type floor | **Munns**: even with a lean format (postcard), serde's per-type codegen remains. Table-driven postcard-forth: −38% `.text`, −59% without enums. | `sources/munns/postcard-forth-rp2040-results-2024-06.md` |

Implication: an opt-in serde layer should keep its generic `Deserializer` methods as thin shims over non-generic primitives, as minicbor-serde does. That minimizes the format-side factor. The derive-side factor can only be removed by a native table-driven derive, or by serde itself changing.

## Results

Flash = `.text + .rodata + .data`. "Marginal" is the least-squares slope over N ∈ {1, 2, 4, 8, 16, 32} message types, and "fixed" is the intercept. N=0 is 36 B for every backend.

### Flat shape (u32/u64/i32/bool/Option<u32>/&str/&[u8] fields) — `results-flat.csv`

| backend | fixed (B), z | marginal B/type, z | marginal B/type, s | flash @ N=32, z |
|---|---:|---:|---:|---:|
| postcard (reference, not CBOR) | 1,798 | 258 | 713 | 9,800 |
| minicbor 2.3.1, hand-written text-key impls | 7,152 | 550 | 687 | 24,352 |
| minicbor-serde 0.7.2 | 9,424 | 642 | 993 | 29,555 |
| minicbor 2.3.1 derive, `#[cbor(map)]` int keys | 8,507 | 663 | 813 | 29,488 |
| cbor-smol 0.5.1 | 3,415 | 682 | 1,279 | 25,019 |
| serde_cbor 0.11.2 | 17,553 | 4,798 | 6,205 | 169,503 |
| serde_cbor_2 0.13.0 | 17,559 | 4,798 | 6,205 | 169,519 |

### Rich shape (flat kinds plus nested structs and enums) — `results-rich.csv`

Nested fields reference earlier messages, and borrowing propagates through them. Every enum is `{ A, B(u32), C { x: u32, y: bool } }`.
- The serde and hand-written backends encode it externally tagged with text names.
- minicbor-derive uses its own index-tagged array form.

| backend | fixed (B), z | marginal B/type, z | marginal B/type, s | flash @ N=32, z |
|---|---:|---:|---:|---:|
| postcard (reference) | 2,198 | 404 | 849 | 14,836 |
| cbor-smol | 3,285 | 897 | 1,510 | 31,582 |
| minicbor-serde | 9,513 | 1,100 | 1,399 | 43,387 |
| minicbor hand-written text keys | 6,666 | 1,163 | 1,295 | 42,588 |
| minicbor-derive int keys | 7,727 | 1,453 | 1,764 | 53,007 |

serde_cbor was not re-run on the rich shape; it was already ruled out on the flat shape.

Regenerate with `python3 gen.py --shape flat|rich` and `python3 fit.py results-<shape>.csv`.

## Method

- `gen.py` builds N deterministic message structs (`random.Random(i)`) with 3–6 fields and SMP-style text keys.
  - CBOR backends encode text-keyed maps, omit `None`, and borrow `&str` / `&[u8]` from the input.
  - `minicbor-derive-intkeys` uses integer keys, because that is all minicbor-derive supports.
- Each type is round-tripped (decode, then encode) on input and output slices obtained from `black_box`'d raw pointers.
- Build: rustc 1.95.0, `thumbv7m-none-eabi`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `loop {}` panic handler.
- `core::fmt` appears only in serde_cbor: 2.5 KB of fixed cost from integer `Display` in error paths.
- Reproducibility: the flat N=4 minicbor-hand build was rebuilt after the generator refactor and is byte-identical in size (9,883 B).

## Caveats

- The thumbv7m binaries are never executed. Behaviour was checked on the host instead (`experiments/codesize/crosscheck`, `cargo test`) for rich `Msg5` (an enum field plus two levels of nesting, all 3 variants):
  - **minicbor-serde and the hand-written impls give byte-identical output, and each decodes the other's bytes.** The size comparison is therefore like for like.
  - **cbor-smol round-trips its own bytes, but encodes enums by variant index** (`00`, `[1, 42]`, `[2, {…}]`) rather than text tags. Its enum wire format, and so its size, is not comparable. Its matrix gaps also apply: float encode panics, no tags, unbounded recursion.
  - The other 31 generated types are covered by the same generator code but were not individually exercised. The flat-shape probes in `experiments/probes/` verify exact bytes for a 4-field struct.
- `minicbor-hand-textkeys` approximates what a text-key derive would emit. It is not minicbor PR #57's actual output.
- One target and one toolchain, with round-trips only (no encode-only or decode-only measurement).
