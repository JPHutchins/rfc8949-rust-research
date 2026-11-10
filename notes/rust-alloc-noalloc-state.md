# alloc vs no-alloc APIs in Rust at large — state as of 2026-11-10

Not CBOR-specific. Every item was checked against the upstream issue, PR, crate or a local compile on the date above.

## Summary

- **New: the `Allocator` trait is stabilized for Rust 1.100.0.** rust-lang/rust#156882 "alloc: stabilise `Allocator`" was merged 2026-09-23 with milestone 1.100.0, and tracking issue #32838 was closed as completed.
  - Release is expected around 2026-11-12, two days from now. That date is inferred from the six-week cadence; the milestone has no due date.
  - RalfJung, on #32838 (2026-09-23): "If nothing goes wrong, then yes it will" ship in 1.100.
  - Checked on rust-lang/rust's `stable` branch (`src/version` = 1.100.0):
    - `library/core/src/alloc/mod.rs:196`: `#[stable(feature = "allocator_api", since = "1.100.0")]` on `pub const unsafe trait Allocator`;
    - `library/alloc/src/vec/mod.rs:934`: the same attribute on `Vec::with_capacity_in`.
    - So it is on the release branch. Only the public release itself is still pending.
- **The stable MVP surface** (from the PR's stabilisation report):
  - `unsafe trait Allocator` with `allocate`, `deallocate`, `allocate_zeroed`, `grow`, …;
  - `Global`;
  - `Box<T, A>` with `new_in`, `from_raw_in`, `from_non_null_in`;
  - `Vec<T, A>` with `new_in`, `with_capacity_in`, `from_raw_parts_in`, `from_parts_in`.
  - The rest of the old feature becomes unstable `allocator_ext` (#163177).
- **Not there yet** (wg-allocators "Roadmap 2.0", rust-lang/wg-allocators#150, opened 2026-08-08):
  - allocator-generic `String`;
  - **moving collection types into `core`**;
  - more fallible collection methods;
  - `Box::into_pin` with custom allocators (held back by soundness bug #157089);
  - the `Clone`-related safety marker trait.
- **Embedded catch (ran it):** linking the `alloc` crate still requires a `#[global_allocator]`.
  - A `#![no_std]` thumbv7m binary with only `extern crate alloc` fails: "no global memory allocator found but one is required; link to std or add `#[global_allocator]`" (rustc 1.95.0).
  - **Tested on 1.95 only.** Nothing in the 1.100 stabilization report removes the requirement, and "collections into `core`" is still open on the roadmap, so it most likely still holds. Under that assumption, even after 1.100 a heapless firmware using `Vec<T, MyArena>` must register a global allocator, at minimum one that always fails, until collections move to `core`.

## The toolbox, from most to least established

| Approach | Status | What it gives | Evidence |
|---|---|---|---|
| Additive `alloc` / `std` Cargo features (`std = ["alloc"]`), gating `extern crate alloc` and alloc-typed impls | Universal convention | One crate serving both worlds; no-alloc users simply lack the gated APIs | minicbor `Cargo.toml` (`alloc = []`, `std = ["alloc"]`); smoltcp (`std = ["managed/std", "alloc"]`) |
| Caller-provided buffers / sink and source traits | Universal | Core logic never allocates; alloc adds a `Vec`-backed sink | minicbor `encode::Write` for `&mut [u8]` / `[u8; N]` / `Vec<u8>`; postcard flavors (`notes/design-sketch.md`) |
| Fixed-capacity collections | Stable crates | No allocator at all | heapless 0.9.3 (`Vec<T, N>`, capacity-erased `VecView`), arrayvec 0.7.8, tinyvec 1.13.3 (`ArrayVec`; `TinyVec` spills to heap behind `alloc`) |
| Borrowed-or-owned enums | Stable crates | One type, with `Owned` compiled only under `alloc` | `managed::ManagedSlice` (smoltcp) |
| **Allocator-generic collections `Vec<T, A>` / `Box<T, A>`** | **Stable in 1.100.0**. Before that: nightly, or the `allocator-api2` 0.4.0 polyfill (≈177 M recent downloads; optional dependency of hashbrown 0.17.1; used by bumpalo) | One code path; the caller picks heap, bump arena, or a static-buffer arena | rust-lang/rust#156882; crates.io |
| Global heap on the MCU | Stable crates | Just use `alloc` everywhere | embedded-alloc 0.7.0 (rust-embedded), talc 5.1.1 |
| **Store API** (`Vec<T, S: Store>`: inline, shared-memory or heap storage behind one trait) | **RFC only**: rust-lang/rfcs#3446, open since 2023-06-17, last updated 2026-10-02 | Would unify fixed-buffer and heap collections in one type | RFC text: "suitable for in-line memory store, shared memory store, compaction of allocations"; companion repo github.com/matthieu-m/storage |
| `ArrayVec` in `core` | RFC only: rust-lang/rfcs#3316, open since 2022-09-15 | A std fixed-capacity vector | — |
| Fallible allocation | Partial. `try_reserve` is stable. `Vec::push_within_capacity` is unstable (#100486, open); `alloc_error_hook` is unstable (#51245, open) | Recovering from OOM instead of aborting | tracking issues |
| `core` I/O buffers | Unstable: RFC 2930 read-buf (`BorrowedBuf`), #78485 open | — | Ecosystem uses `embedded-io` 0.7.1 instead |

## What this means for a library today *(inference)*

- Keep the core allocation-free: caller buffers plus sink/source traits, with `alloc` and `std` as additive features. This works on every toolchain and target.
- Treat `A: Allocator` generics (stable 1.100, or `allocator-api2` for older MSRV) as the option for owned collections. It lets a no-heap caller supply an arena, but the `alloc` crate still needs a global allocator registered.
- Watch Store API (RFC #3446) and "collections into `core`" (Roadmap 2.0). Either would remove the global-allocator requirement and make one-type-for-both-worlds possible without the `alloc` crate.
