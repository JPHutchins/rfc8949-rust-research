# CBOR (RFC 8949) for no_std Rust — needs assessment

Throwaway research repo. Question: is there a gap for a `no_std` (no-alloc), flash-efficient, type-safe, declarative RFC 8949 CBOR codec in Rust, and is there an existing effort worth contributing to?

Target spec is RFC 8949, which obsoletes RFC 7049 with an identical wire format; see `notes/rfc7049-vs-8949.md`.

## Verdict (2026-11-10)

**Yes, there is a gap, and no existing project matches the target.** The target is a born-no_std codec with alloc and no-alloc modes, no recursion, opt-in serde, and a table-driven native derive.

- **Closest efforts, and why none fits** (`notes/upstream-status.md`, `notes/discovery-longtail.md`):
  - **minicbor** is born no_std and passes 81/81 RFC 8949 Appendix A vectors, but its derive is integer-keyed only, generates code per type, and its derived decoders recurse without a depth bound. Its text-key PRs (#53, #57) and max-size PR (#56) have gone **8–9 months unreviewed** while other PRs were merged.
  - **Hubris `microcbor`**: derives encode only, is in-tree, and is unpublished.
  - **Thomasdezeeuw/cbor**: no_std, no alloc and opt-in serde, but it has no derive, was dumped in a single push on 2026-01-06, and is unpublished.
  - **facet-cbor**: needs std, and facet-core has an unconditional `extern crate alloc`.
  - **const-cbor**: no derive.
  - **tps_cddl / cddlc**: CDDL schema code generators, not derives.
- **Serde costs no more flash than other per-type code** (`notes/codesize.md`).
  - Measured: a well-built serde backend (minicbor-serde) costs about the same flash per type as hand-written or derived per-type code. With enums and nesting it is 1,100 vs 1,163 B/type at opt-level z, and +8% at s.
  - The comparison is like for like. A host test shows byte-identical wire output and cross-decoding between minicbor-serde and the hand-written impls for a message with an enum (all 3 variants) and two levels of nesting (`experiments/codesize/crosscheck`).
  - A badly built one (serde_cbor) costs 6–7.5× as much (about 9× hand-written).
  - Opt-in serde can be depth-*limited* but never non-recursive: serde's visitors nest on the call stack by design.
  - From the sources: serde_derive 1.0.229 still emits a per-struct `__Field` type and visitor. dtolnay's type-erased fix would need a change inside serde, and it is unshipped. Munns' table-driven gains are therefore out of reach through serde.
- **The flash win comes only from table-driven derive.** Every CBOR crate generates code per type. Munns measured −38% total `.text` (−59% without enums) for postcard-forth vs serde on RP2040. For CBOR the gain is **unmeasured**, and that is the first thing to prototype.
- **Missing from every no-alloc crate:**
  - RFC 8949 §4.2.1 deterministic encoding (free for derived structs: sort keys at compile time);
  - compile-time max encoded length;
  - non-recursive decode. Some crates bound the recursion (serde_cbor at 128, cbor4ii at 256); cbor-smol, and minicbor's derive/serde paths, are unbounded.
- **Avoid as foundations:**
  - serde_cbor / serde_cbor_2: no_std was retrofitted in 2019, the original is archived, and they cost 4.8 KB per type.
  - ciborium: alloc is baked into its core.
  - cbor-smol: partial CBOR, unbounded recursion.

### Shape of the crate (`notes/design-sketch.md`)

A **derive crate** plus a **small no_std runtime**: a proc-macro crate can export only macros.
- **The derive** emits const descriptors:
  - field offsets;
  - pre-encoded, pre-sorted keys;
  - nested-descriptor links;
  - `MAX_ENCODED_LEN` and `MAX_DEPTH`.
- **The runtime** holds one non-generic, iterative engine plus `alloc` and `serde` features.
- **Trade-off against "fully type safe":** a type-erased engine walking offsets and function pointers needs `unsafe`. Munns called postcard-forth "wildly unsafe". The aim is `unsafe` confined to the engine and derive output, behind a safe public API whose types are checked at compile time.
- **The alloc/no-alloc API** follows existing precedents: minicbor's `Write` impls, postcard's flavors and `MaxSize`, smoltcp's `managed::ManagedSlice`, and heapless 0.9's storage-generic `VecInner`.
- **The wire layer** can start on minicbor's primitives. Two catches: minicbor writes floats at their declared width, so §4.2.1 output for float fields needs its own float path; and it accepts and emits the forbidden simple values `f8 14`…`f8 1f`.

### People who have hit the same wall

These are potential collaborators, not projects to contribute to:
- Oxide, Hubris `microcbor`;
- the authors of minicbor #57 (text keys), #56 (max length) and #71 (depth limit);
- James Munns (postcard-forth).

## Index

| Path | Contents |
|---|---|
| `notes/rfc7049-vs-8949.md` | What changed between the RFCs; key-ordering divergence experiment |
| `notes/crate-matrix.md`, `notes/crates/` | Evidence-tagged feature matrix for 10 crates |
| `notes/discovery-longtail.md` | Long-tail crates, who uses what in embedded, demand signals |
| `notes/upstream-status.md` | PR ages, maintainer activity, born-no_std vs retrofit, closed leads |
| `notes/codesize.md` | thumbv7m flash per message type, flat and enum/nested shapes |
| `notes/design-sketch.md` | Crate shape, alloc/no-alloc precedents, no-recursion plan, opt-in serde |
| `notes/rust-alloc-noalloc-state.md` | Rust-wide alloc/no-alloc state: `Allocator` stable in 1.100, Store API, heapless, polyfills |
| `notes/munns-serde-codesize.md` | James Munns' serde bloat analysis (SDR "A different serde") |
| `sources/` | RFC texts (IETF Trust), RFC test vectors (submodule). Third-party talks, posts and transcripts are linked from `notes/`, not republished |
| `vendor/` | Upstream crate checkouts (submodules) |
| `experiments/` | Code-size harness, probes, Appendix A conformance, key-ordering script |
