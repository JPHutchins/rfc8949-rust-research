#[allow(dead_code)]
pub mod serde_side {
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Msg0<'a> {
    #[serde(rename = "value")] pub f_value: u32,
    #[serde(rename = "buf", borrow, with = "serde_bytes")] pub f_buf: &'a [u8],
    #[serde(rename = "confirmed", default, skip_serializing_if = "Option::is_none")] pub f_confirmed: Option<u32>,
    #[serde(rename = "len")] pub f_len: u32,
    #[serde(rename = "rsn")] pub f_rsn: &'a str,
    #[serde(rename = "name")] pub f_name: bool,
}


#[derive(serde::Serialize, serde::Deserialize)]
pub struct Msg1<'a> {
    #[serde(rename = "count")] pub f_count: u64,
    #[serde(rename = "err", default, skip_serializing_if = "Option::is_none")] pub f_err: Option<u32>,
    #[serde(rename = "status")] pub f_status: u64,
    #[serde(rename = "value", borrow)] pub f_value: Msg0<'a>,
}


#[derive(serde::Serialize, serde::Deserialize)]
pub struct Msg2 {
    #[serde(rename = "data")] pub f_data: i32,
    #[serde(rename = "bootable", default, skip_serializing_if = "Option::is_none")] pub f_bootable: Option<u32>,
    #[serde(rename = "msg", default, skip_serializing_if = "Option::is_none")] pub f_msg: Option<u32>,
}


#[derive(serde::Serialize, serde::Deserialize)]
pub struct Msg3<'a> {
    #[serde(rename = "count")] pub f_count: Msg2,
    #[serde(rename = "mtu")] pub f_mtu: u64,
    #[serde(rename = "upgrade")] pub f_upgrade: u32,
    #[serde(rename = "bootable", borrow)] pub f_bootable: Msg1<'a>,
}


#[derive(serde::Serialize, serde::Deserialize)]
pub struct Msg4<'a> {
    #[serde(rename = "version", borrow)] pub f_version: Msg1<'a>,
    #[serde(rename = "sha")] pub f_sha: u64,
    #[serde(rename = "ts")] pub f_ts: u64,
    #[serde(rename = "pending")] pub f_pending: u32,
}


#[derive(serde::Serialize, serde::Deserialize)]
pub enum Msg5TsKind { A, B(u32), C { x: u32, y: bool } }
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Msg5<'a> {
    #[serde(rename = "ts")] pub f_ts: Msg5TsKind,
    #[serde(rename = "bootable")] pub f_bootable: u32,
    #[serde(rename = "status", borrow)] pub f_status: Msg1<'a>,
    #[serde(rename = "flags")] pub f_flags: u32,
    #[serde(rename = "msg")] pub f_msg: i32,
}


}
#[allow(dead_code)]
pub mod hand_side {

pub struct Msg0<'a> {
    pub f_value: u32,
    pub f_buf: &'a [u8],
    pub f_confirmed: Option<u32>,
    pub f_len: u32,
    pub f_rsn: &'a str,
    pub f_name: bool,
}

impl<'a> minicbor::Encode<()> for Msg0<'a> {
    fn encode<W: minicbor::encode::Write>(&self, e: &mut minicbor::Encoder<W>, _: &mut ()) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.map(5 + self.f_confirmed.is_some() as u64)?;
        e.str("value")?.u32(self.f_value)?;
        e.str("buf")?.bytes(self.f_buf)?;
        if let Some(v) = self.f_confirmed { e.str("confirmed")?.u32(v)?; }
        e.str("len")?.u32(self.f_len)?;
        e.str("rsn")?.str(self.f_rsn)?;
        e.str("name")?.bool(self.f_name)?;
        Ok(())
    }
}

impl<'a> minicbor::Decode<'a, ()> for Msg0<'a> {
    fn decode(d: &mut minicbor::Decoder<'a>, _: &mut ()) -> Result<Self, minicbor::decode::Error> {
        let mut f_value = None;
        let mut f_buf = None;
        let mut f_confirmed = None;
        let mut f_len = None;
        let mut f_rsn = None;
        let mut f_name = None;
        let entries = d.map()?.ok_or_else(|| minicbor::decode::Error::message("indefinite map"))?;
        for _ in 0..entries {
            match d.str()? {
                "value" => f_value = Some(d.u32()?),
                "buf" => f_buf = Some(d.bytes()?),
                "confirmed" => f_confirmed = Some(d.u32()?),
                "len" => f_len = Some(d.u32()?),
                "rsn" => f_rsn = Some(d.str()?),
                "name" => f_name = Some(d.bool()?),
                _ => d.skip()?,
            }
        }
        Ok(Msg0 {
            f_value: f_value.ok_or_else(|| minicbor::decode::Error::missing_value(0))?,
            f_buf: f_buf.ok_or_else(|| minicbor::decode::Error::missing_value(1))?,
            f_confirmed,
            f_len: f_len.ok_or_else(|| minicbor::decode::Error::missing_value(3))?,
            f_rsn: f_rsn.ok_or_else(|| minicbor::decode::Error::missing_value(4))?,
            f_name: f_name.ok_or_else(|| minicbor::decode::Error::missing_value(5))?,
        })
    }
}


pub struct Msg1<'a> {
    pub f_count: u64,
    pub f_err: Option<u32>,
    pub f_status: u64,
    pub f_value: Msg0<'a>,
}

impl<'a> minicbor::Encode<()> for Msg1<'a> {
    fn encode<W: minicbor::encode::Write>(&self, e: &mut minicbor::Encoder<W>, _: &mut ()) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.map(3 + self.f_err.is_some() as u64)?;
        e.str("count")?.u64(self.f_count)?;
        if let Some(v) = self.f_err { e.str("err")?.u32(v)?; }
        e.str("status")?.u64(self.f_status)?;
        e.str("value")?.encode(&self.f_value)?;
        Ok(())
    }
}

impl<'a> minicbor::Decode<'a, ()> for Msg1<'a> {
    fn decode(d: &mut minicbor::Decoder<'a>, _: &mut ()) -> Result<Self, minicbor::decode::Error> {
        let mut f_count = None;
        let mut f_err = None;
        let mut f_status = None;
        let mut f_value = None;
        let entries = d.map()?.ok_or_else(|| minicbor::decode::Error::message("indefinite map"))?;
        for _ in 0..entries {
            match d.str()? {
                "count" => f_count = Some(d.u64()?),
                "err" => f_err = Some(d.u32()?),
                "status" => f_status = Some(d.u64()?),
                "value" => f_value = Some(d.decode()?),
                _ => d.skip()?,
            }
        }
        Ok(Msg1 {
            f_count: f_count.ok_or_else(|| minicbor::decode::Error::missing_value(0))?,
            f_err,
            f_status: f_status.ok_or_else(|| minicbor::decode::Error::missing_value(2))?,
            f_value: f_value.ok_or_else(|| minicbor::decode::Error::missing_value(3))?,
        })
    }
}


pub struct Msg2 {
    pub f_data: i32,
    pub f_bootable: Option<u32>,
    pub f_msg: Option<u32>,
}

impl minicbor::Encode<()> for Msg2 {
    fn encode<W: minicbor::encode::Write>(&self, e: &mut minicbor::Encoder<W>, _: &mut ()) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.map(1 + self.f_bootable.is_some() as u64 + self.f_msg.is_some() as u64)?;
        e.str("data")?.i32(self.f_data)?;
        if let Some(v) = self.f_bootable { e.str("bootable")?.u32(v)?; }
        if let Some(v) = self.f_msg { e.str("msg")?.u32(v)?; }
        Ok(())
    }
}

impl<'b> minicbor::Decode<'b, ()> for Msg2 {
    fn decode(d: &mut minicbor::Decoder<'b>, _: &mut ()) -> Result<Self, minicbor::decode::Error> {
        let mut f_data = None;
        let mut f_bootable = None;
        let mut f_msg = None;
        let entries = d.map()?.ok_or_else(|| minicbor::decode::Error::message("indefinite map"))?;
        for _ in 0..entries {
            match d.str()? {
                "data" => f_data = Some(d.i32()?),
                "bootable" => f_bootable = Some(d.u32()?),
                "msg" => f_msg = Some(d.u32()?),
                _ => d.skip()?,
            }
        }
        Ok(Msg2 {
            f_data: f_data.ok_or_else(|| minicbor::decode::Error::missing_value(0))?,
            f_bootable,
            f_msg,
        })
    }
}


pub struct Msg3<'a> {
    pub f_count: Msg2,
    pub f_mtu: u64,
    pub f_upgrade: u32,
    pub f_bootable: Msg1<'a>,
}

impl<'a> minicbor::Encode<()> for Msg3<'a> {
    fn encode<W: minicbor::encode::Write>(&self, e: &mut minicbor::Encoder<W>, _: &mut ()) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.map(4)?;
        e.str("count")?.encode(&self.f_count)?;
        e.str("mtu")?.u64(self.f_mtu)?;
        e.str("upgrade")?.u32(self.f_upgrade)?;
        e.str("bootable")?.encode(&self.f_bootable)?;
        Ok(())
    }
}

impl<'a> minicbor::Decode<'a, ()> for Msg3<'a> {
    fn decode(d: &mut minicbor::Decoder<'a>, _: &mut ()) -> Result<Self, minicbor::decode::Error> {
        let mut f_count = None;
        let mut f_mtu = None;
        let mut f_upgrade = None;
        let mut f_bootable = None;
        let entries = d.map()?.ok_or_else(|| minicbor::decode::Error::message("indefinite map"))?;
        for _ in 0..entries {
            match d.str()? {
                "count" => f_count = Some(d.decode()?),
                "mtu" => f_mtu = Some(d.u64()?),
                "upgrade" => f_upgrade = Some(d.u32()?),
                "bootable" => f_bootable = Some(d.decode()?),
                _ => d.skip()?,
            }
        }
        Ok(Msg3 {
            f_count: f_count.ok_or_else(|| minicbor::decode::Error::missing_value(0))?,
            f_mtu: f_mtu.ok_or_else(|| minicbor::decode::Error::missing_value(1))?,
            f_upgrade: f_upgrade.ok_or_else(|| minicbor::decode::Error::missing_value(2))?,
            f_bootable: f_bootable.ok_or_else(|| minicbor::decode::Error::missing_value(3))?,
        })
    }
}


pub struct Msg4<'a> {
    pub f_version: Msg1<'a>,
    pub f_sha: u64,
    pub f_ts: u64,
    pub f_pending: u32,
}

impl<'a> minicbor::Encode<()> for Msg4<'a> {
    fn encode<W: minicbor::encode::Write>(&self, e: &mut minicbor::Encoder<W>, _: &mut ()) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.map(4)?;
        e.str("version")?.encode(&self.f_version)?;
        e.str("sha")?.u64(self.f_sha)?;
        e.str("ts")?.u64(self.f_ts)?;
        e.str("pending")?.u32(self.f_pending)?;
        Ok(())
    }
}

impl<'a> minicbor::Decode<'a, ()> for Msg4<'a> {
    fn decode(d: &mut minicbor::Decoder<'a>, _: &mut ()) -> Result<Self, minicbor::decode::Error> {
        let mut f_version = None;
        let mut f_sha = None;
        let mut f_ts = None;
        let mut f_pending = None;
        let entries = d.map()?.ok_or_else(|| minicbor::decode::Error::message("indefinite map"))?;
        for _ in 0..entries {
            match d.str()? {
                "version" => f_version = Some(d.decode()?),
                "sha" => f_sha = Some(d.u64()?),
                "ts" => f_ts = Some(d.u64()?),
                "pending" => f_pending = Some(d.u32()?),
                _ => d.skip()?,
            }
        }
        Ok(Msg4 {
            f_version: f_version.ok_or_else(|| minicbor::decode::Error::missing_value(0))?,
            f_sha: f_sha.ok_or_else(|| minicbor::decode::Error::missing_value(1))?,
            f_ts: f_ts.ok_or_else(|| minicbor::decode::Error::missing_value(2))?,
            f_pending: f_pending.ok_or_else(|| minicbor::decode::Error::missing_value(3))?,
        })
    }
}

pub enum Msg5TsKind { A, B(u32), C { x: u32, y: bool } }

impl minicbor::Encode<()> for Msg5TsKind {
    fn encode<W: minicbor::encode::Write>(&self, e: &mut minicbor::Encoder<W>, _: &mut ()) -> Result<(), minicbor::encode::Error<W::Error>> {
        match self {
            Msg5TsKind::A => { e.str("A")?; }
            Msg5TsKind::B(v) => { e.map(1)?.str("B")?.u32(*v)?; }
            Msg5TsKind::C { x, y } => { e.map(1)?.str("C")?.map(2)?.str("x")?.u32(*x)?.str("y")?.bool(*y)?; }
        }
        Ok(())
    }
}

impl<'b> minicbor::Decode<'b, ()> for Msg5TsKind {
    fn decode(d: &mut minicbor::Decoder<'b>, _: &mut ()) -> Result<Self, minicbor::decode::Error> {
        match d.datatype()? {
            minicbor::data::Type::String => match d.str()? {
                "A" => Ok(Msg5TsKind::A),
                _ => Err(minicbor::decode::Error::message("unknown variant")),
            },
            _ => {
                d.map()?;
                match d.str()? {
                    "B" => Ok(Msg5TsKind::B(d.u32()?)),
                    "C" => {
                        let mut x = None;
                        let mut y = None;
                        let n = d.map()?.ok_or_else(|| minicbor::decode::Error::message("indefinite map"))?;
                        for _ in 0..n {
                            match d.str()? { "x" => x = Some(d.u32()?), "y" => y = Some(d.bool()?), _ => d.skip()? }
                        }
                        Ok(Msg5TsKind::C { x: x.ok_or_else(|| minicbor::decode::Error::missing_value(0))?, y: y.ok_or_else(|| minicbor::decode::Error::missing_value(1))? })
                    }
                    _ => Err(minicbor::decode::Error::message("unknown variant")),
                }
            }
        }
    }
}

pub struct Msg5<'a> {
    pub f_ts: Msg5TsKind,
    pub f_bootable: u32,
    pub f_status: Msg1<'a>,
    pub f_flags: u32,
    pub f_msg: i32,
}

impl<'a> minicbor::Encode<()> for Msg5<'a> {
    fn encode<W: minicbor::encode::Write>(&self, e: &mut minicbor::Encoder<W>, _: &mut ()) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.map(5)?;
        e.str("ts")?.encode(&self.f_ts)?;
        e.str("bootable")?.u32(self.f_bootable)?;
        e.str("status")?.encode(&self.f_status)?;
        e.str("flags")?.u32(self.f_flags)?;
        e.str("msg")?.i32(self.f_msg)?;
        Ok(())
    }
}

impl<'a> minicbor::Decode<'a, ()> for Msg5<'a> {
    fn decode(d: &mut minicbor::Decoder<'a>, _: &mut ()) -> Result<Self, minicbor::decode::Error> {
        let mut f_ts = None;
        let mut f_bootable = None;
        let mut f_status = None;
        let mut f_flags = None;
        let mut f_msg = None;
        let entries = d.map()?.ok_or_else(|| minicbor::decode::Error::message("indefinite map"))?;
        for _ in 0..entries {
            match d.str()? {
                "ts" => f_ts = Some(d.decode()?),
                "bootable" => f_bootable = Some(d.u32()?),
                "status" => f_status = Some(d.decode()?),
                "flags" => f_flags = Some(d.u32()?),
                "msg" => f_msg = Some(d.i32()?),
                _ => d.skip()?,
            }
        }
        Ok(Msg5 {
            f_ts: f_ts.ok_or_else(|| minicbor::decode::Error::missing_value(0))?,
            f_bootable: f_bootable.ok_or_else(|| minicbor::decode::Error::missing_value(1))?,
            f_status: f_status.ok_or_else(|| minicbor::decode::Error::missing_value(2))?,
            f_flags: f_flags.ok_or_else(|| minicbor::decode::Error::missing_value(3))?,
            f_msg: f_msg.ok_or_else(|| minicbor::decode::Error::missing_value(4))?,
        })
    }
}

}
#[allow(dead_code)]
pub mod smol_side {
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Msg0<'a> {
    #[serde(rename = "value")] pub f_value: u32,
    #[serde(rename = "buf", borrow, with = "serde_bytes")] pub f_buf: &'a [u8],
    #[serde(rename = "confirmed", default, skip_serializing_if = "Option::is_none")] pub f_confirmed: Option<u32>,
    #[serde(rename = "len")] pub f_len: u32,
    #[serde(rename = "rsn")] pub f_rsn: &'a str,
    #[serde(rename = "name")] pub f_name: bool,
}


#[derive(serde::Serialize, serde::Deserialize)]
pub struct Msg1<'a> {
    #[serde(rename = "count")] pub f_count: u64,
    #[serde(rename = "err", default, skip_serializing_if = "Option::is_none")] pub f_err: Option<u32>,
    #[serde(rename = "status")] pub f_status: u64,
    #[serde(rename = "value", borrow)] pub f_value: Msg0<'a>,
}


#[derive(serde::Serialize, serde::Deserialize)]
pub struct Msg2 {
    #[serde(rename = "data")] pub f_data: i32,
    #[serde(rename = "bootable", default, skip_serializing_if = "Option::is_none")] pub f_bootable: Option<u32>,
    #[serde(rename = "msg", default, skip_serializing_if = "Option::is_none")] pub f_msg: Option<u32>,
}


#[derive(serde::Serialize, serde::Deserialize)]
pub struct Msg3<'a> {
    #[serde(rename = "count")] pub f_count: Msg2,
    #[serde(rename = "mtu")] pub f_mtu: u64,
    #[serde(rename = "upgrade")] pub f_upgrade: u32,
    #[serde(rename = "bootable", borrow)] pub f_bootable: Msg1<'a>,
}


#[derive(serde::Serialize, serde::Deserialize)]
pub struct Msg4<'a> {
    #[serde(rename = "version", borrow)] pub f_version: Msg1<'a>,
    #[serde(rename = "sha")] pub f_sha: u64,
    #[serde(rename = "ts")] pub f_ts: u64,
    #[serde(rename = "pending")] pub f_pending: u32,
}


#[derive(serde::Serialize, serde::Deserialize)]
pub enum Msg5TsKind { A, B(u32), C { x: u32, y: bool } }
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Msg5<'a> {
    #[serde(rename = "ts")] pub f_ts: Msg5TsKind,
    #[serde(rename = "bootable")] pub f_bootable: u32,
    #[serde(rename = "status", borrow)] pub f_status: Msg1<'a>,
    #[serde(rename = "flags")] pub f_flags: u32,
    #[serde(rename = "msg")] pub f_msg: i32,
}


}
