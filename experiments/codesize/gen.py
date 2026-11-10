"""Generate, build and measure no_std thumbv7m binaries with N message types per codec backend."""

import argparse
import csv
import os
import random
import subprocess
import sys
from collections.abc import Sequence
from pathlib import Path
from typing import NamedTuple

from backends import BACKENDS, Backend, FieldKind


HERE = Path(__file__).resolve().parent
KEY_POOL = (
    "off len data sha upgrade image slot rc rsn version hash bootable pending confirmed active "
    "permanent name mtu count seq group id flags ts value status msg err buf size"
).split()
TARGET = "thumbv7m-none-eabi"


class Field(NamedTuple):
    key: str
    kind: FieldKind


class Message(NamedTuple):
    name: str
    fields: tuple[Field, ...]

    @property
    def borrows(self) -> bool:
        return any(field.kind in (FieldKind.STR, FieldKind.BYTES) for field in self.fields)


class Measurement(NamedTuple):
    backend: str
    profile: str
    message_count: int
    text: int
    rodata: int
    data: int

    @property
    def flash(self) -> int:
        return self.text + self.rodata + self.data


def message(index: int) -> Message:
    rng = random.Random(index)
    return Message(
        name=f"Msg{index}",
        fields=tuple(
            Field(key, rng.choice(list(FieldKind)))
            for key in rng.sample(KEY_POOL, rng.randint(3, 6))
        ),
    )


def render_message(backend: Backend, msg: Message) -> str:
    lifetime = "<'a>" if msg.borrows else ""
    fields = "\n".join(
        f"    {backend.field_attrs(field.kind, field.key, position)}pub f_{field.key}: {backend.rust_type(field.kind)},"
        for position, field in enumerate(msg.fields)
    )
    return f"{backend.struct_attrs}\npub struct {msg.name}{lifetime} {{\n{fields}\n}}\n"


def render_crate(backend: Backend, message_count: int) -> dict[str, str]:
    messages = tuple(map(message, range(message_count)))
    calls = "\n".join(
        f"    black_box(roundtrip_{m.name.lower()}(input, out));" for m in messages
    )
    main = (
        "#![no_std]\n#![no_main]\n#![allow(dead_code)]\n\n"
        "use core::hint::black_box;\n\n"
        f"{backend.prelude}\n"
        "#[panic_handler]\nfn panic(_: &core::panic::PanicInfo) -> ! {\n    loop {}\n}\n\n"
                + "\n".join(render_message(backend, m) + "\n" + backend.roundtrip(m.name) for m in messages)
        + "\n#[unsafe(no_mangle)]\npub extern \"C\" fn _start() -> ! {\n"
        "    let input: &[u8] = unsafe { core::slice::from_raw_parts(black_box(0x2000_0000usize) as *const u8, black_box(256)) };\n"
        "    let out: &mut [u8] = unsafe { core::slice::from_raw_parts_mut(black_box(0x2000_1000usize) as *mut u8, black_box(256)) };\n"
        f"{calls}\n"
        "    loop {}\n}\n"
    )
    cargo = (
        f'[package]\nname = "cs-{backend.name}-n{message_count}"\nversion = "0.0.0"\nedition = "2024"\npublish = false\n\n'
        f"[dependencies]\n{backend.dependencies}\n\n"
        "[profile.release]\nopt-level = \"z\"\nlto = \"fat\"\ncodegen-units = 1\npanic = \"abort\"\ndebug = false\n\n"
        "[profile.release-s]\ninherits = \"release\"\nopt-level = \"s\"\n\n"
        "[workspace]\n"
    )
    return {"Cargo.toml": cargo, "src/main.rs": main}


def write_crate(root: Path, files: dict[str, str]) -> Path:
    for relative, content in files.items():
        (root / relative).parent.mkdir(parents=True, exist_ok=True)
        (root / relative).write_text(content)
    return root


def section_sizes(elf: Path) -> dict[str, int]:
    return {
        parts[0]: int(parts[1])
        for parts in (line.split() for line in subprocess.run(
            ["size", "-A", "-d", str(elf)], check=True, capture_output=True, text=True
        ).stdout.splitlines())
        if len(parts) >= 2 and parts[0].startswith(".") and parts[1].isdigit()
    }


def build_and_measure(backend: Backend, message_count: int, profile: str) -> Measurement:
    crate = write_crate(HERE / "build" / f"{backend.name}-n{message_count}", render_crate(backend, message_count))
    subprocess.run(
        ["cargo", "build", "--quiet", "--target", TARGET, "--profile", profile],
        cwd=crate,
        check=True,
        env={**os.environ, "CARGO_TARGET_DIR": str(HERE / "target")},
    )
    profile_dir = "release" if profile == "release" else profile
    sizes = section_sizes(HERE / "target" / TARGET / profile_dir / f"cs-{backend.name}-n{message_count}")
    return Measurement(
        backend=backend.name,
        profile=profile,
        message_count=message_count,
        text=sizes.get(".text", 0),
        rodata=sizes.get(".rodata", 0),
        data=sizes.get(".data", 0),
    )


def main(argv: Sequence[str]) -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--backends", nargs="+", default=list(BACKENDS))
    parser.add_argument("--counts", nargs="+", type=int, default=[0, 1, 2, 4, 8, 16, 32])
    parser.add_argument("--profiles", nargs="+", default=["release", "release-s"])
    parser.add_argument("--out", type=Path, default=HERE / "results.csv")
    args = parser.parse_args(argv)
    measurements = [
        build_and_measure(BACKENDS[name], count, profile)
        for name in args.backends
        for profile in args.profiles
        for count in args.counts
    ]
    with args.out.open("w", newline="") as handle:
        writer = csv.writer(handle)
        writer.writerow([*Measurement._fields, "flash"])
        writer.writerows([*m, m.flash] for m in measurements)
    for m in measurements:
        print(f"{m.backend:22} {m.profile:10} n={m.message_count:<3} text={m.text:>7} rodata={m.rodata:>6} flash={m.flash:>7}")


if __name__ == "__main__":
    main(sys.argv[1:])
