# James Munns: serde code size and table-driven alternatives

## Executive summary

- **Best match for the remembered video:** Self-Directed Research (SDR) episode **"A different serde"** (2024-11-27). James presents it and Amos co-hosts. Video: <https://www.youtube.com/watch?v=qlvr4wqAIfg>. A full transcript is published on the [episode page](https://sdr-podcast.com/episodes/a-different-serde/). This is the best candidate, not confirmed as the one the user remembers. I found no other video presented by Munns that focuses on this topic, though S2 and S4 touch on it.
- **How the bloat happens, in his words:** serde's derive writes visitor-driving code for each type. That code is generic, so the compiler "stamp[s] out a version" for each serializer. The optimizer then inlines those copies into large functions, which are repeated at every inline site, and this "end[s] up as a lot of code in your binary".
- **Flash numbers he measured:** RP2040 firmware (`thumbv6m`, `opt-level="s"`, LTO), 512 generated nested types, round-tripped through deserialize and serialize:
  - postcard+serde `.text` was **640,456 B**; postcard-forth was **395,712 B** (61.8%).
  - With no enums: **550,012 → 223,492 B** (40.6%).
  - Expanded source shrank to about one third, and crate compile time to about 45%.
  - He reported **no function or symbol counts**.
- **What he built:** **postcard-forth**, an experiment that is not on crates.io (last push 2024-06-13).
  - For structs, the derive writes **only a const table of `(offset_of!, fn pointer)`**. A **shared interpreter loop** walks that table.
  - In the repo, each composite type still gets a thin generic shim (`ser_fields::<T>`) around that loop.
  - Limits: enums still need generated code, the engine is "wildly unsafe", and it supports a single wire format.
  - dtolnay independently called this kind of "type-erased bytecode" the ideal design.
- **What it means for a CBOR codec:** make each type's cost static data, not code, and run it through one shared, type-erased engine built for one wire format. Treat enums as the hard case. See section 5 for which points are cited and which are my inference.

---

## 1. Sources

| # | Title | URL | Date | Type | Transcript / full text read? | Local copy |
|---|---|---|---|---|---|---|
| S1 | A different serde (SDR podcast) | <https://sdr-podcast.com/episodes/a-different-serde/> · video <https://www.youtube.com/watch?v=qlvr4wqAIfg> · slides `/slides/2024-11-27-a-different-serde.pdf` (not read) | 2024-11-27 | podcast + YouTube video | **Yes, end to end** (published transcript). YouTube auto-captions used only for approximate timestamps | `sources/munns/sdr-2024-11-27-a-different-serde.txt`, `…yt-autosubs.txt`, `qlvr4wqAIfg/qlvr4wqAIfg.en.vtt` |
| S2 | Merde is not Serde (SDR; Amos presents) | <https://sdr-podcast.com/episodes/merde-is-not-serde/> · video <https://www.youtube.com/watch?v=8MeZopyQ_UM> | 2024-12-04 | podcast + video | Relevant sections read; rest keyword-searched | `sources/munns/sdr-2024-12-04-merde-is-not-serde.txt` |
| S3 | What good is partial understanding? (SDR) | <https://sdr-podcast.com/episodes/partial-understanding/> | 2024-10-30 | podcast + video | Relevant sections read; rest keyword-searched | `sources/munns/sdr-2024-10-30-partial-understanding.txt` |
| S4 | I Was Wrong About Rust Build Times (SDR; Amos presents) | <https://sdr-podcast.com/episodes/wrong-build-times/> | 2024-08-07 | podcast + video | Relevant section read; rest keyword-searched | `sources/munns/sdr-2024-08-07-wrong-build-times.txt` |
| S4b | Talking to Microcontrollers with Postcard-RPC (SDR) | <https://sdr-podcast.com/episodes/postcard-rpc/> | 2024-09-11 | podcast + video | Keyword-searched only (cited for one CBOR line) | `sources/munns/sdr-2024-09-11-postcard-rpc.txt` |
| S5 | Building Plain Old Data from Scratch | <https://onevariable.com/blog/pods-from-scratch/> | 2024-06-06 | blog | postcard-forth sections read; rest keyword-searched | `sources/munns/onevariable-2024-06-06-pods-from-scratch.txt` |
| S6 | Postcard 2.0: Looking for Sponsors! | <https://onevariable.com/blog/postcard-2-sponsors/> | 2024-09-25 | blog | Relevant paragraph read; rest keyword-searched | `sources/munns/onevariable-2024-09-25-postcard-2-sponsors.txt` |
| S7 | Munns Bluesky thread on serde expansion | <https://bsky.app/profile/jamesmunns.com/post/3ktsi3xdgy62u> | 2024-05-31 | social | Yes (via public API) | `sources/munns/bsky-2024-munns-dtolnay-threads.txt` |
| S8 | dtolnay reply to the S1 announcement, plus Munns' reply | <https://bsky.app/profile/dtolnay.bsky.social/post/3lby6hlugt22q> | 2024-11-27/28 | social | Yes | same file as S7 |
| S9 | dtolnay review on serde PR #2250, "Shrink derive output size" (closed, unmerged) | <https://github.com/serde-rs/serde/pull/2250#pullrequestreview-1564212707> | 2023-08-06 | GitHub review | Yes (verbatim) | `sources/munns/serde-pr2250-dtolnay-review-2023-08-06.txt` |
| S10 | `jamesmunns/postcard-forth`: README (speed benchmark), `source/rp2040-demo/results.md`, `Cargo.toml`, `.cargo/config.toml`, `postcard-forth/src/lib.rs`, `stress-gen/src/main.rs` | <https://github.com/jamesmunns/postcard-forth> | created 2024-06-02, pushed 2024-06-13 | repo | Yes (files listed) | `sources/munns/postcard-forth-rp2040-results-2024-06.md` |
| S11 | `jamesmunns/postcard` workspace: `postcard-core` (README, start of `lib.rs`, PR #285), `postcard2` README, `-ng` crate manifests | <https://github.com/jamesmunns/postcard> | 2025-12 to 2026-11 | repo | Partially (files listed) | none |
| S12 | "Embedded Rust and Serde", Meeting Embedded talk slides | <https://github.com/ferrous-systems/embedded-rust-and-serde> (`index.md`) | 2019-11-13 | slides | Keyword-searched plus matching slides read; no recording found | none |

**Searched but not found or not checked**

- I found no conference talk recording (RustConf, Oxidize, EuroRust, RustNL) by Munns on serde code size. This rests on web searches, not on a crawl of every conference channel.
- The 2023 cohost post "What good is partial understanding" (linked from S3) was not retrieved, because cohost has shut down.
- Blogs: I listed `onevariable.com/blog` pages 1 to 3 and `jamesmunns.com/blog` page 1 (2022 postcard posts only). Later pages were not checked.
- SDR feed: 32 episodes, the last on 2025-08-13. I keyword-searched 9 likely episodes: S1–S4, S4b, Compile Time Crimes, BBQueue, The Embedded Buddy System, and sans-io. Only S1 was read end to end. The other 23 episodes were not opened.
- I tried Bluesky full-text search, but the public `searchPosts` endpoint returned a non-JSON response, so I skipped it.

---

## 2. The bloat mechanism, as he explains it

Quote wording comes from the published transcript (S1) unless noted otherwise. `~mm:ss` is an approximate position in the YouTube video, taken from the auto-captions. The speaker is named on each quote; several of the sharpest lines are Amos's or dtolnay's, not Munns's.

### 2.1 The derive generates per-type visitor-driving code

[S1 §the-visitor-pattern](https://sdr-podcast.com/episodes/a-different-serde/#the-visitor-pattern), ~06:30. **Munns:**
> "Now this visitor driver code is usually derived. You usually don't have to write this even for other types. You just throw a derive serialize on there and a procedural macro goes in and generates: okay, I have seven fields. The first field is this. Do this, do this, do this, do this. And it expands out into essentially driving that visitor at all points."

### 2.2 "It generates a LOT of code"

[S1 §problem-this-generates-a-lot-of-code](https://sdr-podcast.com/episodes/a-different-serde/#problem-this-generates-a-lot-of-code), ~08:18. **Munns:**
> "So the main problem is that it generates a lot of code. It can generate a sometimes double digit multiple of the input source that you are giving it."

### 2.3 Monomorphization and then inlining multiply it in the binary

Same section, ~09:29–10:04. **Munns:**
> "And like you were saying, there's a lot of monomorphization and inlining. This isn't necessarily serde's fault, but Rust, like you said: generics are templates, basically, so it will stamp out a version of this. If you have multiple serializers, it will stamp out a version of each of these that are used. And when it comes to the optimizer, we will inline a lot of this, which means some of those instantiated copies will end up being one very large function, and then you'll have multiple versions of that function because they get inlined in different places.
> And so it can end up taking a fair amount of compilation and optimization time, but then end up as a lot of code in your binary as well."

The same section has two lines from **Amos, not Munns**:
> "it's all generic code as well, so it's going to be instantiated using a serializer or a deserializer implementation."

> "the reason serde has methods per type is that it can inline everything. There's no dynamic dispatch at all. It all gets compiled down to just compact code, and you pay for that in compile times."

### 2.4 The contrast: one machine instead of one copy per type

[S1 §good-things-first](https://sdr-podcast.com/episodes/a-different-serde/#good-things-first), ~27:02. **Munns:**
> "One good thing is that there's only ever one serialization deserialization machine. It doesn't get monomorphized for every single version of that. And serde generates a very simple, easy to optimize chunk of code, but it generates one of those for everything. And in this, there is just one: the VM that handles whatever data that you throw at it."

### 2.5 Recursion and by-value returns (a stack/RAM cost, not flash)

[S1 §problem-the-visitor-pattern-is-recursive](https://sdr-podcast.com/episodes/a-different-serde/#problem-the-visitor-pattern-is-recursive), ~12:34. **Munns:**
> "Especially for deserialization, data is returned by value. And sometimes because we don't have a heap vector, we will use stack allocated vectors where you say: okay, here's a bounded vector with room for 32 items. And when you return that deserialization step by value, even if you only deserialized one item, if it's a fixed size collection, essentially you have the capacity for all 32, you're going to return that by value down the stack frame. Which means you've created that whole 32 item vec in one function, and then returning it by value means you memcpy it back to the caller frame, essentially."
>
> "And this is one of those things that LLVM can optimize sometimes, but RVO is not a guaranteed optimization."

### 2.6 Hedges he attached (keep these with any citation)

- **Unsure whether postcard pays for serde features it doesn't use.** [S1 §postcard-more-flexibility-than-we-need](https://sdr-podcast.com/episodes/a-different-serde/#postcard-more-flexibility-than-we-need), ~15:15:
  > "I think it's one of those things that if it doesn't exist, it's almost certainly optimized out. I believe that serde has the latitude for that, but I don't necessarily think that we're paying extra for that in postcard by not using it. […] I'd have to check. The answer is I don't know."
- **Keep using serde.** ~02:26:
  > "Despite having messed around with other things today, you should absolutely keep using serde..."
- **Not exhaustive.** ~32:00:
  > "It's not an exhaustive test, you know, lies, damn lies and benchmarks."
- **The pain he measured was compile time on a desktop project.** ~34:14, asked whether the customer should adopt postcard-forth:
  > "No! No, no, no, no. They do not need this. They're doing this on a desktop."
- **No smoking gun.** [S4](https://sdr-podcast.com/episodes/wrong-build-times/):
  > "So I don't have like a smoking bullet that it was serde, but serde and one other proc macro were generating like literally seven or eight times the size of an already 50,000 line Rust crate."
- **Code size was the main goal before he measured it.** [S5](https://onevariable.com/blog/pods-from-scratch/):
  > "My hope is that this could generate less code at compile time, and potentially end up with faster compile times (due to less code - which matters when you have a LOT of #[derive(Serialize, Deserialize)] instances in your project, and potentially smaller code for embedded systems."
  >
  > "I still need to come up with an A/B test to compare the volume of generated code, the compile time, and the code size in a bare metal embedded program […] as this was the primary objective, ser/de speed was only a check to ensure that I wasn't introducing a regression in performance."

---

## 3. Numbers he reported

### 3.1 Flash (`.text`) A/B on RP2040: the main measurement

Source: S10 [`source/rp2040-demo/results.md`](https://github.com/jamesmunns/postcard-forth/blob/main/source/rp2040-demo/results.md), rows verbatim. He describes the table in S1 [§serialization-benchmarks](https://sdr-podcast.com/episodes/a-different-serde/#serialization-benchmarks), ~32:09–33:52.

**Conditions**, from the repo:
- Target `thumbv6m-none-eabi` (Cortex-M0+, RP2040), with embassy, defmt and panic-probe.
- `[profile.release] opt-level='s', lto=true, codegen-units=1`.
- `postcard = "=1.0.8"` and `serde 1.0` (default features off) against postcard-forth with its derive.
- Sizes come from `cargo size --release`. Result directories are timestamped 2024-06-09/10.
- `stress-gen` uses a fixed seed to generate nested types: structs ×2, enums ×2, arrays, options, tuples and tuple structs, each referencing earlier types or primitives.
- The firmware deserializes each type from host input and re-serializes it, so nothing can be optimized out.
- Measured totals include the 10,584 B baseline image. The baseline is the firmware with no ser/de, which he describes as "something like 10 kilobytes" and 53 expanded lines (~32:36).

| case | types | text size (B) | input lines | expanded lines | ttl time | crate time |
|---|---|---|---|---|---|---|
| baseline | 0 | 10584 | 0 | 53 | 13.3s | 0.33s |
| postcard-serde | 128 | 164000 | 2664 | 47240 | 20.1s | 7.03s |
| postcard-forth | 128 | 111220 | 2664 | 15931 | 16.4s | 3.14s |
| postcard-serde | 512 | 640456 | 10244 | 181583 | 39.7s | 25.95s |
| postcard-forth | 512 | 395712 | 10244 | 60232 | 24.6s | 11.82s |
| postcard-forth (inlined) | 512 | 389944 | 10244 | 79471 | 24.8s | 11.78s |
| postcard-serde (no enums) | 512 | 550012 | 8248 | 72610 | 33.1s | 19.7s |
| postcard-forth (no enums) | 512 | 223492 | 8248 | 20594 | 19.6s | 6.82s |
| postcard-serde (onlyprims) | 512 | 610800 | 10248 | 177647 | 45.6s | 32.4s |
| postcard-forth (onlyprims) | 512 | 295704 | 10248 | 59645 | 22.3s | 9.63s |

**Derived figures (my arithmetic, not his):**

| pair | forth/serde `.text` | net of baseline | net B per generated type, serde → forth | expanded lines ratio | crate-time ratio |
|---|---|---|---|---|---|
| 128 types | 67.8% | 65.6% | 1,199 → 786 | 33.7% | 44.7% |
| 512 types | 61.8% | 61.1% | 1,230 → 752 | 33.2% | 45.5% |
| 512, no enums | **40.6%** | 39.5% | 1,054 → 416 | 28.4% | 34.6% |
| 512, onlyprims | 48.4% | 47.5% | 1,172 → 557 | 33.6% | 29.7% |

- The "per type" figures average each type's serialize and deserialize paths together with the round-trip harness. They are not per-function sizes.
- The "no enums" and "onlyprims" runs used different generated sets: their input-line counts differ from the main 512-type row. Compare ratios within a row, not absolutes across rows.

**What he said about these numbers** (S1, ~32:55–33:52):
> "The compiled code size for serde was 640,000 bytes, and the compiled code size for postcard-forth was 395,000 bytes, so like two thirds of the size."
>
> "this is like a mean test of generating 512 types where most of them include seven or eight of different ones. It's about as bad as it can get. But in this stress test, the sizes are typically about a third of generated code, about two thirds of the actual compiled binary size after optimizations."

**Why the no-enums gap is bigger.** postcard-forth can describe a struct as a table, but enums still need a generated function. [S5](https://onevariable.com/blog/pods-from-scratch/):
> "Unfortunately, there's no such ability for enums, which means I have to generate a full deserialization function (instead of just a simple const list), like serde does"

In S1 he says he added the no-enums row "because I was trying to figure out how bad that was when compared to other types".

**Not reported.** He gave no function, symbol or monomorphized-instance counts, and no per-type flash cost. The per-type figures above are mine.

### 3.2 Generated-code volume (a desktop client project; drove the compile-time concern)

- **S7 (Bluesky, 2024-05-31):**
  > "I have a client project that takes ~40s to compile their main crate. To be fair, it IS 50kloc in one crate, but after using `cargo-expand` and `form`, it's actually 200kloc after macro expansion! One file went from 1.1kloc to *22kloc*, almost all serde impls"
  >
  > "An 8 line struct, no generics, with 6 fields, generates 451 lines of serialization and deserialization code."
- **S1** repeats this: 50,000 to "over 200,000 lines", a file of a little over 1,000 lines becoming 22,000, and a 40-second build.
- **S4** gives a different multiple:
  > "seven or eight times the size of an already 50,000 line Rust crate"

  and, as an example, "2,000 lines before expansion and 54,000 lines after expansion". Asked whether that was exact, he answered "For some of them, yeah", meaning the files that were "just data structures".
- I have not reconciled the 4× (S7) and 7–8× (S4) multiples.

### 3.3 Runtime speed (desktop; secondary)

Source: the S10 README, which is output of `rust_serialization_benchmark` (2024-06-10, rustc 1.77.2, aarch64-apple-darwin).

| dataset | serialize: postcard as % of postcard-forth speed | deserialize: postcard as % of postcard-forth speed |
|---|---|---|
| log | 67.71% | 90.57% |
| mesh | forth is 84.07% of postcard (postcard faster) | 56.44% |
| minecraft_savedata | 67.11% | 86.19% |
| mk48 | 58.05% | 67.48% |

Encoded sizes are identical, because the wire format is the same. In S1 he summarizes it as "on seven of eight tests, it's faster". Cite the table, not his spoken percentages.

---

## 4. Alternatives he proposed or built

### 4.1 postcard-forth (built; experimental)

- **Repo:** <https://github.com/jamesmunns/postcard-forth>. Created 2024-06-02; `pushed_at` 2024-06-13T00:53:03Z.
- **Status:** not on crates.io; the crates.io API returns "does not exist" for both `postcard-forth` and `postcard_forth`. He calls it "not ready to use by any sort of the imagination" (S1 §postcard-forth).

**How it works** (S1 §postcard-forth and §turn-ser-de-into-a-stack-machine; S5; S10 `lib.rs`):

- **The derive emits data, not a visitor.** For each struct it writes `unsafe impl Serialize { const FIELDS: &'static [SerField] }`, where `SerField { offset: usize, func: SerFunc }` and `type SerFunc = unsafe fn(&mut SerStream, NonNull<()>) -> Result<(), ()>`. Deserialize is symmetric. Offsets come from `offset_of!`. Primitives are hand-written leaf functions. S1, ~16:59, Munns:
  > "we only generate a list of field offsets and function pointers for functions that know how to serialize or deserialize that data."
- **Per-field cost is two words of constant data.** ~21:03, Munns:
  > "the offset is a usize, so it's one word, four or eight bytes depending on what platform you are. A thin function pointer is one word."
  >
  > "It is a constant value that doesn't get used unless it gets used. It's not even code that has been generated at this point."
- **One interpreter walks the tables.** That is Munns's description; the repo code is slightly different (next bullet). ~25:45, Munns:
  > "we're generating data instead of code at compile time, and then we write one interpreter that knows how to turn that data into data on the wire."
- **What the repo actually does.** I checked the code (`lib.rs`, `postcard-forth-derive/src/ser.rs`):
  - The shared loop, `ser_fields_inner`, takes `(stream, base, fields)`. For each field it adds `offset` to a type-erased base pointer and calls `func`.
  - The derive fills each field's `func` with `ser_inliner::<FieldTy>()`. For a composite field type, this returns `ser_fields::<FieldTy>`, a generic `#[inline]` shim that calls `ser_fields_inner(…, T::FIELDS)`.
  - So **each composite type still gets a small monomorphized shim**, and the `#[inline]` loop may be duplicated into it.
  - Generic containers (`ser_vec<T>`, `ser_arr<T, N>`, `ser_option<T>`) are also monomorphized per element type.
  - The 3.1 numbers include these shims.
- **Flattening** is something he describes as a possibility, not a feature. ~21:50: "So maybe for that outer one, we don't even have that call […] We just splat the entire list into one list".
  - What the repo implements is narrower: `ser_inliner` and `deser_inliner` collapse only empty types and single-field, offset-0 types to their leaf function.
  - That is the "inlined" row in 3.1: 389,944 B against 395,712 B.
- **Deserialization is in place** into `MaybeUninit<T>`. ~34:25:
  > "It is built to be done in place. So there are never stack copies because everything is either done from the source or to the destination."
- **It can bound the stack explicitly.** ~27:22:
  > "It is an explicit stack machine. You can have a stack in there, and as you traverse down the stack, you can say, 'Ope, my max stack is 16,' and if you ever try to push a 17th item onto the stack, the stack machine just goes, 'Oops, out of stack! Serialization failed.'"

  The repo's `ser_fields_inner` is still recursive; a TODO in it asks about making it iterative.

**Limitations he named** (S1 §good-things-first onward, ~34:47–38:57):

- **Single format.** Asked how other formats would work, ~31:00: "I wouldn't." Then:
  > "this is that like cheating by doing less. I'm cheating by only supporting one format that I know the qualities of in and out."
- **It needs its own derived trait:**
  > "It's another crate that requires a derived trait for every type. Unless we get reflection in Rust, there's no way to get these offsets and map them to a pointer for every type."
- **Unsafe:**
  > "the engine itself for doing this serialization or deserialization is wildly unsafe"

  Manual implementations would be an `unsafe trait`: "a designed exploit in a box".
- **Some monomorphization remains** (~36:41):
  > "there might still be some recursion or function calls or some kind of monomorphization that's required to get you to that data type"

  This covers types that can't deref to a data-model type, such as `HashSet` and iterators.
- **Enums.** They need control flow (~38:20):
  > "For enums, you need branching. […] And we need loops, because you have slices and iters"

  In-place enum construction is blocked until there is a way to set the discriminant. He proposed one in [rust-lang/rfcs#3727](https://github.com/rust-lang/rfcs/pull/3727) (linked from S1's show notes).
- **Verdict** (~38:57):
  > "is it worth it? It is wildly unsafe. It is questionably implementable today. […] And really the answer is: I have no idea yet."

### 4.2 Stated direction: non-serde postcard

- **S6** (2024-09-25):
  > "I'm also looking into non-serde approaches for serialization and deserialization, as an optional alternative where speed, code size, or support for the postcard data format in other languages like C, C++, Python, or Typescript is important; while still being fully wire-compatible with the current Rust + serde version of postcard."
- **`postcard-core`** (S11): <https://github.com/jamesmunns/postcard/tree/main/source/postcard-core>, v0.1.0 on crates.io, introduced 2025-12-28 in PR #285.
  - PR #285: "contains the bulk of the ser/de logic, without relying on the `serde` crate."
  - It provides framework-agnostic primitive functions, such as `try_push_u32` and `try_take_discriminant`, against a `Flavor`.
  - In the repo, `postcard2` uses it as a serde front end. crates.io `postcard2` 0.2.1 was updated 2025-12-27, a day before PR #285 merged, so the published 0.2.1 may predate that change.
  - **No source I read ties `postcard-core` to code-size results, and it is not presented as postcard-forth's successor.** It is the decoupled wire layer that S6's stated direction would need.

### 4.3 Schemas and reflection (asymmetric device and host)

- **Crates:** `postcard-schema` 0.2.5 and `postcard-schema-ng` 0.4.1 (2026-10-21); `postcard-dyn` 0.2.1 and `postcard-dyn-ng` 0.4.1 (2026-11-06). All are in the postcard workspace and published on crates.io.
- **What they're for.** In S3, schemas are hashed into message keys, and the host can decode or transcode messages it doesn't know:
  > "I essentially had to invent reflection for Postcard messages so that you could get the schemas."
- **The argument is about device burden, not codec code size** (S3):
  > "our tiny embedded device, we don't want to burden it with the ability to send 15 different kinds of messages, but our desktop server doesn't care if it's got 57 flavors of decoding."

### 4.4 Related approaches that appear in the sources read

- **dtolnay, serde PR #2250 review (S9, 2023-08-06)**, which he linked when replying to S1 (S8):
  - On field matching: there "should be one single non-generic `DeserializeSeed` type that is used for deserializing all fields of all data structures". He estimates this "would reduce the number of instantiations of `Deserializer` methods by 50%".
  - Further: "`Deserialize` impls should be done by handing off a kind of type-erased bytecode for the deserializer to evaluate […] In this approach no control flow needs to be compiled individually per derived Deserialize impl, and no Deserializer methods need to be instantiated, so Deserialize impls become free — basically a single `static`."
  - On Bluesky (S8): "This is the ideal design for a trait-impl-based serialization library IMO."
  - Munns replied (S8): "Hah, beat me to the punch by almost a year :)". In S2 he also says dtolnay called it "the optimal way to do that".
- **merde, Amos's crate (S2):** v10.0.7 on crates.io, updated 2025-04-25.
  - It has its own traits and declarative macros instead of proc macros, and its goal is build speed.
  - Amos: "I'm gonna compromise on monomorphization, cause I want better build speeds. I'm gonna compromise on- I don't know, doing dynamic dispatch".
  - On runtime key mapping: "Something costs at runtime, but it's less code also in the binary."
  - His own caveat: "it's questionable whether it actually builds faster".
- **Pre-instantiating generics in upstream crates.** Amos raises this idea in S1 and S2 ("More research is needed").
- **facet, miniserde, erased-serde, musli:** not discussed in any source read. A grep of all saved transcripts and posts for these names finds nothing.

### 4.5 CBOR mentions (all incidental; none measure CBOR)

- **S3, Munns:**
  > "CBOR is like the binary version of JSON and it has all that same flexibility that JSON has, just in a much smaller form."

  He groups it with the self-describing key-value formats.
- **S4b** (2024-09-11): CoAP "use[s] CBOR instead of text for a lot of things".
- **S12 (2019 slides):** lists CBOR as a serde backend, and lists "code size" first under "resources are expensive". It contains no measurements.

---

## 5. Implications for a CBOR codec design

**[Direct]** means a cited quote or measurement supports the point. **[Inference]** means it is my extrapolation. Everything Munns measured was **postcard**: positional, not self-describing, one format. Any transfer to CBOR is inference unless marked otherwise.

1. **Make per-type cost static data, not code.** The derive should emit a const descriptor table: field offset plus a leaf handler or opcode. **[Direct]** See 2.4 and 4.1. Measured at 61.8% of serde `.text` overall, and 40.6% without enums (3.1).
   - Flattening nested tables into the parent is **[Inference]**. He described it only as a "maybe", and the repo doesn't do it.
2. **One shared, type-erased engine, not generic over the type.** Leaf handlers take `(&mut Stream, NonNull<()>)` against a concrete stream type.
   - The design intent is **[Direct, Munns's description]**: "doesn't get monomorphized for every single version", plus postcard-forth's `SerFunc` signature.
   - His prototype still has a small generic shim per composite type, `ser_fields::<T>` (4.1). The measured savings include those shims.
   - Removing the shims by passing table pointers instead of `fn` pointers is **[Inference]**: it should save more, but nobody has measured it.
   - Keeping the engine non-generic over the output buffer or "flavor" as well is **[Inference]**.
3. **Specialise to one wire format.** Don't route through a format-agnostic data model that every type must drive. **[Direct]** ("cheating by doing less… only supporting one format").
4. **Treat enums (tagged unions) as the hard case.** A table-only design still needed generated per-enum code, and that is where most of the remaining gap was. **[Direct]** (S5 quote; the no-enums rows in 3.1).
   - Encoding variant selection as a "discriminant → sub-table" opcode so enums stay data is **[Inference]**. It does follow his "you need branching… loops… a whole interpreter".
5. **Encode structs positionally:** CBOR arrays in declaration order, not maps with text keys. This avoids per-type key-matching code and key strings in flash. **[Inference]**
   - Munns never measured CBOR maps.
   - His S3 point that self-describing formats make "accessors get much larger in terms of code" is about **application-side handling** of optional or reordered fields, not codec size.
   - If maps are required, dtolnay's single non-generic field-matching routine over a static key table is the cited pattern. **[Direct, dtolnay (S9), not Munns]**
6. **Deserialize in place** into `MaybeUninit<T>` through destination pointers. This avoids by-value returns of large `heapless` containers; it saves stack and RAM, not flash. **[Direct]** (2.5; 4.1 in-place quote). In-place enums need RFC 3727 or a fallback. **[Direct]**
7. **Use a bounded, explicit nesting stack** instead of recursion, so maximum depth is known statically and malicious nesting fails cleanly. **[Direct]** ("my max stack is 16"). Note that his prototype still recurses (TODO in `lib.rs`).
8. **Keep the device codec minimal and put flexibility on the host:** schema-driven dynamic decode, transcoding, migration. **[Direct]** (S3 "tiny embedded device… desktop server", in the context of device burden).
9. **Costs to budget for.** The engine needs `unsafe` (`offset_of!` plus pointer casts), manual impls need an `unsafe trait`, and the format needs its own derive that third-party types must adopt. **[Direct]** (4.1 limitations)
   - A Miri-tested engine and a `cfg_attr`-gated derive to ease adoption are **[Inference]**.
10. **Measure the way he did.** Build no_std firmware that reads input from the host so nothing is optimized out. Subtract a no-ser/de baseline from `cargo size` `.text`. Use a fixed-seed generator of nested types, and keep enum-heavy and enum-free variants. **[Direct]** (S10 `results.md` steps; S1 ~32:09)
