use crate::{
    client::Client,
    error::Result,
    resp3::{
        Cmd,
        constants::{
            BIT, BITCOUNT, BITFIELD, BITFIELD_RO, BITOP, BITPOS, BYTE, GET, GETBIT, INCRBY,
            OVERFLOW, SET, SETBIT,
        },
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitIndexUnit {
    Byte,
    Bit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Bitfield<'a> {
    Get(&'a str, &'a str),
    Set(&'a str, &'a str, i64),
    IncrBy(&'a str, &'a str, i64),
    Overflow(&'a str),
}

fn apply_bit_unit(cmd: Cmd, unit: Option<BitIndexUnit>) -> Cmd {
    match unit {
        Some(BitIndexUnit::Byte) => cmd.arg(BYTE),
        Some(BitIndexUnit::Bit) => cmd.arg(BIT),
        None => cmd,
    }
}

fn apply_bitfield_option(cmd: Cmd, conf: &Bitfield<'_>) -> Cmd {
    match conf {
        Bitfield::Get(enc, offset) => cmd.arg(GET).arg_bytes(enc).arg_bytes(offset),
        Bitfield::Set(enc, offset, val) => {
            cmd.arg(SET).arg_bytes(enc).arg_bytes(offset).arg_int(*val)
        }
        Bitfield::IncrBy(enc, offset, inc) => cmd
            .arg(INCRBY)
            .arg_bytes(enc)
            .arg_bytes(offset)
            .arg_int(*inc),
        Bitfield::Overflow(ov) => cmd.arg(OVERFLOW).arg_bytes(ov),
    }
}

impl Client {
    pub async fn getbit(&self, key: impl AsRef<[u8]>, offset: u64) -> Result<u8> {
        let v: u64 = self
            .execute_cmd(Cmd::new(GETBIT).arg_bytes(key).arg_int(offset))
            .await?;
        Ok(v as u8)
    }

    pub async fn setbit(&self, key: impl AsRef<[u8]>, offset: u64, value: u8) -> Result<u8> {
        let v: u64 = self
            .execute_cmd(
                Cmd::new(SETBIT)
                    .arg_bytes(key)
                    .arg_int(offset)
                    .arg_int(value),
            )
            .await?;
        Ok(v as u8)
    }

    pub async fn bitcount(
        &self,
        key: impl AsRef<[u8]>,
        start: Option<i64>,
        end: Option<i64>,
    ) -> Result<u64> {
        self.bitcount_opt(key, start, end, None).await
    }

    pub async fn bitcount_opt(
        &self,
        key: impl AsRef<[u8]>,
        start: Option<i64>,
        end: Option<i64>,
        unit: Option<BitIndexUnit>,
    ) -> Result<u64> {
        let mut cmd = Cmd::new(BITCOUNT).arg_bytes(key);
        if let (Some(s), Some(e)) = (start, end) {
            cmd = apply_bit_unit(cmd.arg_int(s).arg_int(e), unit);
        }
        self.execute_cmd(cmd).await
    }

    pub async fn bitpos(
        &self,
        key: impl AsRef<[u8]>,
        bit: u8,
        start: Option<i64>,
        end: Option<i64>,
    ) -> Result<i64> {
        self.bitpos_opt(key, bit, start, end, None).await
    }

    pub async fn bitpos_opt(
        &self,
        key: impl AsRef<[u8]>,
        bit: u8,
        start: Option<i64>,
        end: Option<i64>,
        unit: Option<BitIndexUnit>,
    ) -> Result<i64> {
        let mut cmd = Cmd::new(BITPOS).arg_bytes(key).arg_int(bit);
        if let Some(s) = start {
            cmd = cmd.arg_int(s);
            if let Some(e) = end {
                cmd = apply_bit_unit(cmd.arg_int(e), unit);
            }
        }
        self.execute_cmd(cmd).await
    }

    pub async fn bitop<K: AsRef<[u8]>>(
        &self,
        op: &str,
        destkey: impl AsRef<[u8]>,
        srckeys: &[K],
    ) -> Result<u64> {
        let cmd = Cmd::new(BITOP)
            .arg_bytes(op)
            .arg_bytes(destkey)
            .args_slice(srckeys);
        self.execute_cmd(cmd).await
    }

    pub async fn bitfield(
        &self,
        key: impl AsRef<[u8]>,
        conf_li: impl AsRef<[Bitfield<'_>]>,
    ) -> Result<Vec<Option<i64>>> {
        let mut cmd = Cmd::new(BITFIELD).arg_bytes(key);
        for conf in conf_li.as_ref() {
            cmd = apply_bitfield_option(cmd, conf);
        }
        self.execute_cmd(cmd).await
    }

    pub async fn bitfield_ro(
        &self,
        key: impl AsRef<[u8]>,
        conf_li: impl AsRef<[Bitfield<'_>]>,
    ) -> Result<Vec<Option<i64>>> {
        let mut cmd = Cmd::new(BITFIELD_RO).arg_bytes(key);
        for conf in conf_li.as_ref() {
            if let Bitfield::Get(enc, offset) = conf {
                cmd = cmd.arg(GET).arg_bytes(enc).arg_bytes(offset);
            }
        }
        self.execute_cmd(cmd).await
    }
}
