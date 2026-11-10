# Design sketch: no_std, table-driven, optionally-serde CBOR

User constraints (2026-11-10):
- alloc **or** no-alloc (static / stack buffers);
- no recursion;
- serde **or** no serde;
- Munns-style flash optimizations.

Items marked *(inference)* are design judgment, not measured.

## Crate shape: proc macro + small runtime

A `proc-macro = true` crate can export only macros, so the design needs two crates.

| Crate | Contents |
|---|---|
| `foo-derive` (most of the logic) | Per type, emits **const data, not code**:<br>• a descriptor with field kinds;<br>• `core::mem::offset_of!` offsets;<br>• pre-encoded key bytes, sorted bytewise at compile time per RFC 8949 §4.2.1;<br>• links to nested descriptors;<br>• `const MAX_ENCODED_LEN` and `const MAX_DEPTH`. |
| `foo` (`#![no_std]`) | • Wire primitives;<br>• **one** non-generic, iterative engine that walks descriptors;<br>• `Write`/`Read` sinks;<br>• features `alloc` and `serde`. |

The per-type generic surface shrinks to a thin typed wrapper around the type-erased engine. Munns' postcard-forth measured that design at −38% total `.text`, or −59% without enums (`sources/munns/postcard-forth-rp2040-results-2024-06.md`: 640,456→395,712 B and 550,012→223,492 B). He also notes that enums still needed generated code.

## Trade-off: table-driven vs "fully type safe" *(inference)*

- A type-erased engine walking `offset_of!` offsets and function pointers must use `unsafe`: raw-pointer field access, and per-kind casts behind function pointers. Munns called postcard-forth "wildly unsafe" (`notes/munns-serde-codesize.md`).
- The type safety the user wants can still live at the API boundary.
  - The derive generates descriptors only from the type definition it sees, so offsets and kinds agree by construction.
  - The public API is `fn decode<T: Cbor>(…) -> Result<T, _>`.
  - `unsafe` stays confined to the engine and generated consts, which makes it a Miri and fuzzing target.

## alloc / no-alloc: verified precedents

| Pattern | Precedent | Evidence |
|---|---|---|
| Output-sink trait with impls for `&mut [u8]`, `[u8; N]`, and `Vec<u8>` behind `alloc` | minicbor `encode::Write` | `vendor/minicbor/minicbor/src/encode/write.rs:19,35,82,95` |
| Pluggable output "flavors" | postcard `ser_flavors::Flavor`: `Slice`, `HVec<B>` (heapless), `AllocVec`, `Cobs` | `postcard-1.1.3/src/ser/flavors.rs:113,139,381,450,515` |
| One entry point per storage | postcard `to_slice` / `to_vec::<B>` (heapless) / `to_allocvec` / `to_eio` / `to_io` | `postcard-1.1.3/src/ser/mod.rs:85,154,224,290,311` |
| Borrowed-or-owned buffer enum, with the owned variant `cfg(alloc)` | `managed::ManagedSlice { Borrowed(&mut [T]), Owned(Vec<T>) }`, used by smoltcp (`alloc = ["managed/alloc"]`) | `m-labs/rust-managed src/slice.rs:29-34`; smoltcp `Cargo.toml:23,44` |
| Storage-generic container (owned fixed-capacity vs borrowed view) | heapless 0.9 `VecInner<T, LenT, S: VecStorage>` → `Vec<T, N>` / `VecView<T>` | `heapless src/vec/mod.rs:227,277,301` |
| Compile-time max size so the caller can use a static buffer | postcard `MaxSize::POSTCARD_MAX_SIZE`; minicbor PR #56 `MaxCborLen` (open) | `postcard-1.1.3/src/max_size.rs:23-26` |
| Exact length before encoding (allocate once, or check a static buffer) | minicbor `CborLen` | `vendor/minicbor/minicbor/src/encode.rs:52` |
| Caller-provided decode scratch | `serde_cbor::de::from_slice_with_scratch(input, &mut scratch)` | `experiments/probes/serde_cbor/src/lib.rs` |
| no_std I/O traits | `embedded-io` (≈80 M recent downloads) | crates.io |

On the decode side, field types choose the storage, so the derive does not care:
- `&'a str` / `&'a [u8]` for zero-copy;
- `heapless::String<N>` for fixed capacity;
- `String` / `Vec<T>` for heap, with impls gated behind `alloc`.

## No recursion *(inference, except where cited)*

- A non-recursive Rust type has a finite nesting depth that the derive can compute, emitted as `MAX_DEPTH`. The engine keeps an explicit `[Frame; N]` stack.
  - Stable Rust can't write `[Frame; T::MAX_DEPTH]` in generic code, because `generic_const_exprs` is unstable.
  - Instead, take `N` as a const generic at the entry point and check it with `const { assert!(N >= T::MAX_DEPTH) }`.
  - Verified on rustc 1.95: a binary calling `decode::<Shallow, 2>` with `MAX_DEPTH = 3` fails with `E0080 … stack too small for T`, and `N = 3` compiles.
  - Caveat: it is a post-monomorphization error. It fires only when a final binary instantiates the call, not in `cargo check` or in a library that never instantiates it.
- Skipping unknown **definite-length** values needs only a counter: arrays add n, maps add 2n, tags add 1. No stack.
  - **Indefinite-length** containers also need one bit per open level to match `break` codes, so a `u64` bitset bounds them at 64 levels.
- Recursive types (`Box<Self>`) already require alloc. Under `alloc` the engine can grow a `Vec<Frame>` with a configured limit.
- Today's crates mostly recurse: cbor-smol, cbor_event and facet-cbor overflowed the stack in our probes. Others use fixed recursion limits (`notes/crate-matrix.md`).

## Serde: make it opt-in

- serde_derive 1.0.229 still generates a `__Field` identifier type and a visitor per struct (`serde_derive/src/de/struct_.rs`, `identifier.rs`). Every derived type therefore instantiates the format's `Deserializer` methods again. A format crate can keep those methods lean, as minicbor-serde does, but cannot remove the per-type instantiations.
- dtolnay's preferred fix is "handing off a kind of type-erased bytecode for the deserializer to evaluate … Deserialize impls become free — basically a single `static`" (`sources/munns/serde-pr2250-dtolnay-review-2023-08-06.txt`). That would be a change inside serde, and it is unshipped.
- **The serde path cannot be non-recursive.** `Deserialize::deserialize` → `Visitor::visit_map` → nested `Deserialize::deserialize` uses the call stack by construction. An opt-in serde layer can enforce a depth *limit* (as serde_cbor does at 128), but cannot use the iterative engine.
- So the native derive is the table-driven fast path, and a `serde` feature implements `Serializer` / `Deserializer` on the same runtime so existing serde types still work, at serde's per-type cost. Measured cost: `notes/codesize.md`.

## Reuse minicbor's wire layer?

- **For:** born no_std (first commit 2020-03-02); RFC 8949 Appendix A 81/81 (`experiments/conformance/appendix-a/minicbor`, re-run 2026-11-10); maintained; decoder primitives are about 1.5 KB on thumbv7m.
- **Against:**
  - Floats are encoded at their declared width rather than in preferred serialization, so RFC 8949 §4.2.1 deterministic output for float fields needs its own float path.
  - It emits and accepts `f8 14`…`f8 1f`, which RFC 8949 §3.3 forbids (test `simple_values_20_to_31_use_not_well_formed_two_byte_form`).
- *(inference)* The wire layer is small enough to own outright if those two points matter. Depending on minicbor would mean shipping workarounds alongside it.
