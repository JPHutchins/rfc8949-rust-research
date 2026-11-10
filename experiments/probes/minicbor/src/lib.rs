#![cfg_attr(not(test), no_std)]

use minicbor::decode::{self, Decoder};
use minicbor::encode::write::{Cursor, EndOfSlice};
use minicbor::encode::{self, Encoder, Write};
use minicbor::{Decode, Encode};

#[derive(Debug, PartialEq, Eq)]
pub struct Chunk<'a> {
    pub off: u32,
    pub data: &'a [u8],
    pub len: u32,
    pub sha: &'a [u8],
}

impl<C> Encode<C> for Chunk<'_> {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _: &mut C,
    ) -> Result<(), encode::Error<W::Error>> {
        e.map(4)?
            .str("off")?
            .u32(self.off)?
            .str("data")?
            .bytes(self.data)?
            .str("len")?
            .u32(self.len)?
            .str("sha")?
            .bytes(self.sha)?
            .ok()
    }
}

impl<'b, C> Decode<'b, C> for Chunk<'b> {
    fn decode(d: &mut Decoder<'b>, _: &mut C) -> Result<Self, decode::Error> {
        let entries = d
            .map()?
            .ok_or_else(|| decode::Error::message("indefinite map"))?;
        let (off, data, len, sha) =
            (0..entries).try_fold((None, None, None, None), |(off, data, len, sha), _| match d
                .str()?
            {
                "off" => Ok((Some(d.u32()?), data, len, sha)),
                "data" => Ok((off, Some(d.bytes()?), len, sha)),
                "len" => Ok((off, data, Some(d.u32()?), sha)),
                "sha" => Ok((off, data, len, Some(d.bytes()?))),
                _ => d.skip().map(|()| (off, data, len, sha)),
            })?;
        Ok(Chunk {
            off: off.ok_or_else(|| decode::Error::missing_value(0))?,
            data: data.ok_or_else(|| decode::Error::missing_value(1))?,
            len: len.ok_or_else(|| decode::Error::missing_value(2))?,
            sha: sha.ok_or_else(|| decode::Error::missing_value(3))?,
        })
    }
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

pub fn encode_chunk(chunk: &Chunk, buf: &mut [u8]) -> Result<usize, encode::Error<EndOfSlice>> {
    let mut cursor = Cursor::new(buf);
    minicbor::encode(chunk, &mut cursor)?;
    Ok(cursor.position())
}

pub fn decode_chunk(buf: &[u8]) -> Result<Chunk<'_>, decode::Error> {
    minicbor::decode(buf)
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
    fn decode_without_alloc_cannot_skip_nested_indefinite_unknown_value() {
        let input = b"\xa5\x61x\x82\x9f\x01\xff\x02\x63off\x00\x64data\x40\x63len\x00\x63sha\x40";
        assert!(decode_chunk(input).is_err());
    }

    #[test]
    fn encode_reports_short_buffer() {
        let mut buf = [0u8; 16];
        assert!(encode_chunk(&sample(), &mut buf).is_err());
    }
}
