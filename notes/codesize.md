# Flash cost per message type — thumbv7m, no allocator

## Summary

- Among no-alloc CBOR codecs that handle text-keyed structs, **marginal flash per message type clusters at 550–680 B** (opt-level z). Serde is **not** the dominant cost here: minicbor-serde (642 B) is on par with minicbor's own index-keyed derive (663 B).
- **serde_cbor / serde_cbor_2 are pathological: 4.8 KB per type.** `Deserializer::parse_value` is monomorphized once per visitor at 1.2–2.2 KB each. This is the bloat mechanism Munns describes (`notes/munns-serde-codesize.md`), and it shows up here directly.
- **Fixed cost** (N=1) ranges from 3.8 KB (cbor-smol, partial CBOR) through 6.1 KB (minicbor) to 9.0 KB (minicbor-serde).
- Hand-written minicbor impls with text keys (what a text-key derive would roughly expand to) are the CBOR floor for **per-type codegen**: 550 B/type. Postcard, which is positional with no keys, is 258 B/type.
- No CBOR crate tries table-driven / type-erased derive (Munns' postcard-forth approach, −39% per type in his measurement). That design space is unexplored for CBOR.

## Results (opt-level z unless noted)

Flash = `.text + .rodata + .data`. Slope is a least-squares fit over N ∈ {1, 2, 4, 8, 16, 32}. N=0 is 36 B for every backend.

| backend | flash @ N=1 (B) | marginal B / message type | flash @ N=32 (B) |
|---|---:|---:|---:|
| postcard (reference, not CBOR) | 1,508 | 258 | 9,800 |
| minicbor 2.3.1, hand-written text-key impls | 6,131 | 550 | 24,352 |
| minicbor-serde 0.7.2 | 8,965 | 642 | 29,555 |
| minicbor 2.3.1 derive, `#[cbor(map)]` int keys | 8,588 | 663 | 29,488 |
| cbor-smol 0.5.1 | 3,773 | 682 | 25,019 |
| serde_cbor 0.11.2 | 19,206 | 4,798 | 169,503 |
| serde_cbor_2 0.13.0 | 19,210 | 4,798 | 169,519 |

At opt-level s every slope is steeper: minicbor hand 687, minicbor-derive 813, minicbor-serde 993, cbor-smol 1,279, serde_cbor 6,205. Postcard rises to 713. Full data: `experiments/codesize/results.csv`; regenerate with `python3 gen.py && python3 fit.py results.csv`.

| backend (flash, B) | N=1 | N=2 | N=4 | N=8 | N=16 | N=32 |
|---|---:|---:|---:|---:|---:|---:|
| postcard | 1,508 | 1,882 | 3,380 | 4,320 | 6,138 | 9,800 |
| serde_cbor | 19,206 | 25,778 | 38,759 | 58,035 | 96,323 | 169,503 |
| cbor-smol | 3,773 | 4,661 | 6,260 | 9,062 | 14,693 | 25,019 |
| minicbor-serde | 8,965 | 10,497 | 12,578 | 15,292 | 20,103 | 29,555 |
| minicbor-derive-intkeys | 8,588 | 9,842 | 11,390 | 14,072 | 19,428 | 29,488 |
| minicbor-hand-textkeys | 6,131 | 8,723 | 9,883 | 12,069 | 16,414 | 24,352 |

## Method

- `experiments/codesize/gen.py` generates N deterministic message structs (`random.Random(i)`): 3–6 fields from {u32, u64, i32, bool, Option<u32>, &str, &[u8]}, with keys drawn from SMP-style names.
  - CBOR backends encode text-keyed maps, omit `None`, and borrow `&str`/`&[u8]` from the input.
  - `minicbor-derive-intkeys` is the exception: it uses integer keys, because that is all minicbor-derive supports.
- Each type is round-tripped (decode, then encode) on input and output slices obtained from `black_box`'d raw pointers, so nothing constant-folds and no static buffer lands in `.rodata`.
- Build: rustc 1.95.0, `thumbv7m-none-eabi`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, and a `loop {}` panic handler.
- `core::fmt` appears only in serde_cbor (2.5 KB, fixed cost from integer `Display` in error paths).

## Caveats

- The binaries are never executed. Wire correctness of the generated round-trips is not checked here; the per-crate probes in `experiments/probes/` verify exact bytes for a 4-field text-keyed struct.
- Only the round-trip is measured; encode-only or decode-only firmware would see smaller numbers.
- `minicbor-hand-textkeys` approximates a text-key derive. It is not the code from minicbor PR #57, which is the obvious next backend to add.
- One target, one toolchain, and synthetic flat messages: no enums and no nesting. Munns found enums are the expensive case.
