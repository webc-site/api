use crate::{
    client::Client,
    error::Result,
    resp3::{
        Cmd, Value,
        constants::{
            CURSOR, LIMIT, SIADD, SICARD, SIEXISTS, SIRANGE, SIRANGEBYVALUE, SIREM, SIREVRANGE,
            SIREVRANGEBYVALUE,
        },
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SiRange {
    Cursor(u64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SiRangeByValue {
    Limit(usize, usize),
}

fn build_sirange_cmd(
    name: &'static str,
    key: impl AsRef<[u8]>,
    offset: u64,
    limit: u64,
    conf_li: &[SiRange],
) -> Cmd {
    let mut cmd = Cmd::new(name).arg_bytes(key).arg_int(offset).arg_int(limit);
    for conf in conf_li {
        match conf {
            SiRange::Cursor(c) => {
                cmd = cmd.arg(CURSOR).arg_int(*c);
            }
        }
    }
    cmd
}

fn build_sirangebyvalue_cmd(
    name: &'static str,
    key: impl AsRef<[u8]>,
    arg1: u64,
    arg2: u64,
    conf_li: &[SiRangeByValue],
) -> Cmd {
    let mut cmd = Cmd::new(name).arg_bytes(key).arg_int(arg1).arg_int(arg2);
    for conf in conf_li {
        match conf {
            SiRangeByValue::Limit(off, cnt) => {
                cmd = cmd.arg(LIMIT).arg_int(*off).arg_int(*cnt);
            }
        }
    }
    cmd
}

impl Client {
    pub async fn siadd(&self, key: impl AsRef<[u8]>, ids: &[u64]) -> Result<u64> {
        self.execute_cmd(Cmd::new(SIADD).arg_bytes(key).args_ints(ids))
            .await
    }

    pub async fn sirem(&self, key: impl AsRef<[u8]>, ids: &[u64]) -> Result<u64> {
        self.execute_cmd(Cmd::new(SIREM).arg_bytes(key).args_ints(ids))
            .await
    }

    pub async fn sicard(&self, key: impl AsRef<[u8]>) -> Result<u64> {
        self.execute_cmd(Cmd::new(SICARD).arg_bytes(key)).await
    }

    pub async fn siexists(&self, key: impl AsRef<[u8]>, id: u64) -> Result<bool> {
        let val = self
            .execute(Cmd::new(SIEXISTS).arg_bytes(key).arg_int(id))
            .await?;
        match val {
            Value::Array(items) if !items.is_empty() => items[0].as_bool(),
            v => v.as_bool(),
        }
    }

    pub async fn msiexists(&self, key: impl AsRef<[u8]>, ids: &[u64]) -> Result<Vec<bool>> {
        self.execute_cmd(Cmd::new(SIEXISTS).arg_bytes(key).args_ints(ids))
            .await
    }

    pub async fn sirange(
        &self,
        key: impl AsRef<[u8]>,
        offset: u64,
        limit: u64,
        conf_li: impl AsRef<[SiRange]>,
    ) -> Result<Vec<u64>> {
        let cmd = build_sirange_cmd(SIRANGE, key, offset, limit, conf_li.as_ref());
        self.execute_cmd(cmd).await
    }

    pub async fn sirevrange(
        &self,
        key: impl AsRef<[u8]>,
        offset: u64,
        limit: u64,
        conf_li: impl AsRef<[SiRange]>,
    ) -> Result<Vec<u64>> {
        let cmd = build_sirange_cmd(SIREVRANGE, key, offset, limit, conf_li.as_ref());
        self.execute_cmd(cmd).await
    }

    pub async fn sirangebyvalue(
        &self,
        key: impl AsRef<[u8]>,
        min: u64,
        max: u64,
        conf_li: impl AsRef<[SiRangeByValue]>,
    ) -> Result<Vec<u64>> {
        let cmd = build_sirangebyvalue_cmd(SIRANGEBYVALUE, key, min, max, conf_li.as_ref());
        self.execute_cmd(cmd).await
    }

    pub async fn sirevrangebyvalue(
        &self,
        key: impl AsRef<[u8]>,
        max: u64,
        min: u64,
        conf_li: impl AsRef<[SiRangeByValue]>,
    ) -> Result<Vec<u64>> {
        let cmd = build_sirangebyvalue_cmd(SIREVRANGEBYVALUE, key, max, min, conf_li.as_ref());
        self.execute_cmd(cmd).await
    }
}
