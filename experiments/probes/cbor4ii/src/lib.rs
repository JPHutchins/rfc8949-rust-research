#![cfg_attr(not(test), no_std)]

use cbor4ii::core::dec::{self, Decode, IgnoredAny, Read};
use cbor4ii::core::enc::{self, Encode, Write};
use cbor4ii::core::types::{Bytes, Map};
use cbor4ii::core::utils::SliceReader;

#[derive(Debug, PartialEq, Eq)]
pub struct Chunk<'a> {
    pub off: u32,
    pub data: &'a [u8],
    pub len: u32,
    pub sha: &'a [u8],
}

pub const SHA: [u8; 32] = [0xaa; 32];

pub fn sample() -> Chunk<'static> {
    Chunk {
        off: 0,
        data: &[1, 2, 3, 4],
        len: 123,
        sha: &SHA,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BufferFull;

impl core::fmt::Display for BufferFull {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("buffer full")
    }
}

impl core::error::Error for BufferFull {}

pub struct SliceWriter<'a> {
    remaining: &'a mut [u8],
}

impl Write for SliceWriter<'_> {
    type Error = BufferFull;

    fn push(&mut self, input: &[u8]) -> Result<(), BufferFull> {
        let (head, tail) = core::mem::take(&mut self.remaining)
            .split_at_mut_checked(input.len())
            .ok_or(BufferFull)?;
        head.copy_from_slice(input);
        self.remaining = tail;
        Ok(())
    }
}

pub fn encode_chunk(chunk: &Chunk, buf: &mut [u8]) -> Result<usize, enc::Error<BufferFull>> {
    let capacity = buf.len();
    let mut writer = SliceWriter { remaining: buf };
    chunk.encode(&mut writer)?;
    Ok(capacity - writer.remaining.len())
}

pub fn decode_chunk(buf: &[u8]) -> Result<Chunk<'_>, dec::Error<core::convert::Infallible>> {
    Chunk::decode(&mut SliceReader::new(buf))
}

impl Encode for Chunk<'_> {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), enc::Error<W::Error>> {
        Map::bounded(4, writer)?;
        "off".encode(writer)?;
        self.off.encode(writer)?;
        "data".encode(writer)?;
        Bytes(self.data).encode(writer)?;
        "len".encode(writer)?;
        self.len.encode(writer)?;
        "sha".encode(writer)?;
        Bytes(self.sha).encode(writer)
    }
}

pub const DUPLICATE_KEY: u32 = 1;
pub const MISSING_KEY: u32 = 2;

fn chunk_error<E>(num: u32) -> dec::Error<E> {
    dec::Error::Custom {
        name: &"chunk",
        num,
    }
}

enum Field<'b> {
    Off(u32),
    Data(&'b [u8]),
    Len(u32),
    Sha(&'b [u8]),
    Unknown,
}

#[derive(Default)]
struct Fields<'b> {
    off: Option<u32>,
    data: Option<&'b [u8]>,
    len: Option<u32>,
    sha: Option<&'b [u8]>,
}

fn set_once<T, E>(slot: Option<T>, value: T) -> Result<Option<T>, dec::Error<E>> {
    slot.map_or(Ok(Some(value)), |_| Err(chunk_error(DUPLICATE_KEY)))
}

impl<'b> Fields<'b> {
    fn with<E>(self, field: Field<'b>) -> Result<Self, dec::Error<E>> {
        Ok(match field {
            Field::Off(v) => Self {
                off: set_once(self.off, v)?,
                ..self
            },
            Field::Data(v) => Self {
                data: set_once(self.data, v)?,
                ..self
            },
            Field::Len(v) => Self {
                len: set_once(self.len, v)?,
                ..self
            },
            Field::Sha(v) => Self {
                sha: set_once(self.sha, v)?,
                ..self
            },
            Field::Unknown => self,
        })
    }

    fn finish<E>(self) -> Result<Chunk<'b>, dec::Error<E>> {
        let missing = || chunk_error(MISSING_KEY);
        Ok(Chunk {
            off: self.off.ok_or_else(missing)?,
            data: self.data.ok_or_else(missing)?,
            len: self.len.ok_or_else(missing)?,
            sha: self.sha.ok_or_else(missing)?,
        })
    }
}

fn decode_field<'b, R: Read<'b>>(reader: &mut R) -> Result<Field<'b>, dec::Error<R::Error>> {
    Ok(match <&'b str>::decode(reader)? {
        "off" => Field::Off(u32::decode(reader)?),
        "data" => Field::Data(<Bytes<&'b [u8]>>::decode(reader)?.0),
        "len" => Field::Len(u32::decode(reader)?),
        "sha" => Field::Sha(<Bytes<&'b [u8]>>::decode(reader)?.0),
        _ => IgnoredAny::decode(reader).map(|_| Field::Unknown)?,
    })
}

fn next_field<'b, R: Read<'b>>(
    reader: &mut R,
    remaining: &mut Option<usize>,
) -> Option<Result<Field<'b>, dec::Error<R::Error>>> {
    match *remaining {
        Some(0) => None,
        Some(n) => {
            *remaining = Some(n - 1);
            Some(decode_field(reader))
        }
        None => match dec::is_break(reader) {
            Ok(true) => None,
            Ok(false) => Some(decode_field(reader)),
            Err(e) => Some(Err(e)),
        },
    }
}

impl<'b> Decode<'b> for Chunk<'b> {
    fn decode<R: Read<'b>>(reader: &mut R) -> Result<Self, dec::Error<R::Error>> {
        let mut remaining = Map::len(reader)?;
        core::iter::from_fn(|| next_field(reader, &mut remaining))
            .try_fold(Fields::default(), |fields, field| fields.with(field?))?
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static EXPECTED: [u8; 60] = {
        let head = *b"\xa4\x63off\x00\x64data\x44\x01\x02\x03\x04\x63len\x18\x7b\x63sha\x58\x20";
        let mut out = [0xaa; 60];
        let mut i = 0;
        while i < head.len() {
            out[i] = head[i];
            i += 1;
        }
        out
    };

    #[test]
    fn encodes_text_keyed_map_into_slice() {
        let mut buf = [0u8; 128];
        let written = encode_chunk(&sample(), &mut buf).unwrap();
        assert_eq!(&buf[..written], &EXPECTED[..]);
    }

    #[test]
    fn decodes_borrowing_from_input() {
        let decoded = decode_chunk(&EXPECTED).unwrap();
        assert_eq!(decoded, sample());
        assert!(core::ptr::eq(decoded.sha.as_ptr(), EXPECTED[28..].as_ptr()));
    }

    #[test]
    fn decode_ignores_unknown_key_and_accepts_any_key_order() {
        let input = b"\xa5\x63len\x18\x7b\x67upgrade\xf5\x63sha\x41\xaa\x63off\x00\x64data\x40";
        let decoded = decode_chunk(input).unwrap();
        assert_eq!(
            decoded,
            Chunk {
                off: 0,
                data: &[],
                len: 123,
                sha: &[0xaa]
            }
        );
    }

    #[test]
    fn encode_reports_short_buffer() {
        let mut buf = [0u8; 16];
        assert!(encode_chunk(&sample(), &mut buf).is_err());
    }
}
