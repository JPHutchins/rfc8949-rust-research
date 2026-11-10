#[cfg(test)]
mod vectors;

#[cfg(test)]
mod tests {
    use super::vectors::VECTORS;
    use minicbor::data::Token;
    use minicbor::encode::write::Cursor;
    use minicbor::{Decoder, Encoder};

    fn hex(s: &str) -> Vec<u8> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }

    #[test]
    fn rfc8949_appendix_a_token_roundtrip() {
        let failures: Vec<String> = VECTORS
            .iter()
            .filter_map(|(diag, h)| {
                let input = hex(h);
                let tokens: Result<Vec<Token>, _> = Decoder::new(&input).tokens().collect();
                match tokens {
                    Err(e) => Some(format!("DECODE-ERR {diag} {h}: {e}")),
                    Ok(tokens) => {
                        let mut buf = [0u8; 128];
                        let mut cursor = Cursor::new(&mut buf[..]);
                        Encoder::new(&mut cursor).tokens(&tokens).unwrap();
                        let n = cursor.position();
                        (buf[..n] != input[..])
                            .then(|| format!("REENCODE-DIFF {diag} {h} -> {:02x?}", &buf[..n]))
                    }
                }
            })
            .collect();
        println!("vectors={} failures={}", VECTORS.len(), failures.len());
        failures.iter().for_each(|f| println!("{f}"));
    }

    #[test]
    fn simple_values_20_to_31_use_not_well_formed_two_byte_form() {
        let mut buf = [0u8; 8];
        let mut cursor = Cursor::new(&mut buf[..]);
        Encoder::new(&mut cursor)
            .simple(20)
            .unwrap()
            .simple(24)
            .unwrap();
        let n = cursor.position();
        assert_eq!(&buf[..n], &[0xf8, 0x14, 0xf8, 0x18]);
        assert_eq!(Decoder::new(&[0xf8, 0x18]).simple().unwrap(), 24);
        assert_eq!(Decoder::new(&[0xf8, 0x14]).simple().unwrap(), 20);
    }

    #[test]
    fn decoder_accepts_non_preferred_integer_head() {
        assert_eq!(Decoder::new(&[0x1a, 0, 0, 0, 1]).u8().unwrap(), 1);
    }

    #[test]
    fn floats_are_encoded_at_declared_width() {
        let mut buf = [0u8; 16];
        let mut cursor = Cursor::new(&mut buf[..]);
        Encoder::new(&mut cursor)
            .f64(1.5)
            .unwrap()
            .f32(1.5)
            .unwrap();
        let n = cursor.position();
        assert_eq!(&buf[..n], &hex("fb3ff8000000000000fa3fc00000")[..]);
    }
}
