# CBOR (RFC 8949) for no_std Rust — needs assessment

Throwaway research repo. Question: is there a gap for a `no_std` (no-alloc), flash-efficient, type-safe, declarative RFC 8949 CBOR codec in Rust, and is there an existing effort worth contributing to?

Target spec is RFC 8949, which obsoletes RFC 7049 with an identical wire format; see `notes/rfc7049-vs-8949.md`.

## Verdict (2026-11-10)

**Yes, a real but narrow gap exists, and nobody has started the thing you described.**

- **No crate has a non-serde, no-alloc derive with text-string keys.** minicbor-derive is integer-keyed only. Oxide built an in-tree workaround (Hubris `microcbor`: encode-only, unpublished). minicbor's text-key PRs (#53, #57) and max-size PR (#56) have sat **8–9 months unreviewed** while the maintainer merged other work (`notes/upstream-status.md`).
- **Serde works today but has holes for embedded.** minicbor-serde (born no_std, maintained) handles text-keyed, zero-copy, no-alloc decode.
  - Its flash cost per message type (642 B) matches minicbor's own derive (663 B), so serde itself is not the problem (`notes/codesize.md`).
  - What serde can't give: a compile-time max encoded size, flatten without alloc, deterministic encoding, or a recursion bound.
- **Avoid:**
  - serde_cbor and serde_cbor_2: no_std was retrofitted onto serde_cbor, the repo is archived, and both cost **4.8 KB per type**.
  - ciborium: alloc is baked into its core.
  - cbor-smol: partial CBOR (float encode panics; no tags; no indefinite-length decode), and recursion is unbounded.
- **Missing from every no-alloc crate:** RFC 8949 §4.2.1 deterministic encoding, a compile-time max encoded length, and bounded decode recursion in derive.
  - Two of these are cheap for derived structs: keys are static, so key order can be fixed at compile time.
- **Unexplored for CBOR:** table-driven / type-erased derive, as in Munns' postcard-forth (−39% `.text` per type in his RP2040 measurement). Every CBOR crate uses per-type codegen. The measured floor for that approach is about 550 B per text-keyed type.

### Paths, best first

1. **A derive crate layered on minicbor's `Encode`/`Decode` traits**, published separately as Hubris did in-tree.
   - minicbor was `no_std` from its first commit, passes all 81 RFC Appendix A vectors, and is the most-used CBOR crate in embedded Rust.
   - Needs no upstream permission.
   - Can ship text keys, `MaxCborLen`, alloc-free flatten, depth limits and compile-time §4.2.1 key order.
   - Before writing code, ask on minicbor #52/#53 what the maintainer's "show stoppers" are, and ask Oxide whether `microcbor` is meant to be extracted.
2. **Then, optionally, a table-driven backend** for that derive (const field tables plus one shared engine on top of minicbor's `Encoder`/`Decoder`), to test whether Munns' result carries over to CBOR.

## Index

| Path | Contents |
|---|---|
| `notes/rfc7049-vs-8949.md` | What changed between the RFCs; key-ordering divergence experiment |
| `notes/crate-matrix.md`, `notes/crates/` | Evidence-tagged feature matrix for 10 crates |
| `notes/discovery-longtail.md` | Long-tail crates, who uses what in embedded, demand signals |
| `notes/upstream-status.md` | PR ages, maintainer activity, born-no_std vs retrofit |
| `notes/codesize.md` | thumbv7m flash per message type, 7 backends |
| `notes/munns-serde-codesize.md` | James Munns' serde bloat analysis (SDR "A different serde") |
| `sources/` | RFC texts, test vectors (submodule), transcripts |
| `vendor/` | Upstream crate checkouts (submodules) |
| `experiments/` | Code-size harness, probes, Appendix A conformance, key-ordering script |
