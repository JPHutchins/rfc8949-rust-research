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


class Field(NamedTuple):
    key: str
    kind: FieldKind

    @property
    def ident(self) -> str:
        return f"f_{self.key}"


class Message(NamedTuple):
    name: str
    fields: tuple[Field, ...]

    @property
    def borrows(self) -> bool:
        return any(field.kind in (FieldKind.STR, FieldKind.BYTES) for field in self.fields)

    @property
    def lifetime(self) -> str:
        return "<'a>" if self.borrows else ""

    @property
    def fn_name(self) -> str:
        return f"roundtrip_{self.name.lower()}"
