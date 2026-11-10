"""Codec backends: how each crate spells a message struct and a decode-then-encode roundtrip."""

from collections.abc import Callable
from typing import NamedTuple

from model import Field, FieldKind, Message


class Backend(NamedTuple):
    name: str
    dependencies: str
    prelude: str
    struct_attrs: str
    field_attrs: Callable[[Field, int], str]
    impls: Callable[[Message], str]
    roundtrip: Callable[[Message], str]


def rust_type(kind: FieldKind) -> str:
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


def no_impls(_msg: Message) -> str:
    return ""


def roundtrip_fn(decode: str, encode: str) -> Callable[[Message], str]:
    return lambda msg: (
        f"fn {msg.fn_name}(input: &[u8], out: &mut [u8]) -> usize {{\n"
        f"    {decode.format(T=msg.name)}\n"
        "        .ok()\n"
        f"        .and_then(|m| {encode})\n"
        "        .unwrap_or(0)\n"
        "}\n"
    )


SERDE_DEPS = (
    'serde = { version = "1", default-features = false, features = ["derive"] }\n'
    'serde_bytes = { version = "0.11", default-features = false }'
)
SERDE_DERIVE = "#[derive(serde::Serialize, serde::Deserialize)]"


def serde_map_field_attrs(field: Field, _position: int) -> str:
    match field.kind:
        case FieldKind.BYTES:
            return f'#[serde(rename = "{field.key}", borrow, with = "serde_bytes")] '
        case FieldKind.OPT_U32:
            return f'#[serde(rename = "{field.key}", default, skip_serializing_if = "Option::is_none")] '
        case FieldKind.U32 | FieldKind.U64 | FieldKind.I32 | FieldKind.BOOL | FieldKind.STR:
            return f'#[serde(rename = "{field.key}")] '


def serde_positional_field_attrs(field: Field, _position: int) -> str:
    match field.kind:
        case FieldKind.BYTES:
            return '#[serde(borrow, with = "serde_bytes")] '
        case FieldKind.U32 | FieldKind.U64 | FieldKind.I32 | FieldKind.BOOL | FieldKind.OPT_U32 | FieldKind.STR:
            return ""


def serde_backend(name: str, dependency: str, prelude: str, decode: str, encode: str, field_attrs: Callable[[Field, int], str]) -> Backend:
    return Backend(
        name=name,
        dependencies=f"{dependency}\n{SERDE_DEPS}",
        prelude=prelude,
        struct_attrs=SERDE_DERIVE,
        field_attrs=field_attrs,
        impls=no_impls,
        roundtrip=roundtrip_fn(decode, encode),
    )


POSTCARD = serde_backend(
    name="postcard",
    dependency='postcard = { version = "=1.1.3", default-features = false }',
    prelude="",
    decode="postcard::from_bytes::<{T}>(input)",
    encode="postcard::to_slice(&m, out).ok().map(|s| s.len())",
    field_attrs=serde_positional_field_attrs,
)

SERDE_CBOR = serde_backend(
    name="serde_cbor",
    dependency='serde_cbor = { version = "=0.11.2", default-features = false }',
    prelude="use serde::Serialize;\n",
    decode="serde_cbor::de::from_slice_with_scratch::<{T}>(input, &mut [])",
    encode="{ let mut s = serde_cbor::Serializer::new(serde_cbor::ser::SliceWrite::new(out)); m.serialize(&mut s).ok().map(|()| s.into_inner().bytes_written()) }",
    field_attrs=serde_map_field_attrs,
)

SERDE_CBOR_2 = serde_backend(
    name="serde_cbor_2",
    dependency='serde_cbor_2 = { version = "=0.13.0", default-features = false }',
    prelude="use serde::Serialize;\n",
    decode="serde_cbor_2::de::from_slice_with_scratch::<{T}>(input, &mut [])",
    encode="{ let mut s = serde_cbor_2::Serializer::new(serde_cbor_2::ser::SliceWrite::new(out)); m.serialize(&mut s).ok().map(|()| s.into_inner().bytes_written()) }",
    field_attrs=serde_map_field_attrs,
)

CBOR_SMOL = serde_backend(
    name="cbor-smol",
    dependency='cbor-smol = { version = "=0.5.1", default-features = false }',
    prelude="",
    decode="cbor_smol::cbor_deserialize::<{T}>(input)",
    encode="cbor_smol::cbor_serialize_to(&m, out).ok()",
    field_attrs=serde_map_field_attrs,
)

MINICBOR_SERDE = serde_backend(
    name="minicbor-serde",
    dependency='minicbor = { version = "=2.3.1", default-features = false }\nminicbor-serde = { version = "=0.7.2", default-features = false }',
    prelude="",
    decode="minicbor_serde::from_slice::<{T}>(input)",
    encode="minicbor_serde::to_slice(&m, out).ok()",
    field_attrs=serde_map_field_attrs,
)


def minicbor_derive_field_attrs(field: Field, position: int) -> str:
    match field.kind:
        case FieldKind.BYTES:
            return f'#[cbor(b({position}), with = "minicbor::bytes")] '
        case FieldKind.STR:
            return f"#[b({position})] "
        case FieldKind.U32 | FieldKind.U64 | FieldKind.I32 | FieldKind.BOOL | FieldKind.OPT_U32:
            return f"#[n({position})] "


MINICBOR_ROUNDTRIP = roundtrip_fn(
    decode="minicbor::decode::<{T}>(input)",
    encode="{ let mut c = minicbor::encode::write::Cursor::new(out); minicbor::encode(&m, &mut c).ok().map(|()| c.position()) }",
)

MINICBOR_DERIVE_INTKEYS = Backend(
    name="minicbor-derive-intkeys",
    dependencies='minicbor = { version = "=2.3.1", default-features = false, features = ["derive"] }',
    prelude="",
    struct_attrs="#[derive(minicbor::Encode, minicbor::Decode)]\n#[cbor(map)]",
    field_attrs=minicbor_derive_field_attrs,
    impls=no_impls,
    roundtrip=MINICBOR_ROUNDTRIP,
)


def minicbor_encode_call(kind: FieldKind) -> str:
    match kind:
        case FieldKind.U32:
            return "u32"
        case FieldKind.U64:
            return "u64"
        case FieldKind.I32:
            return "i32"
        case FieldKind.BOOL:
            return "bool"
        case FieldKind.STR:
            return "str"
        case FieldKind.BYTES:
            return "bytes"
        case FieldKind.OPT_U32:
            return "u32"


def minicbor_hand_encode_field(field: Field) -> str:
    match field.kind:
        case FieldKind.OPT_U32:
            return f'        if let Some(v) = self.{field.ident} {{ e.str("{field.key}")?.u32(v)?; }}\n'
        case FieldKind.U32 | FieldKind.U64 | FieldKind.I32 | FieldKind.BOOL | FieldKind.STR | FieldKind.BYTES:
            return f'        e.str("{field.key}")?.{minicbor_encode_call(field.kind)}(self.{field.ident})?;\n'


def minicbor_hand_finish_field(field: Field, position: int) -> str:
    match field.kind:
        case FieldKind.OPT_U32:
            return f"            {field.ident},\n"
        case FieldKind.U32 | FieldKind.U64 | FieldKind.I32 | FieldKind.BOOL | FieldKind.STR | FieldKind.BYTES:
            return f"            {field.ident}: {field.ident}.ok_or_else(|| minicbor::decode::Error::missing_value({position}))?,\n"


def minicbor_hand_impls(msg: Message) -> str:
    impl_lifetime = "<'a>" if msg.borrows else ""
    decode_lifetime = "'a" if msg.borrows else "'b"
    required = sum(field.kind != FieldKind.OPT_U32 for field in msg.fields)
    optional_len = "".join(f" + self.{f.ident}.is_some() as u64" for f in msg.fields if f.kind == FieldKind.OPT_U32)
    return (
        f"impl{impl_lifetime} minicbor::Encode<()> for {msg.name}{msg.lifetime} {{\n"
        "    fn encode<W: minicbor::encode::Write>(&self, e: &mut minicbor::Encoder<W>, _: &mut ()) -> Result<(), minicbor::encode::Error<W::Error>> {\n"
        f"        e.map({required}{optional_len})?;\n"
        + "".join(map(minicbor_hand_encode_field, msg.fields))
        + "        Ok(())\n    }\n}\n\n"
        f"impl<{decode_lifetime}> minicbor::Decode<{decode_lifetime}, ()> for {msg.name}{msg.lifetime} {{\n"
        f"    fn decode(d: &mut minicbor::Decoder<{decode_lifetime}>, _: &mut ()) -> Result<Self, minicbor::decode::Error> {{\n"
        + "".join(f"        let mut {f.ident} = None;\n" for f in msg.fields)
        + '        let entries = d.map()?.ok_or_else(|| minicbor::decode::Error::message("indefinite map"))?;\n'
        "        for _ in 0..entries {\n"
        "            match d.str()? {\n"
        + "".join(f'                "{f.key}" => {f.ident} = Some(d.{minicbor_encode_call(f.kind)}()?),\n' for f in msg.fields)
        + "                _ => d.skip()?,\n"
        "            }\n"
        "        }\n"
        f"        Ok({msg.name} {{\n"
        + "".join(minicbor_hand_finish_field(f, i) for i, f in enumerate(msg.fields))
        + "        })\n    }\n}\n"
    )


MINICBOR_HAND_TEXTKEYS = Backend(
    name="minicbor-hand-textkeys",
    dependencies='minicbor = { version = "=2.3.1", default-features = false }',
    prelude="",
    struct_attrs="",
    field_attrs=lambda _field, _position: "",
    impls=minicbor_hand_impls,
    roundtrip=MINICBOR_ROUNDTRIP,
)


BACKENDS: dict[str, Backend] = {
    backend.name: backend
    for backend in (
        POSTCARD,
        SERDE_CBOR,
        SERDE_CBOR_2,
        CBOR_SMOL,
        MINICBOR_SERDE,
        MINICBOR_DERIVE_INTKEYS,
        MINICBOR_HAND_TEXTKEYS,
    )
}
