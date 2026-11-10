#[cfg(test)]
mod vectors;

#[cfg(test)]
mod tests {
    use super::vectors::VECTORS;
    use ciborium_io::{Read, Write};
    use ciborium_ll::{Decoder, Encoder, Header};

    fn hex(s: &str) -> Vec<u8> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }

    fn roundtrip(input: &[u8]) -> Result<Vec<u8>, String> {
        let mut out = [0u8; 256];
        let mut remaining = &mut out[..];
        let mut dec = Decoder::from(input);
        {
            let mut enc = Encoder::from(&mut remaining);
            while dec.offset() < input.len() {
                let h = dec.pull().map_err(|e| format!("{e:?}"))?;
                enc.push(h).map_err(|e| format!("{e:?}"))?;
                if let Header::Bytes(Some(n)) | Header::Text(Some(n)) = h {
                    let mut body = vec![0u8; n];
                    dec.read_exact(&mut body).map_err(|e| format!("{e:?}"))?;
                    enc.write_all(&body).map_err(|e| format!("{e:?}"))?;
                }
            }
        }
        let n = 256 - remaining.len();
        Ok(out[..n].to_vec())
    }

    #[test]
    fn rfc8949_appendix_a_header_roundtrip() {
        let failures: Vec<String> = VECTORS
            .iter()
            .filter_map(|(diag, h)| match roundtrip(&hex(h)) {
                Err(e) => Some(format!("DECODE-ERR {diag} {h}: {e}")),
                Ok(out) => (out != hex(h)).then(|| {
                    format!(
                        "REENCODE-DIFF {diag} {h} -> {}",
                        out.iter().map(|b| format!("{b:02x}")).collect::<String>()
                    )
                }),
            })
            .collect();
        println!("vectors={} failures={}", VECTORS.len(), failures.len());
        failures.iter().for_each(|f| println!("{f}"));
        assert_eq!(VECTORS.len(), 81);
        assert_eq!(
            failures,
            [
                "REENCODE-DIFF Infinity fa7f800000 -> f97c00",
                "REENCODE-DIFF NaN fa7fc00000 -> f97e00",
                "REENCODE-DIFF -Infinity faff800000 -> f9fc00",
                "REENCODE-DIFF Infinity fb7ff0000000000000 -> f97c00",
                "REENCODE-DIFF NaN fb7ff8000000000000 -> f97e00",
                "REENCODE-DIFF -Infinity fbfff0000000000000 -> f9fc00",
            ]
        );
        assert_eq!(
            Decoder::from(&[0xf8u8, 0x18][..]).pull().unwrap(),
            Header::Simple(24)
        );
        assert_eq!(
            Decoder::from(&[0x1au8, 0, 0, 0, 1][..]).pull().unwrap(),
            Header::Positive(1)
        );
        println!(
            "simple(24) f818 -> {:?}",
            Decoder::from(&[0xf8u8, 0x18][..]).pull()
        );
        println!(
            "simple(20) f814 -> {:?}",
            Decoder::from(&[0xf8u8, 0x14][..]).pull()
        );
        println!(
            "non-preferred 1a00000001 -> {:?}",
            Decoder::from(&[0x1au8, 0, 0, 0, 1][..]).pull()
        );
    }
}
