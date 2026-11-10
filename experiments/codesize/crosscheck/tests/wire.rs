use crosscheck::{hand_side, serde_side, smol_side};

macro_rules! msg5 {
    ($m:ident, $kind:expr) => {
        $m::Msg5 {
            f_ts: $kind,
            f_bootable: 7,
            f_status: $m::Msg1 {
                f_count: 1 << 40,
                f_err: Some(3),
                f_status: 9,
                f_value: $m::Msg0 {
                    f_value: 1,
                    f_buf: &[1, 2, 3],
                    f_confirmed: None,
                    f_len: 300,
                    f_rsn: "hello",
                    f_name: true,
                },
            },
            f_flags: 0xdead,
            f_msg: -5,
        }
    };
}

macro_rules! variants {
    ($m:ident) => {
        [
            msg5!($m, $m::Msg5TsKind::A),
            msg5!($m, $m::Msg5TsKind::B(42)),
            msg5!($m, $m::Msg5TsKind::C { x: 7, y: false }),
        ]
    };
}

fn encode_serde(value: &serde_side::Msg5) -> Vec<u8> {
    let mut buf = [0u8; 256];
    let n = minicbor_serde::to_slice(value, &mut buf).expect("minicbor-serde encode");
    buf[..n].to_vec()
}

fn encode_hand(value: &hand_side::Msg5) -> Vec<u8> {
    let mut buf = [0u8; 256];
    let mut cursor = minicbor::encode::write::Cursor::new(&mut buf[..]);
    minicbor::encode(value, &mut cursor).expect("hand encode");
    let n = cursor.position();
    buf[..n].to_vec()
}

fn encode_smol(value: &smol_side::Msg5) -> Vec<u8> {
    let mut buf = [0u8; 256];
    let n = cbor_smol::cbor_serialize_to(value, &mut buf[..]).expect("cbor-smol encode");
    buf[..n].to_vec()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[test]
fn minicbor_serde_and_hand_written_agree_on_the_wire_for_every_variant() {
    for (serde_value, hand_value) in variants!(serde_side).iter().zip(variants!(hand_side).iter()) {
        let (from_serde, from_hand) = (encode_serde(serde_value), encode_hand(hand_value));
        println!("serde {}\nhand  {}", hex(&from_serde), hex(&from_hand));
        assert_eq!(from_serde, from_hand);
        let hand_decoded: hand_side::Msg5 = minicbor::decode(&from_serde).expect("hand decodes serde bytes");
        let serde_decoded: serde_side::Msg5 = minicbor_serde::from_slice(&from_hand).expect("serde decodes hand bytes");
        assert_eq!(encode_hand(&hand_decoded), from_serde);
        assert_eq!(encode_serde(&serde_decoded), from_hand);
    }
}

#[test]
fn cbor_smol_round_trips_every_variant_but_encodes_unit_variants_by_index() {
    for (smol_value, serde_value) in variants!(smol_side).iter().zip(variants!(serde_side).iter()) {
        let from_smol = encode_smol(smol_value);
        println!("smol  {}", hex(&from_smol));
        let from_serde = encode_serde(serde_value);
        println!("serde {}", hex(&from_serde));
        let decoded: smol_side::Msg5 = cbor_smol::cbor_deserialize(&from_smol).expect("cbor-smol decodes own bytes");
        assert_eq!(encode_smol(&decoded), from_smol);
    }
    assert_eq!(&encode_smol(&variants!(smol_side)[0])[..6], &[0xa5, 0x62, b't', b's', 0x00, 0x68]);
    assert_eq!(&encode_serde(&variants!(serde_side)[0])[..7], &[0xa5, 0x62, b't', b's', 0x61, b'A', 0x68]);
}
