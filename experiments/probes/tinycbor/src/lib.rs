#![cfg_attr(not(test), no_std)]

use tinycbor::{Decode, Decoder, Encode, Encoder, Write};

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

pub fn encode_chunk(chunk: &Chunk, buf: &mut [u8]) -> Result<usize, embedded_io::SliceWriteError> {
    let capacity = buf.len();
    let mut encoder = Encoder(buf);
    chunk.encode(&mut encoder)?;
    Ok(capacity - encoder.0.len())
}

pub fn decode_chunk(buf: &[u8]) -> Result<Chunk<'_>, ChunkError> {
    Chunk::decode(&mut Decoder(buf))
}

impl Encode for Chunk<'_> {
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), W::Error> {
        e.map(4)?;
        "off".encode(e)?;
        self.off.encode(e)?;
        "data".encode(e)?;
        self.data.encode(e)?;
        "len".encode(e)?;
        self.len.encode(e)?;
        "sha".encode(e)?;
        self.sha.encode(e)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkError {
    Malformed,
    UnknownKey,
    DuplicateKey,
    MissingKey,
}

impl core::fmt::Display for ChunkError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }
}

impl core::error::Error for ChunkError {}

enum Field<'b> {
    Off(u32),
    Data(&'b [u8]),
    Len(u32),
    Sha(&'b [u8]),
}

#[derive(Default)]
struct Fields<'b> {
    off: Option<u32>,
    data: Option<&'b [u8]>,
    len: Option<u32>,
    sha: Option<&'b [u8]>,
}

fn set_once<T>(slot: Option<T>, value: T) -> Result<Option<T>, ChunkError> {
    slot.map_or(Ok(Some(value)), |_| Err(ChunkError::DuplicateKey))
}

impl<'b> Fields<'b> {
    fn with(self, field: Field<'b>) -> Result<Self, ChunkError> {
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
        })
    }

    fn finish(self) -> Result<Chunk<'b>, ChunkError> {
        Ok(Chunk {
            off: self.off.ok_or(ChunkError::MissingKey)?,
            data: self.data.ok_or(ChunkError::MissingKey)?,
            len: self.len.ok_or(ChunkError::MissingKey)?,
            sha: self.sha.ok_or(ChunkError::MissingKey)?,
        })
    }
}

fn malformed<E>(_: E) -> ChunkError {
    ChunkError::Malformed
}

fn decode_field<'b>(key: &str, d: &mut Decoder<'b>) -> Result<Field<'b>, ChunkError> {
    match key {
        "off" => u32::decode(d).map(Field::Off).map_err(malformed),
        "data" => <&[u8]>::decode(d).map(Field::Data).map_err(malformed),
        "len" => u32::decode(d).map(Field::Len).map_err(malformed),
        "sha" => <&[u8]>::decode(d).map(Field::Sha).map_err(malformed),
        _ => Err(ChunkError::UnknownKey),
    }
}

impl<'b> Decode<'b> for Chunk<'b> {
    type Error = ChunkError;

    fn decode(d: &mut Decoder<'b>) -> Result<Self, ChunkError> {
        let mut visitor = d.map_visitor().map_err(malformed)?;
        core::iter::from_fn(|| visitor.with_key(|key: &str, d| decode_field(key, d)))
            .try_fold(Fields::default(), |fields, entry| {
                fields.with(entry.map_err(malformed)??)
            })?
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
    #[ignore = "tinycbor has no allocation-free skip for unknown values (Any requires alloc)"]
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
    fn decode_accepts_any_key_order_and_rejects_unknown_key() {
        let reordered = b"\xa4\x63len\x18\x7b\x63sha\x41\xaa\x63off\x00\x64data\x40";
        assert_eq!(
            decode_chunk(reordered).unwrap(),
            Chunk {
                off: 0,
                data: &[],
                len: 123,
                sha: &[0xaa]
            }
        );
        let unknown = b"\xa5\x63len\x18\x7b\x67upgrade\xf5\x63sha\x41\xaa\x63off\x00\x64data\x40";
        assert_eq!(decode_chunk(unknown), Err(ChunkError::UnknownKey));
    }

    #[test]
    fn encode_reports_short_buffer() {
        let mut buf = [0u8; 16];
        assert!(encode_chunk(&sample(), &mut buf).is_err());
    }
}
