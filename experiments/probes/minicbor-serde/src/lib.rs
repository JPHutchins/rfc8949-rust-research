#![cfg_attr(not(test), no_std)]

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Chunk<'a> {
    pub off: u32,
    #[serde(with = "serde_bytes")]
    pub data: &'a [u8],
    pub len: u32,
    #[serde(with = "serde_bytes")]
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

pub fn encode_chunk(
    chunk: &Chunk,
    buf: &mut [u8],
) -> Result<usize, minicbor_serde::error::EncodeError<minicbor::encode::write::EndOfSlice>> {
    minicbor_serde::to_slice(chunk, buf)
}

pub fn decode_chunk(buf: &[u8]) -> Result<Chunk<'_>, minicbor_serde::error::DecodeError> {
    minicbor_serde::from_slice(buf)
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
    fn duplicate_field_key_rejected_by_serde_derive() {
        let input = b"\xa5\x63off\x00\x63off\x01\x64data\x40\x63len\x00\x63sha\x40";
        assert!(decode_chunk(input).is_err());
    }

    #[test]
    fn encode_reports_short_buffer() {
        let mut buf = [0u8; 16];
        assert!(encode_chunk(&sample(), &mut buf).is_err());
    }
}
