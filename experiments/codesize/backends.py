"""Codec backends: how each crate spells a text-keyed message struct and a decode-then-encode roundtrip."""

from collections.abc import Callable
from enum import Enum, auto
from typing import NamedTuple


class FieldKind(Enum):
    U32 = auto()
    U64 = auto()
    I32 = auto()
    BOOL = auto()
    OPT_U32 = auto()
    STR = auto()
    BYTES = auto()


class Backend(NamedTuple):
    name: str
    dependencies: str
    prelude: str
    struct_attrs: str
    field_attrs: Callable[[FieldKind, str, int], str]
    rust_type: Callable[[FieldKind], str]
    roundtrip: Callable[[str], str]


def plain_rust_type(kind: FieldKind) -> str:
    match kind:
        case FieldKind.U32:
            return "u32"
        case FieldKind.U64:
            return "u64"
        case FieldKind.I32:
            return "i32"
        case FieldKind.BOOL:
            return "bool"
        case FieldKind.OPT_U32:
            return "Option<u32>"
        case FieldKind.STR:
            return "&'a str"
        case FieldKind.BYTES:
            return "&'a [u8]"


SERDE_DEPS = (
    'serde = { version = "1", default-features = false, features = ["derive"] }\n'
    'serde_bytes = { version = "0.11", default-features = false }'
)
SERDE_DERIVE = "#[derive(serde::Serialize, serde::Deserialize)]"


def serde_map_field_attrs(kind: FieldKind, key: str, _position: int) -> str:
    match kind:
        case FieldKind.BYTES:
            return f'#[serde(rename = "{key}", borrow, with = "serde_bytes")] '
        case FieldKind.OPT_U32:
            return f'#[serde(rename = "{key}", default, skip_serializing_if = "Option::is_none")] '
        case FieldKind.U32 | FieldKind.U64 | FieldKind.I32 | FieldKind.BOOL | FieldKind.STR:
            return f'#[serde(rename = "{key}")] '


def serde_positional_field_attrs(kind: FieldKind, _key: str, _position: int) -> str:
    match kind:
        case FieldKind.BYTES:
            return '#[serde(borrow, with = "serde_bytes")] '
        case FieldKind.U32 | FieldKind.U64 | FieldKind.I32 | FieldKind.BOOL | FieldKind.OPT_U32 | FieldKind.STR:
            return ""


def postcard_roundtrip(name: str) -> str:
    return (
        f"fn roundtrip_{name.lower()}(input: &[u8], out: &mut [u8]) -> usize {{\n"
        f"    postcard::from_bytes::<{name}>(input)\n"
        "        .ok()\n"
        "        .and_then(|m| postcard::to_slice(&m, out).ok().map(|s| s.len()))\n"
        "        .unwrap_or(0)\n"
        "}\n"
    )


POSTCARD = Backend(
    name="postcard",
    dependencies=f'postcard = {{ version = "1.1", default-features = false }}\n{SERDE_DEPS}',
    prelude="",
    struct_attrs=SERDE_DERIVE,
    field_attrs=serde_positional_field_attrs,
    rust_type=plain_rust_type,
    roundtrip=postcard_roundtrip,
)


BACKENDS: dict[str, Backend] = {backend.name: backend for backend in (POSTCARD,)}
