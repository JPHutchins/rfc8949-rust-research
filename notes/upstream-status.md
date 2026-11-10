# Upstream status: are the "in-progress" efforts alive, and were candidates born no_std?

Checked 2026-11-10 via `gh` and the vendored git history.

## minicbor PRs covering the gap

| PR | What | Author | Opened | Last activity | Reviews | State |
|---|---|---|---|---|---|---|
| #53 | `#[s("…")]` string indices | twittner (maintainer) | 2026-02-15 | 2026-02-15 | 0 | draft, **merge conflicts**, 1 commit |
| #57 | `#[cbor(text_keys)]` on top of #53 | ropuls | 2026-03-14 | 2026-03-15 | 0 | 0 comments |
| #56 | `MaxCborLen` compile-time max encoded size | menasheofd | 2026-03-08 | 2026-04-28 (a user's "+1") | 0 | mergeable |
| #71 | `max_depth` for `Decode` derive | sdbondi | 2026-09-22 | 2026-10-08 | 5 (maintainer engaged) | under review |

- The maintainer is **active, not absent**. twittner authored 47 of the repo's commits in the last 12 months, and PRs were merged on 2026-10-11 (#75), 2026-09-26 (#72, #69) and 2026-07-13/22 (#62, #63, #64).
- The text-key and max-length PRs have therefore sat **8–9 months without review while other PRs merged**. That is a selective stall.
- What the maintainer said on issue #52 (2026-02-14/15):
  - First reply: "Would one of the serde-based CBOR crates work for you, e.g. minicbor-serde?"
  - Then: "Since it is not the first time this feature was requested I started a work-in-progress PR … There may still be show stoppers and the experiment may not work out."
- Demand is visible: on 2026-04-22 a user wrote they are "using str-index branch as the source for minicbor right now, hope to see it merged one day".

## Other efforts

- **Oxide Hubris `lib/microcbor`**: an in-tree derive over `minicbor::Encode`, adding text keys, `StaticCborLen` and alloc-free flatten.
  - **Encode-only**: its proc macros are `Encode` and `EncodeFields` (`lib/microcbor-derive/src/lib.rs:420,449`).
  - **Not on crates.io.**
- **npmccallum/ciborium fork**: last pushed 2026-04-13, no upstream PR, and the serde path still needs alloc (`notes/discovery-longtail.md`).

## Born no_std, or retrofitted?

Method: was `no_std` present in the first commit (`git rev-list --max-parents=0`), or when did it first appear (`git log -S no_std`)?

| crate | first commit | no_std in first commit | note |
|---|---|---|---|
| minicbor | 2020-03-02 | **yes** | alloc/std are opt-in features |
| cbor4ii | 2021-12-04 | yes | serde path needs alloc |
| tinycbor | 2025-12-21 | yes | yet 0.14–0.15.1 broke no-alloc builds |
| cbor-smol | 2021-01-31 | yes | "Re-extraction from `solo-bee`"; Trussed-specific subset |
| ciborium | 2020-11-17 | yes, **but alloc unconditional** | no-alloc would be a retrofit; #90 open since 2023 |
| serde_cbor | 2015-09-13 | **no**, added 2019-02-26 | retrofit; repo archived |

## Leads closed (2026-11-10)

| Lead | Finding | Evidence |
|---|---|---|
| facet-cbor (bearcove/vox) | std-only; deleted from vox on 2026-08-08 (`4b58849b`, "restore current Vox workspace from Facet"). No replacement CBOR crate found in facet-rs/facet or facet-rs/facet-format. facet-core has an unconditional `extern crate alloc;`, so any facet-based codec needs alloc by design. | `gh api repos/facet-rs/facet/contents/facet-core/src/lib.rs`; `git -C vendor/facet-cbor log --diff-filter=D` |
| maximebedard/facet-cbor | Depends on facet 0.28; last push 2025-08-01; 0 stars | `gh api repos/maximebedard/facet-cbor` |
| Thomasdezeeuw/cbor | Created and last pushed 2026-01-06 (43 commits in one push); 0 stars; unpublished (the crate name `cbor` belongs to BurntSushi's crate). no_std, **no alloc usage anywhere in `src/`**, opt-in `serde-v1` via `serde_core`, RFC 8949, about 1.9 kloc, **no derive**. | shallow clone; `grep -rn "alloc" src` (no hits) |
| const-cbor | Zero-dependency no-alloc core with `const fn` methods and no derive; its serde adapter needs alloc | `notes/discovery-longtail.md` row |
| tps_minicbor + tps_cddl (GlobalPlatform) | Schema-first CDDL code generation, not a type-first derive. Deterministic encoding needs `std`. | `notes/discovery-longtail.md` row |
| dgtlrift/cddlc | CDDL code generator targeting minicbor or ciborium; unpublished | `notes/discovery-longtail.md` row |

Not searched: Reddit, Rust Embedded Matrix archives. Codeberg search was rate-limited (HTTP 429), so only known Codeberg repos were checked.
