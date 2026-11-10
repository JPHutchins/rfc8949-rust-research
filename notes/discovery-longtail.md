# Discovery: the long tail of Rust CBOR codecs, embedded usage, and demand signals

Survey date: 2026-11-10. Scope: crates.io long tail, GitHub, Codeberg/GitLab, the CBOR crates that embedded Rust projects actually use, and public demand signals for a `no_std` (ideally no-alloc), flash-efficient, derive-based CBOR codec.

Citation tags:
- `[crate X-v path]`: read from the published `.crate` tarball (static.crates.io).
- `[gh O/R path]`: read from the repo's default branch via the GitHub API.
- `[api]`: GitHub/crates.io/Codeberg API metadata.

"Last commit" is the newest commit on the default branch (`repos/O/R/commits`), not crates.io `updated_at`. Nothing was built or run. Flash sizes are only the numbers a project publishes itself.

## Executive summary

- **What embedded projects use (fact).** The most common CBOR crate in the projects checked is **minicbor**: Ariel OS, Hubris, cfg-noodle, faderpunk, SpoolEase (esp-hal), a device-side Rust SMP server, BitBox02, Passport, Keystone, rustot, and miniconf. The FIDO firmwares from Trussed, Nitrokey and SoloKeys use **cbor-smol** (serde). The two firmware projects found using ciborium both have a heap allocator in their lockfile (`esp-alloc`, `embedded-alloc`). Otherwise ciborium shows up in host-side MCUmgr tools.
- **Strongest unmet-need signal (fact).** Firmware teams built their own codec, derive layer, or fork instead of using a crate as-is:
  - Oxide **Hubris `microcbor`** (2025-10), a derive layer on top of `minicbor::Encode`. It gives three reasons: serde cannot give a compile-time max length, `serde(flatten)` needs alloc, and minicbor-derive is index-keyed.
  - **Caliptra** `common/eat/src/cbor.rs` (2026-01), a fixed-buffer `no_std` encoder.
  - **lakers**' in-tree decoder, which had two bug reports in 2026-10.
  - Three FIDO-key firmwares (pillkey, fapico2, Bruce), each with an in-tree codec.
  - **Sonair**, which maintains a trimmed "certified subset" fork of minicbor.
- **Two of the three Hubris features have open upstream PRs.** The PRs exist (fact); that they match Hubris's needs is judgment. Neither is by Oxide.
  - minicbor PR #56 (2026-03, contributor) adds a `MaxCborLen` derive for compile-time buffer size.
  - PR #57 (2026-03, contributor) adds `#[cbor(text_keys)]`. It is stacked on the maintainer's own draft #53, "Add support for string indices" (2026-02), whose body reads "Work in progress and experimental".
  - The third feature, flatten without alloc (`EncodeFields`), has no upstream PR. The only related work found is the stale fork branch sterraf/minicbor@EnumFlatDerive (2025-02, "WIP: Flat attribute").
  - All three PRs are unmerged. ciborium's no-alloc request (#90, 2023) is still open, and ciborium has not released since 0.2.2 (2024-01-24).
- **In-progress or unpublished efforts (fact):**
  - npmccallum/ciborium. ciborium's original author added `ciborium-atom` and a new serde `cbor` crate in 2026-04. The serde path still needs alloc, and there is no upstream PR.
  - Thomasdezeeuw/cbor: unpublished; `no_std` core plus serde (whether it needs alloc: Unknown).
  - const-cbor: published 2026-08, no-alloc. Among the READMEs read for this survey, it is the only one that publishes Cortex-M flash numbers (3.1 KB core). It has no derive, and its serde adapter needs alloc.
  - GlobalPlatform `tps_minicbor` + `tps_cddl`: CDDL-to-`no_std` code generation "optimized for code size".
  - dgtlrift/cddlc: CDDL code generation that emits Rust for minicbor/ciborium.
- **Judgment.** The gap is real but narrow: a declarative, no-alloc codec with compile-time max length and text-or-integer keys. The lowest-cost route looks like helping land #53/#57 and #56-style derive features in **minicbor**, not a new codec. **tinycbor** is the smaller alternative, but its current releases do not build without alloc (see `notes/crates/tinycbor.md`).

## 1. Crate and repo table

The `no_std` column uses three values:
- **no-alloc**: declares `#![no_std]`, and `alloc` is feature-gated or absent. Not build-tested here; see `notes/crates/*.md` for build probes, which override this table.
- **alloc**: `no_std` but `extern crate alloc` is unconditional or required.
- **std**: no `no_std` attribute.

### 1a. General-purpose codecs that claim `no_std`

| name | URL | crates.io | last commit | archived? | no_std | API model | note | evidence |
|---|---|---|---|---|---|---|---|---|
| minicbor (+derive, -serde, -io) | https://github.com/twittner/minicbor | 2.3.1 / derive 0.19.5 / serde 0.7.2 / io 0.23.1 | 2026-10-18 | no | no-alloc (core, derive, serde); minicbor-io needs std | own-trait derive (`#[n(i)]` index keys) + serde adapter | 520 commits, 20 contributors, 207 reverse deps. `minicbor_serde::to_slice` merged 2026-09 (#72). BlueOak-1.0.0. Old tracker on GitLab. | `[crate minicbor-2.3.1 src/lib.rs:140]` alloc under `cfg(feature="alloc")`; derive without alloc: build probe in `notes/crates/minicbor.md`; `[crate minicbor-serde-0.7.2 src/lib.rs:43]`; `[crate minicbor-io-0.23.1 Cargo.toml]` minicbor `features=["std"]`; `[api]` |
| minicbor-adapters | https://codeberg.org/chrysn/minicbor-adapters | 0.0.8 | 2025-10-09 | no | no-alloc | adapters (heapless, cboritem, embedded-io) for minicbor | Home for the heapless impls the minicbor maintainer declined (#67). | `[crate minicbor-adapters-0.0.8 src/lib.rs]` `#![no_std]`; categories `no-std::no-alloc`; Codeberg API |
| minicbor-embedded-io | https://github.com/rmja/minicbor-embedded-io | 0.6.0 | 2026-07-23 | no | no-alloc | async reader/writer over `embedded-io-async` | 64 commits | `[crate minicbor-embedded-io-0.6.0 src/lib.rs:10]` alloc gated |
| cbor-smol (+ serde-indexed) | https://github.com/trussed-dev/cbor-smol | 0.5.1 (serde-indexed 0.2.0) | 2025-10-01 | no | no-alloc | serde; serde-indexed derives integer-keyed maps | Trussed/Nitrokey/SoloKeys. Supports heapless 0.7–0.9. 10 reverse deps. Deliberately partial: float encode panics (`todo!`), no tags; no thumbv6m build because of `delog` (per `notes/crates/cbor-smol.md`). | `[crate cbor-smol-0.5.1 src/lib.rs]` `cfg_attr(not(test), no_std)`, no `extern crate alloc`; depends on `serde_core` with default features off |
| tinycbor (+derive) | https://github.com/carloskiki/tinycbor | 0.15.1 / derive 0.15.4 | 2026-07-29 | no | declares alloc gated (default `std`), **but 0.14.0–0.15.1 fail to build without alloc**; 0.13.0 builds | own-trait derive (`Encode`/`Decode`/`CborLen`) over `embedded-io` | "inspired by minicbor". 192 commits, 2 contributors, 6 open issues, 1 reverse dep. Upstream `no-std` CI job failing since 2026-07-28 (per sibling note). | `[crate tinycbor-0.15.1 src/lib.rs:1-53]`; build probe in `notes/crates/tinycbor.md` |
| const-cbor (+ const-cbor-serde) | https://github.com/larrydewey/const-cbor | 0.3.0 / serde 0.1.0 | 2026-10-18 | no | core **no-alloc, zero deps**; serde adapter is **alloc** | low-level `const fn` methods, "no traits"; serde adapter | README publishes `thumbv7m` flash: core **3,148 B**, everything **24,102 B**. 7 commits, 1 contributor, created 2026-08-24. README CI badge points to `larry-shobaki/const-cbor` (404 via API). Repo ships `AGENTS.md`. | `[crate const-cbor-0.3.0 README.md, src/lib.rs]` `#![no_std]`; `[crate const-cbor-serde-0.1.0 src/lib.rs]` "`no_std` + `alloc`"; `[api]` |
| tps_minicbor (+ tps_cddl, tps_cose) | https://github.com/GlobalPlatform/TPS-API-Reference-Implementations | 0.10.0 | 2026-11-05 | no | no-alloc; deterministic encoding needs `std` | low-level `Encoder<N>` builder; **CDDL code generation** via `tps_cddl gen-encoder/gen-decoder` | `tps_cddl`: "generate a `tps_minicbor`-based CBOR decoder/encoder … optimized for code size". `tps_cose`: "usable without an alloc". Dormant 2023-03 to 2026-10. One main author. 4 reverse deps. | `[crate tps_minicbor-0.10.0 README.md, src/lib.rs:197]` std only under cfg; `[gh …/tps_cddl/README.md]`; `[api]` commits |
| zerocbor (+derive) | https://github.com/nuskey8/zerocbor | 0.1.0 | 2026-10-01 | no | **alloc** (unconditional `extern crate alloc`); default `std` | own-trait derive (`FromCbor`/`ToCbor`) | 5 commits in one day. Port of the `zerompk` architecture. 0 reverse deps. | `[crate zerocbor-0.1.0 src/lib.rs:27]` |
| strede / strede-cbor / strede-derive | https://github.com/Daniel-Aaron-Bloom/strede | 0.2.3 | 2026-08-29 | no | no-alloc (alloc gated) | serde-like derive, **deserialize-only**, async pull-based | "Async, zero-alloc, pull-based deserialization". No serializer. 53 commits, 1 contributor. | `[crate strede-0.2.3 README.md, src/lib.rs:52]` |
| sacp-cbor (+derive) | https://github.com/Synext-Solution/sacp-cbor | 0.23.0 | Unknown (repo 404 via API) | Unknown | no-alloc for the fixed-storage encoder; **derive requires alloc** | own derive + serde adapter, deterministic profile | Niche deterministic profile (SACP-CBOR/1). 2 reverse deps. | `[crate sacp-cbor-0.23.0 Cargo.toml]` `derive = ["alloc", …]`; README "fixed-storage encoder work without std or alloc" |
| hardy-cbor | https://github.com/ricktaylor/hardy | 2.0.0 | 2026-11-10 | no | **alloc** (unconditional, though category says `no-std::no-alloc`) | own traits `ToCbor`/`FromCbor`, manual impls + callback parser | Serialization layer of the DTN BPv7 stack. Repo has 2,998 commits and 15 contributors. | `[crate hardy-cbor-2.0.0 src/lib.rs:26-27, README]` |
| cbor4ii | https://github.com/quininer/cbor4ii | 1.2.4 | 2026-10-19 | no | no-alloc core; serde path is alloc | own traits, no derive (`core`); serde (`serde1` implies `serde/alloc`) | 57 reverse deps | `[crate cbor4ii-1.2.4 Cargo.toml, src/lib.rs:4]`; `notes/crates/cbor4ii.md` |
| ciborium / -ll / -io | https://github.com/enarx/ciborium | 0.2.2 (2024-01-24) | 2026-06-19 (dependabot) | no | ciborium **alloc**; -ll and -io no-alloc | serde (ciborium); low-level (-ll) | No release in 2.8 years. Unreleased on `main`: `embedded-io` feature in ciborium-io, `canonical_*` functions, optional `half`. 57 open issues. | `[crate ciborium-0.2.2 Cargo.toml]` forces `ciborium-io/alloc`; `[gh enarx/ciborium ciborium-io/Cargo.toml]`; `[api]` commits since 2024-01-25 |
| serde_cbor | https://github.com/pyfisch/cbor | 0.11.2 | 2021-08-15 | **yes** (2022-05-22) | no-alloc serialize-to-slice documented | serde | Unmaintained ("Looking for Maintainers" #179) | `[crate serde_cbor-0.11.2 src/lib.rs:215]` "without `alloc`" |
| serde_cbor_2 | https://github.com/kanidm/cbor | 0.13.0 | 2026-09-14 (dependabot only) | no | as serde_cbor | serde | Open PR #23 moves `Value` to `alloc` | `[crate serde_cbor_2-0.13.0 src/lib.rs]`; `[api]` |
| cbor2 (+derive) | https://github.com/ldclabs/cbor2 | 1.1.6 | 2026-09-24 | no | no-alloc for **serialize** (`to_slice`, derive); **deserialize needs alloc** | serde + `#[derive(Cbor)]` (integer keys, tags) | 33 reverse deps | `[crate cbor2-1.1.6 src/lib.rs:393-398]` |
| sk-cbor | https://github.com/google/OpenSK (`libraries/cbor`) | 0.2.0 | 2026-11-09 | no | **alloc** | `Value` tree + `cbor_map!` macros, `read`/`write` | OpenSK's in-tree codec, published | `[crate sk-cbor-0.2.0 src/lib.rs]` |
| cbor_event | https://github.com/primetype/cbor_event | 3.3.0 | 2026-08-03 | no | **alloc** | low-level event/token codec, own `Serialize`/`Deserialize` traits, no derive | — | `[crate cbor_event-3.3.0 src/lib.rs:107]`; `notes/crates/cbor_event.md` |
| deser-cbor | https://github.com/mitsuhiko/deser | 0.10.2 | 2026-10-26 | no | **alloc** | serde-alternative derive | "this crate only needs `alloc`" | `[crate deser-cbor-0.10.2 src/lib.rs:163]` |
| nextjson | https://github.com/blueokanna/NextJson | 0.1.5 | 2026-11-08 | no | **alloc** ("no `std` feature") | own derive, JSON+CBOR | 64 commits since 2026-08 | `[crate nextjson-0.1.5 src/lib.rs]` |
| neco-cbor | https://github.com/barineco/neco-crates | 0.1.1 | 2026-10-21 | no | **alloc** | Unknown | crate description: "CBOR / DAG-CBOR codec for no_std environments" | `[crate neco-cbor-0.1.1 src/lib.rs:3-4]` |
| fcpw | https://github.com/Elizafox/fcpw | 0.1.3 | 2026-09-04 | no | no_std, alloc gated; default `std,alloc,serde` | serde | "Fastest CBOR Parser in the West" | `[crate fcpw-0.1.3 Cargo.toml, src/lib.rs:50]` |
| babbel_cbor | https://github.com/clockworkengineer/babbel | 0.3.0 | 2026-10-30 | no | no_std, alloc gated; default `std` | Unknown | 1 contributor | `[crate babbel_cbor-0.3.0 src/lib.rs:35]` |
| cbor-rs (kushaldas) | https://github.com/kushaldas/cbor-rs | 0.1.0 | 2026-07-23 | no | **alloc** | low-level + optional serde | 7 commits | `[crate cbor-rs-0.1.0 src/lib.rs:48]` |
| suit_cbor / cose_minicbor | https://github.com/leduclean/suit-rs | 0.1.1 | 2026-02-25 | no | no-alloc (`#![no_std]`, on minicbor) | lazy decoding helpers on minicbor | SUIT/COSE for MCUs. Fork `carl-tud/minicbor-weird` adds "weird cbor-encoded bstr" handling. | `[crate suit_cbor-0.1.1 src/lib.rs]`, `[crate cose_minicbor-0.1.1 src/lib.rs]` |
| senml-cbor | https://github.com/MarkAtwood/rust-senml-cbor | 0.1.0 | 2026-10-23 | no | no-alloc | SenML-specific encode/parse | 2 commits | `[crate senml-cbor-0.1.0 src/lib.rs, README]` |
| cboritem | https://codeberg.org/chrysn/cboritem | 0.1.3 | 2025-04-04 | no | no-alloc (std/alloc only in tests) | types for already-serialized items | used by Ariel OS | `[crate cboritem-0.1.3 src/lib.rs:74,246]` |
| coset | https://github.com/google/coset | 0.4.2 | Unknown | Unknown | **alloc** | COSE types on ciborium | — | `[crate coset-0.4.2 src/lib.rs:101]` |
| dcbor | https://github.com/BlockchainCommons/bc-dcbor-rust | 0.25.2 | Unknown | Unknown | **alloc** | Unknown | crate description: "Deterministic CBOR (dCBOR)" | `[crate dcbor-0.25.2 src/cbor.rs:4]` |
| serde_cbor_core / serde_ipld_dagcbor | gordonbrander/serde_cbor_core, ipld/serde_ipld_dagcbor | 0.1.0 / 0.7.0 | Unknown | Unknown | **alloc** (cbor4ii `use_alloc`) | serde | — | `[crate … Cargo.toml]` |
| cbor-no-std | (no repo) | 0.3.0 (2018) | n/a | n/a | **alloc** | deserializer only | historical | `[crate cbor-no-std-0.3.0 src/lib.rs]` |

### 1b. Unpublished projects, forks, and in-tree codecs

| name | URL | crates.io | last commit | archived? | no_std | API model | note | evidence |
|---|---|---|---|---|---|---|---|---|
| **npmccallum/ciborium** (fork `main`) | https://github.com/npmccallum/ciborium | no (adds `ciborium-atom` 0.1.0, `cbor` 0.1.0) | 2026-04-13 | no | `ciborium-atom`: no_std, `alloc` is a default feature. `cbor` (serde): **alloc** (unconditional `extern crate alloc`, `serde/alloc`), but encodes into `&mut [u8]` | serde + low-level `Atom` wire types | 12 commits ahead of enarx `main` (2026-04-02 to 04-13), by ciborium's original author. Path dependency `../../floats` is not in the repo. No upstream PR from this branch found. | `[api compare enarx/ciborium main...npmccallum:main]`; `[gh npmccallum/ciborium cbor/Cargo.toml, cbor/src/lib.rs, cbor/tests/no_std.rs, ciborium-atom/Cargo.toml]` |
| **Thomasdezeeuw/cbor** | https://github.com/Thomasdezeeuw/cbor | no (Cargo name `cbor` is taken on crates.io) | 2026-01-06 | no | no_std; alloc use Unknown (`ser.rs` has `Vec`/`std::io::Error` signatures whose cfg gating was not checked); `std` is a default feature | low-level `Decoder` iterator + `Encode` trait + serde `Serializer`/`Deserializer` (`serde-v1`) | 43 commits 2025-03 to 2026-01, 1 contributor, no README. Output is a user-implemented `Buf` trait; only a `std::io::Write` impl ships. | `[gh Thomasdezeeuw/cbor Cargo.toml, src/lib.rs:28-35,109, src/encoder.rs:20-35]` |
| 0xicbor | https://github.com/keirsalterego/0xicbor | no | 2026-08-02 | no | `#![no_std]` with `extern crate alloc` (whether used on the hot path: Unknown) | C-ABI drop-in port of intel/tinycbor | 107 commits in 3 days, 1 contributor. Claims the upstream Qt suite passes 4,929/4,929. | `[gh … README.md, crates/cbor-core/src/lib.rs:12-15]` |
| dgtlrift/cddlc | https://github.com/dgtlrift/cddlc | no | 2026-11-04 | no | generated Rust targets minicbor/ciborium "with `#![no_std]` support" | **CDDL code generator** (Rust, C, C++, C#, TS, Python, Dart) | 278 commits since 2026-03, 1 contributor | `[gh dgtlrift/cddlc README.md]`; crates.io API: not published |
| carl-wallace/cbor_formats (`cbor_derive`) | https://github.com/carl-wallace/cbor_formats | no | 2026-11-01 | no | **alloc** (serde `alloc`, ciborium) | proc-macro mapping structs to ciborium `Value` maps (integer keys) | "exists in lieu of a CDDL compiler for Rust"; RATS specs (CoRIM, EAT, CoSWID) | `[gh … README.md, cbor_derive/Cargo.toml]` |
| kaspar030/nanocbor-rs | https://github.com/kaspar030/nanocbor-rs | no | 2022-02-16 | no | Unknown | decoder (repo description: "A small CBOR decoder for embedded") | 1 commit; author Kaspar Schleiser per `Cargo.toml` `authors` | `[gh … Cargo.toml]` |
| nickray/ctapcbor | https://github.com/nickray/ctapcbor | no | 2020-03-02 | no | serde_cbor without default features + heapless | serde | "does not explode in your face like serde-cbor" | `[gh … Cargo.toml]`, repo description |
| **Sonair-AS/minicbor** (fork) | https://github.com/Sonair-AS/minicbor | no | 2026-10-14 | no | as minicbor | as minicbor | Branch `certified_subset` is 9 ahead of upstream: "Create certified subset of minicbor", "Add required documentation for 3rd party crate", "Get required coverage" (+5,618-line coverage test). Removes `bytes.rs`, tokenizer and the `cbor-display` bin. | `[api compare twittner/minicbor develop...Sonair-AS:certified_subset]` |
| hushmesh/serde_cbor (fork) | https://github.com/hushmesh/serde_cbor | no | 2026-07-17 | no | as serde_cbor | serde | 3 ahead: packed enum encoding; "Allow expressive errors with alloc" | `[api compare]` |
| islet-project/ciborium (fork) | https://github.com/islet-project/ciborium | no | 2026-11-05 | no | as ciborium | serde | Dependency pins only (`half`, `serde`) | `[api compare]` |
| sterraf/minicbor (fork) | https://github.com/sterraf/minicbor | no | 2025-02-10 (pushed) | no | as minicbor | as minicbor | Branch `EnumFlatDerive` is 4 ahead: "WIP: Flat attribute and encode", "Flat decoding". This is the only flatten-style derive work found, and the closest analogue to Hubris's `EncodeFields`. Fork description: "Experimenting on minicbor-derive". | `[api compare twittner/minicbor develop...sterraf:EnumFlatDerive]` |
| chrysn-pull-requests/cbor (fork of serde_cbor) | https://github.com/chrysn-pull-requests/cbor | no | 2019-02-02 (newest branch commit) | no | no_std experiments | serde | Branches `no_std_experiments` (32 ahead) and `no_std-bis` (39 ahead) date from 2018-11. Historical: serde_cbor merged no_std support in PR #96 (2019-02-27). | `[api compare pyfisch/cbor master...chrysn-pull-requests:no_std-bis]`; `gh search` PR list |
| Hubris `microcbor` / `microcbor-derive` | https://github.com/oxidecomputer/hubris/tree/master/lib/microcbor | no (in-tree) | 2026-02-26 | no | `#![no_std]` | own derive on top of `minicbor::Encode` + `StaticCborLen` (compile-time max length), `EncodeFields` (flatten without alloc) | PR #2246 by `hawkw`, merged 2025-10-28 | `[gh oxidecomputer/hubris lib/microcbor/src/lib.rs]` |
| Caliptra `caliptra-ocp-eat` | https://github.com/chipsalliance/caliptra-sw/tree/main/common/eat | no (in-tree) | 2026-02-23 (path) | no | "no_std compatible CBOR encoder with a fixed buffer" | hand-written encoder | minicbor 0.25 is a **dev-dependency** only | `[gh … common/eat/Cargo.toml, src/cbor.rs:1-6]` |
| JPHutchins/minicbor (fork) | https://github.com/JPHutchins/minicbor | no | 2024-06-28 | no | as minicbor | as minicbor | Requester's own fork; listed for completeness only | `[api]` |

Fork coverage:
- Forks of ciborium, serde_cbor, minicbor, cbor4ii, cbor-smol and kanidm/cbor that were pushed more than a day after creation were listed via `repos/O/R/forks`.
- About 25 of them were compared branch by branch against upstream.
- Forks not tabulated carry only dependency bumps, single upstream-PR branches, or formatting: cabo, AndrewScull, ahmedcharles, vpetrigo, elrafoon, furiosa-ai, ldclabs (ciborium); mesalock-linux/cbor-sgx (SGX port), fortanix, dr-bonez (serde_cbor); lublak, sdbondi, carloskiki, chrysn-pull-requests (minicbor).

Excluded: 233 crates.io crates mention CBOR in name, description or keywords. About 180 of them are application or protocol wire types, CLIs and viewers, DAG-CBOR/IPLD, blockchain (Cardano/IC/EVM), web-framework glue, or std-only codecs, and they are not tabulated. Std-only codecs seen include seabored, cbor-core, cbor_next, cbored, oasis-cbor, facet-cbor, cbor-data, cbor-edn, cbor-diag, moz_cbor, cbor-codec, `cbor` (BurntSushi, "DEPRECATED AND UNMAINTAINED") and aerolog-cbor; for each, no `no_std` attribute was found in `src/lib.rs` `[crate …]`.

## 2. Who uses what in embedded

| project | URL | CBOR crate used | evidence |
|---|---|---|---|
| Trussed | https://github.com/trussed-dev/trussed | cbor-smol 0.5 (optional, `heapless-bytes-v0-5`), cosey; serde-indexed 0.1 in `core` | `Cargo.toml`, `core/Cargo.toml`; PR #157 "Remove serde-cbor dependency" (2024-03-28) |
| ctap-types (Trussed) | https://github.com/trussed-dev/ctap-types | cbor-smol 0.5, serde-indexed 0.1.1, cosey 0.4; `ciborium = "0.2"` also listed (section not checked) | `Cargo.toml` |
| SoloKeys solo2 | https://github.com/solokeys/solo2 | cbor-smol 0.5.1, serde-indexed 0.1.1, cosey 0.4.0; serde_cbor 0.11.2 also in lock | `Cargo.lock` |
| Nitrokey 3 firmware | https://github.com/Nitrokey/nitrokey-3-firmware | cbor-smol 0.5.1, serde-indexed 0.1.1, cosey 0.4.0 (nrf52840-hal) | `Cargo.lock` |
| Google OpenSK | https://github.com/google/OpenSK | sk-cbor 0.2.0 (its own `libraries/cbor`, alloc) | `Cargo.lock`, `libraries/cbor/Cargo.toml` |
| lakers (EDHOC) | https://github.com/lake-rs/lakers (formerly openwsn-berkeley/lakers) | **no crate in core.** In-tree `mod cbor_decoder` ("Decoder inspired by the minicbor crate"). serde_cbor only in `examples/coap`. | `shared/src/lib.rs:1291-1313`, `examples/coap/Cargo.toml`; bugs #444, #445 (2026-10-16) |
| Ariel OS (coapcore) | https://github.com/ariel-os/ariel-os | minicbor 2 + derive, minicbor-adapters 0.0.8, cboritem | `src/lib/coapcore/Cargo.toml:31-32`, `Cargo.lock` |
| Ariel OS dress-up (SUIT) | https://github.com/ariel-os/dress-up | minicbor 2.2.1 (`half`) | `[crate dress-up-0.1.1 Cargo.toml]` |
| coap-handler-implementations | https://codeberg.org/chrysn/coap-tools | minicbor ^2 (default features off), minicbor-adapters ^0.0.8 | crates.io dependencies API, 0.6.4 |
| Tock OS | https://github.com/tock/tock | none | `Cargo.lock` grep |
| libtock-rs | https://github.com/tock/libtock-rs | none | all `Cargo.toml` grep |
| zephyr-lang-rust | https://github.com/zephyrproject-rtos/zephyr-lang-rust | none; no issues match "cbor" | all `Cargo.toml` grep; `gh search issues` |
| Oxide Hubris | https://github.com/oxidecomputer/hubris | minicbor 2.1.1 + **in-tree microcbor/microcbor-derive**, minicbor-lease; ciborium 0.2.2 in lock (consumer not identified) | `Cargo.lock`, `lib/microcbor/` |
| Caliptra RoT | https://github.com/chipsalliance/caliptra-sw | **in-tree** fixed-buffer encoder; minicbor dev-dependency only | `common/eat/Cargo.toml`, `common/eat/src/cbor.rs` |
| MCUmgr host: mcumgr-toolkit | https://github.com/Finomnis/mcumgr-toolkit | ciborium 0.2.2 | `Cargo.toml` |
| MCUmgr host: smp-rs (mcumgr-smp) | https://github.com/Gessler-GmbH/smp-rs | ciborium 0.2 (`payload-cbor`); default features pull tokio/btleplug/serialport | `mcumgr-smp/Cargo.toml` |
| MCUmgr host: mcumgr-client | https://github.com/vouch-opensource/mcumgr-client | serde_cbor 0.11.2 | `Cargo.lock` |
| MCUmgr host: gabrieltetar/embarch-smp | https://github.com/gabrieltetar/embarch-smp | ciborium 0.2.2. A blocking serial-only SMP client over a caller-supplied `std::io::Read + Write` (echo, reset, image state, image upload), "ported from smpclient and smp". | `README.md`, `Cargo.toml` |
| **MCUmgr device side:** usimd/window-sensor (Embassy, nRF54L10) | https://github.com/usimd/window-sensor | minicbor 2 (default features off), low-level `Encoder`/`Decoder`. Implements an SMP server: echo, reset, params, image state/upload. | `firmware/src/smp.rs:1-17`, `firmware/Cargo.toml` |
| Embassy: ATOVproject/faderpunk (embassy-rp) | https://github.com/ATOVproject/faderpunk | minicbor 2.2.1 + derive | `Cargo.lock` |
| Embassy: tweedegolf/cfg-noodle | https://github.com/tweedegolf/cfg-noodle | minicbor 2 (default features off, derive) | `crates/cfg-noodle/Cargo.toml` |
| Embassy: tangybbq/keyboard-firmware (jolt-embassy-rp) | https://github.com/tangybbq/keyboard-firmware | minicbor 0.25.1 with **alloc** + derive | `jolt-embassy-rp/Cargo.toml` et al. |
| ESP-RS: yanshay/SpoolEase (esp-hal) | https://github.com/yanshay/SpoolEase | minicbor 2.1.3 + derive | `core/Cargo.lock` |
| ESP-RS: Rainymy/volumize-embedded (esp-hal + embassy) | https://github.com/Rainymy/volumize-embedded | ciborium 0.2.2 (default features off); lock includes `esp-alloc` | `Cargo.toml`, `Cargo.lock` |
| RP2350 FIDO: OpenKey-Fido2 | https://github.com/zequinha-taveira/OpenKey-Fido2 | ciborium; lock includes `embedded-alloc` | `Cargo.toml`, `Cargo.lock` |
| RP2350/STM32 FIDO with in-tree codecs | https://github.com/lutyjj/pillkey, https://github.com/eddieoz/fapico2, https://github.com/cryptsoft/Bruce | in-tree: "Bounded, zero-allocation Canonical CBOR reader and writer" (pillkey); `no_heap` canonical writer + zero-copy parser (fapico2); `pqf-cbor` "no_std + alloc, no external deps" (Bruce) | `crates/ctap/src/cbor.rs`; `apps/fido/src/cbor.rs`; `crates/cbor/Cargo.toml` |
| rs-matter | https://github.com/project-chip/rs-matter | none (TLV); no issues match "cbor" | all `Cargo.toml` grep; `gh search issues` |
| Veraison (rust-ear, rust-ccatoken, corim-rs, rust-cmw, rust-regl) | https://github.com/veraison | ciborium everywhere, plus coset (`std`) and minicbor (`std`, rust-cmw); no `no_std` attribute found | each `Cargo.toml`, `src/lib.rs` |
| FactbirdHQ/rustot ("no_std, no_alloc … AWS IoT") | https://github.com/FactbirdHQ/rustot | minicbor 0.25 + minicbor-serde 0.3.2 (optional) | `Cargo.toml` |
| quartiq/miniconf (miniconf_coap) | https://github.com/quartiq/miniconf | minicbor 2.2.2 + minicbor-serde 0.7 (default features off) | `miniconf_coap/Cargo.toml` |
| drogue-iot/embedded-update | https://github.com/drogue-iot/embedded-update | serde_cbor 0.11 (default features off) | `Cargo.toml` |
| BitBox02 firmware | https://github.com/BitBoxSwiss/bitbox02-firmware | minicbor 0.24 via **plain `cargo vendor`** (`.cargo-checksum.json` present, so not a fork) | `src/rust/Cargo.lock`, `external/vendor/minicbor/` |
| Foundation Passport (cake-signer) | https://github.com/Foundation-Devices/passport-cake-signer | minicbor 0.24.4 + derive | `Cargo.lock` |
| Keystone3 firmware | https://github.com/KeystoneHQ/keystone3-firmware | minicbor 0.19.1 + derive, cbor_event 2.4.0 | `rust/Cargo.lock` |
| Sonair | https://github.com/Sonair-AS/minicbor | **forked minicbor** ("certified subset") | branch compare (§1b) |
| kaspar030/myelin, odysseyarm/odyssey | https://github.com/kaspar030/myelin, https://github.com/odysseyarm/odyssey | minicbor 2.x + minicbor-serde | `Cargo.lock` |
| Others seen | towynlin/bristlemouth-rs; Supermagnum/Galdralag-firmware; Alan-Jowett/sonde | cbor2 1.1.6; cbor4ii 1.2.2; ciborium | `Cargo.lock` |

Caveat:
- A `Cargo.lock` hit shows that the crate is in the workspace dependency graph, not that the firmware target consumes it. Host tools, examples and tests share the lockfile.
- Rows marked `Cargo.toml`, or with a named source file, show direct use.

Patterns (fact, from the rows above):
- minicbor appears in 17 of the projects listed above.
- cbor-smol appears in the 4 Trussed-family rows.
- ciborium appears in 2 firmware rows, and both also link a heap allocator.
- Seven embedded or security projects ship an in-tree codec or derive layer: Hubris, Caliptra, lakers, pillkey, fapico2, Bruce, and r2-cbor (`reality2-ai/r2-workshop`, "no_std, zero-allocation … for MCUs with 2–8 KB SRAM").

## 3. Demand signals

| URL | date | one-line quote |
|---|---|---|
| https://github.com/oxidecomputer/hubris/blob/master/lib/microcbor/src/lib.rs (PR #2246) | 2025-10-28 | "`minicbor-derive` also lacks a way to determine the maximum needed buffer length to encode a value at compile time, which is this crate's primary reason to exist." |
| same file | 2025-10-28 | "`serde`'s `#[serde(flatten)]` attribute requires `liballoc` … This means that `#[serde(flatten)]` cannot be used to compose nested structs in Hubris ereport messages." |
| https://github.com/chipsalliance/caliptra-sw/blob/main/common/eat/src/cbor.rs (PR #3218) | 2026-01-30 | "This module provides a no_std compatible CBOR encoder with a fixed buffer." |
| https://github.com/twittner/minicbor/pull/56 (open) | 2026-03-08 | "Adds compile-time maximum CBOR encoding size calculation, useful for fixed-size stack buffers in `no_std` without `alloc`" (+1 comment 2026-04-28) |
| https://github.com/twittner/minicbor/pull/57 (open) | 2026-03-14 | "automatically uses field/variant names as string keys, removing the need to annotate every field individually" |
| https://github.com/twittner/minicbor/pull/71 (open) | 2026-09-22 | "a derived `Decode` impl for a type like `struct Tree { children: Vec<Tree> }` will overflow the stack on hostile input" |
| https://github.com/twittner/minicbor/issues/67 (closed) | 2026-07-23 | Maintainer reply: "I would not want to add those implementations here, but maybe minicbor-adapters would be interested." |
| https://gitlab.com/twittner/minicbor/-/work_items/22 (closed) | 2022-03-14 | "It would be convenient if the `alloc` requirement on derivation could be lifted at least for simple cases" |
| https://gitlab.com/twittner/minicbor/-/work_items/32 (closed) | 2022-10-04 | Title: "How to calculate the length needed for buffer when encoding data" |
| https://github.com/enarx/ciborium/issues/90 (open, 0 comments) | 2023-08-24 | "It'd be great if structs with serde based serializations could be used on embedded targets where alloc is typically unavailable." |
| https://github.com/enarx/ciborium/issues/59 (open) | 2022-10-12 | "ciborium links, if not to libstd, then at least to liballoc, regardless of the presence of the `std` feature flag." |
| https://github.com/enarx/ciborium/issues/66 (open) | 2022-12-21 | "When working in no_std environments, a heapless::Vec is a popular target to write into" |
| https://github.com/enarx/ciborium/issues/172 (open) | 2026-03-01 | "I'm currently working with embassy-usb which exposes only the async versions of the embedded-io traits" |
| https://github.com/kanidm/cbor/pull/23 (open) | 2025-12-21 | "Previously Value was under the std feature flag, meaning it could not be used in no-std contexts." |
| https://github.com/pyfisch/cbor/issues/167 (open; repo archived) | 2019-12-04 | "The code is written for an MCU environment, hence we are using no_std methods." (title: "Reduce stack size required for deserialization") |
| https://github.com/lake-rs/lakers/issues/444 | 2026-10-16 | Title: "`CBORDecoder::i8()` can underflow `u8` on two-byte negative integers (`0x38 xx`)" (in-tree decoder) |
| https://github.com/lake-rs/lakers/issues/445 | 2026-10-16 | Title: "`CBORDecoder::any_as_encoded()` slices with an unchecked index" |
| https://github.com/Sonair-AS/minicbor/tree/certified_subset | 2026-06-15 | Commit: "Create certified subset of minicbor"; "Add required documentation for 3rd party crate" |

Weak or no signal:
- The users.rust-lang.org search for "cbor no_std", "cbor embedded", "cbor heapless" and "cbor microcontroller" returned no on-topic thread.
- "Serde alternatives" (https://users.rust-lang.org/t/serde-alternatives/121903, 2024-12-01) only names minicbor in passing.

Not searched:
- The Rust Embedded matrix archives and Reddit.
- Codeberg repo search returned HTTP 429 ("your search request will not be processed") on two attempts. Known Codeberg repos were looked up directly instead.

## 4. Candidates worth contributing to

1. **minicbor** (https://github.com/twittner/minicbor)
   - Fact:
     - De facto embedded choice in §2; 207 reverse deps; active (2026-10-18); 20 contributors.
     - Core, derive and serde all work without alloc.
     - Two of the three Hubris features have open, unmerged PRs: #56 `MaxCborLen` (contributor) and #57 `text_keys` (contributor, stacked on maintainer draft #53, "Work in progress and experimental"). Flatten without alloc has none.
     - The maintainer recently merged a no-alloc `minicbor_serde::to_slice` (#72), but declined heapless impls (#67).
     - Licence is BlueOak-1.0.0.
   - Judgment:
     - Highest leverage. Reviewing, testing or landing #53/#57, #56 and #71 directly addresses the documented gap without fragmenting the ecosystem.
     - The maintainer is already experimenting with string keys (#53), so the index-keyed design is not a hard blocker.
     - Risk: one gatekeeper.
2. **tinycbor** (https://github.com/carloskiki/tinycbor)
   - Fact:
     - Derives `Encode`/`Decode`/`CborLen` and uses `embedded-io`.
     - no_std with alloc gated on paper, but 0.14.0–0.15.1 fail to build without alloc; 0.13.0 builds (`notes/crates/tinycbor.md`).
     - 192 commits, 2 contributors, 1 reverse dep.
   - Judgment:
     - Small and young, so it is likely receptive to design input, such as a static max length or no-alloc defaults. Fixing the no-alloc build regression is an obvious first contribution.
     - Weak adoption means effort here builds a second ecosystem rather than improving the main one.
3. **const-cbor** (https://github.com/larrydewey/const-cbor)
   - Fact:
     - The only project that publishes Cortex-M flash numbers: 3.1 KB core.
     - Zero deps, no alloc, no derive. The serde adapter needs alloc.
     - 7 commits, 1 contributor, repo created 2026-08-24.
   - Judgment: a plausible low-level backend under a derive layer, but its sustainability is unproven.
4. **tps_minicbor + tps_cddl** (https://github.com/GlobalPlatform/TPS-API-Reference-Implementations)
   - Fact:
     - no-alloc encoder.
     - CDDL code generator "optimized for code size".
     - GlobalPlatform-hosted, one main author, long dormancy until 2026-10.
   - Judgment: relevant if the target is schema-first (CDDL, zcbor-like) rather than derive-first.
5. **ciborium**, and talking to npmccallum (https://github.com/enarx/ciborium, https://github.com/npmccallum/ciborium)
   - Fact:
     - Largest user base, but no release since 2024-01; no-alloc request #90 open since 2023.
     - The original author's 2026-04 rewrite is unpublished, has no PR, and its serde path still needs alloc.
   - Judgment: contributing no-alloc serde here depends on a release process that has stalled. Ask about the rewrite's plans before investing.
6. **cbor-smol** (https://github.com/trussed-dev/cbor-smol)
   - Fact: proven no-alloc serde CBOR in shipping FIDO firmware; last commit 2025-10-01.
   - Judgment: the reference point if a serde-based path is chosen. See `notes/munns-serde-codesize.md` for serde code-size costs.

Not candidates (fact-based):
- zerocbor: unconditional alloc.
- strede: deserialize-only.
- sacp-cbor: repo not reachable; derive needs alloc.
- hardy-cbor: alloc.
- cbor2: deserialize needs alloc.
- Thomasdezeeuw/cbor: unpublished, no README, single author; alloc use Unknown.
