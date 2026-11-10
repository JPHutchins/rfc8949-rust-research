"""Codec backends: how each crate spells message structs, enums and a decode-then-encode roundtrip."""

from collections.abc import Callable
from typing import NamedTuple

from model import EnumDef, Field, FieldKind, Message


class Backend(NamedTuple):
    name: str
    dependencies: str
    prelude: str
    struct_attrs: str
    field_attrs: Callable[[Field, int], str]
    enum_def: Callable[[EnumDef], str]
    impls: Callable[[Message], str]
    roundtrip: Callable[[Message], str]


def rust_type(field: Field) -> str:
    match (field.kind, field.target):
        case (FieldKind.NESTED, Message() as nested):
            return f"{nested.name}{nested.lifetime}"
        case (FieldKind.ENUM, EnumDef(name)):
            return name
        case (FieldKind.U32, _):
            return "u32"
        case (FieldKind.U64, _):
            return "u64"
        case (FieldKind.I32, _):
            return "i32"
        case (FieldKind.BOOL, _):
            return "bool"
        case (FieldKind.OPT_U32, _):
            return "Option<u32>"
        case (FieldKind.STR, _):
            return "&'a str"
        case (FieldKind.BYTES, _):
            return "&'a [u8]"
        case (FieldKind.NESTED | FieldKind.ENUM, _):
            raise ValueError(f"{field} lacks its target")


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


def serde_enum_def(enum: EnumDef) -> str:
    return f"{SERDE_DERIVE}\npub enum {enum.name} {{ A, B(u32), C {{ x: u32, y: bool }} }}\n"


def serde_map_field_attrs(field: Field, _position: int) -> str:
    match field.kind:
        case FieldKind.BYTES:
            return f'#[serde(rename = "{field.key}", borrow, with = "serde_bytes")] '
        case FieldKind.OPT_U32:
            return f'#[serde(rename = "{field.key}", default, skip_serializing_if = "Option::is_none")] '
        case FieldKind.NESTED if field.borrows:
            return f'#[serde(rename = "{field.key}", borrow)] '
        case FieldKind.U32 | FieldKind.U64 | FieldKind.I32 | FieldKind.BOOL | FieldKind.STR | FieldKind.NESTED | FieldKind.ENUM:
            return f'#[serde(rename = "{field.key}")] '


def serde_positional_field_attrs(field: Field, _position: int) -> str:
    match field.kind:
        case FieldKind.BYTES:
            return '#[serde(borrow, with = "serde_bytes")] '
        case FieldKind.NESTED if field.borrows:
            return "#[serde(borrow)] "
        case FieldKind.U32 | FieldKind.U64 | FieldKind.I32 | FieldKind.BOOL | FieldKind.OPT_U32 | FieldKind.STR | FieldKind.NESTED | FieldKind.ENUM:
            return ""


def serde_backend(name: str, dependency: str, prelude: str, decode: str, encode: str, field_attrs: Callable[[Field, int], str]) -> Backend:
    return Backend(
        name=name,
        dependencies=f"{dependency}\n{SERDE_DEPS}",
        prelude=prelude,
        struct_attrs=SERDE_DERIVE,
        field_attrs=field_attrs,
        enum_def=serde_enum_def,
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
        case FieldKind.NESTED if field.borrows:
            return f"#[b({position})] "
        case FieldKind.U32 | FieldKind.U64 | FieldKind.I32 | FieldKind.BOOL | FieldKind.OPT_U32 | FieldKind.NESTED | FieldKind.ENUM:
            return f"#[n({position})] "


def minicbor_derive_enum_def(enum: EnumDef) -> str:
    return (
        "#[derive(minicbor::Encode, minicbor::Decode)]\n"
        f"pub enum {enum.name} {{ #[n(0)] A, #[n(1)] B(#[n(0)] u32), #[n(2)] C {{ #[n(0)] x: u32, #[n(1)] y: bool }} }}\n"
    )


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
    enum_def=minicbor_derive_enum_def,
    impls=no_impls,
    roundtrip=MINICBOR_ROUNDTRIP,
)


def minicbor_scalar_method(kind: FieldKind) -> str:
    match kind:
        case FieldKind.U32 | FieldKind.OPT_U32:
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
        case FieldKind.NESTED | FieldKind.ENUM:
            raise ValueError(f"{kind} is not a scalar")


def minicbor_hand_encode_field(field: Field) -> str:
    match field.kind:
        case FieldKind.OPT_U32:
            return f'        if let Some(v) = self.{field.ident} {{ e.str("{field.key}")?.u32(v)?; }}\n'
        case FieldKind.NESTED | FieldKind.ENUM:
            return f'        e.str("{field.key}")?.encode(&self.{field.ident})?;\n'
        case FieldKind.U32 | FieldKind.U64 | FieldKind.I32 | FieldKind.BOOL | FieldKind.STR | FieldKind.BYTES:
            return f'        e.str("{field.key}")?.{minicbor_scalar_method(field.kind)}(self.{field.ident})?;\n'


def minicbor_hand_decode_expr(kind: FieldKind) -> str:
    match kind:
        case FieldKind.NESTED | FieldKind.ENUM:
            return "d.decode()?"
        case FieldKind.U32 | FieldKind.U64 | FieldKind.I32 | FieldKind.BOOL | FieldKind.OPT_U32 | FieldKind.STR | FieldKind.BYTES:
            return f"d.{minicbor_scalar_method(kind)}()?"


def minicbor_hand_finish_field(field: Field, position: int) -> str:
    match field.kind:
        case FieldKind.OPT_U32:
            return f"            {field.ident},\n"
        case FieldKind.U32 | FieldKind.U64 | FieldKind.I32 | FieldKind.BOOL | FieldKind.STR | FieldKind.BYTES | FieldKind.NESTED | FieldKind.ENUM:
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
        + "".join(f'                "{f.key}" => {f.ident} = Some({minicbor_hand_decode_expr(f.kind)}),\n' for f in msg.fields)
        + "                _ => d.skip()?,\n"
        "            }\n"
        "        }\n"
        f"        Ok({msg.name} {{\n"
        + "".join(minicbor_hand_finish_field(f, i) for i, f in enumerate(msg.fields))
        + "        })\n    }\n}\n"
    )


def minicbor_hand_enum_def(enum: EnumDef) -> str:
    name = enum.name
    return (
        f"pub enum {name} {{ A, B(u32), C {{ x: u32, y: bool }} }}\n\n"
        f"impl minicbor::Encode<()> for {name} {{\n"
        "    fn encode<W: minicbor::encode::Write>(&self, e: &mut minicbor::Encoder<W>, _: &mut ()) -> Result<(), minicbor::encode::Error<W::Error>> {\n"
        "        match self {\n"
        f'            {name}::A => {{ e.str("A")?; }}\n'
        f'            {name}::B(v) => {{ e.map(1)?.str("B")?.u32(*v)?; }}\n'
        f'            {name}::C {{ x, y }} => {{ e.map(1)?.str("C")?.map(2)?.str("x")?.u32(*x)?.str("y")?.bool(*y)?; }}\n'
        "        }\n"
        "        Ok(())\n    }\n}\n\n"
        f"impl<'b> minicbor::Decode<'b, ()> for {name} {{\n"
        "    fn decode(d: &mut minicbor::Decoder<'b>, _: &mut ()) -> Result<Self, minicbor::decode::Error> {\n"
        "        match d.datatype()? {\n"
        "            minicbor::data::Type::String => match d.str()? {\n"
        f'                "A" => Ok({name}::A),\n'
        '                _ => Err(minicbor::decode::Error::message("unknown variant")),\n'
        "            },\n"
        "            _ => {\n"
        "                d.map()?;\n"
        "                match d.str()? {\n"
        f'                    "B" => Ok({name}::B(d.u32()?)),\n'
        '                    "C" => {\n'
        "                        let mut x = None;\n"
        "                        let mut y = None;\n"
        '                        let n = d.map()?.ok_or_else(|| minicbor::decode::Error::message("indefinite map"))?;\n'
        "                        for _ in 0..n {\n"
        '                            match d.str()? { "x" => x = Some(d.u32()?), "y" => y = Some(d.bool()?), _ => d.skip()? }\n'
        "                        }\n"
        f"                        Ok({name}::C {{ x: x.ok_or_else(|| minicbor::decode::Error::missing_value(0))?, y: y.ok_or_else(|| minicbor::decode::Error::missing_value(1))? }})\n"
        "                    }\n"
        '                    _ => Err(minicbor::decode::Error::message("unknown variant")),\n'
        "                }\n"
        "            }\n"
        "        }\n"
        "    }\n}\n"
    )


MINICBOR_HAND_TEXTKEYS = Backend(
    name="minicbor-hand-textkeys",
    dependencies='minicbor = { version = "=2.3.1", default-features = false }',
    prelude="",
    struct_attrs="",
    field_attrs=lambda _field, _position: "",
    enum_def=minicbor_hand_enum_def,
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
