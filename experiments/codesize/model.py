"""Message shapes shared by the generator and the codec backends."""

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
    NESTED = auto()
    ENUM = auto()


FLAT_KINDS = (
    FieldKind.U32,
    FieldKind.U64,
    FieldKind.I32,
    FieldKind.BOOL,
    FieldKind.OPT_U32,
    FieldKind.STR,
    FieldKind.BYTES,
)
RICH_KINDS = (*FLAT_KINDS, FieldKind.NESTED, FieldKind.ENUM)


class EnumDef(NamedTuple):
    """Always `{ A, B(u32), C { x: u32, y: bool } }`: one unit, one newtype and one struct variant."""

    name: str


class Field(NamedTuple):
    key: str
    kind: FieldKind
    target: "Message | EnumDef | None" = None

    @property
    def ident(self) -> str:
        return f"f_{self.key}"

    @property
    def borrows(self) -> bool:
        match self.target:
            case Message() as nested:
                return nested.borrows
            case EnumDef() | None:
                return self.kind in (FieldKind.STR, FieldKind.BYTES)


class Message(NamedTuple):
    name: str
    fields: tuple[Field, ...]

    @property
    def borrows(self) -> bool:
        return any(field.borrows for field in self.fields)

    @property
    def lifetime(self) -> str:
        return "<'a>" if self.borrows else ""

    @property
    def fn_name(self) -> str:
        return f"roundtrip_{self.name.lower()}"

    @property
    def enums(self) -> tuple[EnumDef, ...]:
        return tuple(field.target for field in self.fields if isinstance(field.target, EnumDef))
