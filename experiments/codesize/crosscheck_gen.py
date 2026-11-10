"""Emit host-side modules with the rich-shape types for several backends, so their wire behaviour can be cross-checked."""

from pathlib import Path

from backends import BACKENDS
from gen import messages, render_message, SHAPES

MODULES = {"serde_side": "minicbor-serde", "hand_side": "minicbor-hand-textkeys", "smol_side": "cbor-smol"}
COUNT = 6


def module(name: str, backend_name: str) -> str:
    backend = BACKENDS[backend_name]
    body = "\n".join(render_message(backend, m) + "\n" + backend.impls(m) for m in messages(COUNT, SHAPES["rich"]))
    return f"#[allow(dead_code)]\npub mod {name} {{\n{body}\n}}\n"


if __name__ == "__main__":
    Path(__file__).resolve().parent.joinpath("crosscheck", "src", "lib.rs").write_text(
        "".join(module(name, backend) for name, backend in MODULES.items())
    )
