#![cfg_attr(not(test), no_std)]

use ciborium_io::Read;
use ciborium_ll::{Decoder, Encoder, Header};

#[derive(Debug, PartialEq, Eq)]
pub struct Chunk<'a> {
    pub off: u32,
    pub data: &'a [u8],
    pub len: u32,
    pub sha: &'a [u8],
}

#[derive(Debug, PartialEq, Eq)]
pub enum ProbeError {
    Io,
    Syntax,
    UnexpectedItem,
    MissingField,
    ScratchTooSmall,
}

impl<T> From<ciborium_ll::Error<T>> for ProbeError {
    fn from(error: ciborium_ll::Error<T>) -> Self {
        match error {
            ciborium_ll::Error::Io(_) => ProbeError::Io,
            ciborium_ll::Error::Syntax(_) => ProbeError::Syntax,
        }
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

pub fn encode_chunk(chunk: &Chunk, buf: &mut [u8]) -> Result<usize, ProbeError> {
    let capacity = buf.len();
    let mut remaining = buf;
    let mut encoder = Encoder::from(&mut remaining);
    [
        encoder.push(Header::Map(Some(4))),
        encoder.text("off", None),
        encoder.push(Header::Positive(chunk.off.into())),
        encoder.text("data", None),
        encoder.bytes(chunk.data, None),
        encoder.text("len", None),
        encoder.push(Header::Positive(chunk.len.into())),
        encoder.text("sha", None),
        encoder.bytes(chunk.sha, None),
    ]
    .into_iter()
    .collect::<Result<(), _>>()
    .map_err(|_| ProbeError::Io)?;
    Ok(capacity - remaining.len())
}

pub fn decode_chunk<'s>(buf: &[u8], scratch: &'s mut [u8]) -> Result<Chunk<'s>, ProbeError> {
    let mut decoder = Decoder::from(buf);
    let entries = match decoder.pull()? {
        Header::Map(Some(entries)) => Ok(entries),
        _ => Err(ProbeError::UnexpectedItem),
    }?;
    let (off, data, len, sha, _) = (0..entries).try_fold(
        (None, None, None, None, scratch),
        |(off, data, len, sha, free), _| {
            let mut key = [0u8; 8];
            match read_key(&mut decoder, &mut key)? {
                b"off" => Ok((Some(read_u32(&mut decoder)?), data, len, sha, free)),
                b"len" => Ok((off, data, Some(read_u32(&mut decoder)?), sha, free)),
                b"data" => read_bytes(&mut decoder, free)
                    .map(|(value, rest)| (off, Some(value), len, sha, rest)),
                b"sha" => read_bytes(&mut decoder, free)
                    .map(|(value, rest)| (off, data, len, Some(value), rest)),
                _ => skip(&mut decoder).map(|()| (off, data, len, sha, free)),
            }
        },
    )?;
    Ok(Chunk {
        off: off.ok_or(ProbeError::MissingField)?,
        data: data.ok_or(ProbeError::MissingField)?,
        len: len.ok_or(ProbeError::MissingField)?,
        sha: sha.ok_or(ProbeError::MissingField)?,
    })
}

fn read_key<'k>(decoder: &mut Decoder<&[u8]>, key: &'k mut [u8]) -> Result<&'k [u8], ProbeError> {
    match decoder.pull()? {
        Header::Text(Some(n)) if n <= key.len() => decoder
            .read_exact(&mut key[..n])
            .map(|()| &key[..n])
            .map_err(|_| ProbeError::Io),
        Header::Text(Some(n)) => skip_body(decoder, n).map(|()| &key[..0]),
        _ => Err(ProbeError::UnexpectedItem),
    }
}

fn read_u32(decoder: &mut Decoder<&[u8]>) -> Result<u32, ProbeError> {
    match decoder.pull()? {
        Header::Positive(n) => u32::try_from(n).map_err(|_| ProbeError::UnexpectedItem),
        _ => Err(ProbeError::UnexpectedItem),
    }
}

fn read_bytes<'s>(
    decoder: &mut Decoder<&[u8]>,
    free: &'s mut [u8],
) -> Result<(&'s [u8], &'s mut [u8]), ProbeError> {
    match decoder.pull()? {
        Header::Bytes(Some(n)) if n <= free.len() => {
            let (value, rest) = free.split_at_mut(n);
            decoder.read_exact(value).map_err(|_| ProbeError::Io)?;
            Ok((&*value, rest))
        }
        Header::Bytes(Some(_)) => Err(ProbeError::ScratchTooSmall),
        _ => Err(ProbeError::UnexpectedItem),
    }
}

fn skip_body(decoder: &mut Decoder<&[u8]>, n: usize) -> Result<(), ProbeError> {
    (0..n).try_for_each(|_| decoder.read_exact(&mut [0u8]).map_err(|_| ProbeError::Io))
}

fn skip(decoder: &mut Decoder<&[u8]>) -> Result<(), ProbeError> {
    let mut pending: u64 = 1;
    while pending > 0 {
        pending = pending - 1
            + match decoder.pull()? {
                Header::Bytes(Some(n)) | Header::Text(Some(n)) => skip_body(decoder, n).map(|()| 0),
                Header::Array(Some(n)) => Ok(n as u64),
                Header::Map(Some(n)) => Ok(2 * n as u64),
                Header::Tag(_) => Ok(1),
                Header::Positive(_)
                | Header::Negative(_)
                | Header::Float(_)
                | Header::Simple(_) => Ok(0),
                Header::Bytes(None)
                | Header::Text(None)
                | Header::Array(None)
                | Header::Map(None)
                | Header::Break => Err(ProbeError::UnexpectedItem),
            }?;
    }
    Ok(())
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
    fn decodes_by_copying_into_caller_scratch() {
        let mut scratch = [0u8; 96];
        let decoded = decode_chunk(&EXPECTED, &mut scratch).unwrap();
        assert_eq!(decoded, sample());
        assert!(!core::ptr::eq(
            decoded.sha.as_ptr(),
            EXPECTED[28..].as_ptr()
        ));
    }

    #[test]
    fn decode_ignores_unknown_key_and_accepts_any_key_order() {
        let input = b"\xa5\x63len\x18\x7b\x67upgrade\xf5\x63sha\x41\xaa\x63off\x00\x64data\x40";
        let mut scratch = [0u8; 96];
        let decoded = decode_chunk(input, &mut scratch).unwrap();
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
