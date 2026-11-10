# CBOR (RFC 8949) for no_std Rust — needs assessment

Throwaway research repo. Question: is there a gap for a `no_std` (no-alloc), flash-efficient, type-safe, declarative RFC 8949 CBOR codec in Rust, and is there an existing effort worth contributing to?

Target spec is RFC 8949, which obsoletes RFC 7049 with an identical wire format; see `notes/rfc7049-vs-8949.md`.

| Path | Contents |
|---|---|
| `notes/` | Findings, one file per topic |
| `notes/crates/` | Per-crate evidence |
| `sources/` | Primary-source excerpts and transcripts |
| `vendor/` | Upstream crate checkouts (submodules) |
| `experiments/` | Code-size and conformance experiments |
