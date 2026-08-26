use crate::{
    client::Client,
    error::Result,
    resp3::{
        Cmd, Value,
        constants::{
            BF_ADD, BF_CARD, BF_EXISTS, BF_INFO, BF_INSERT, BF_MADD, BF_MEXISTS, BF_RESERVE,
            BUCKETSIZE, CAPACITY, CF_ADD, CF_RESERVE, ERROR, EXPANSION, ITEMS, MAXITERATIONS,
            NOCREATE, NONSCALING,
        },
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BfReserve {
    Expansion(u32),
    NonScaling,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BfInsert {
    Capacity(u64),
    Error(f64),
    Expansion(u32),
    NoCreate,
    NonScaling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CfReserve {
    BucketSize(u32),
    MaxIterations(u32),
    Expansion(u32),
}

fn apply_bf_reserve(cmd: Cmd, conf: &BfReserve) -> Cmd {
    match conf {
        BfReserve::Expansion(exp) => cmd.arg(EXPANSION).arg_int(*exp),
        BfReserve::NonScaling => cmd.arg(NONSCALING),
    }
}

fn apply_bf_insert(cmd: Cmd, conf: &BfInsert) -> Cmd {
    match conf {
        BfInsert::Capacity(cap) => cmd.arg(CAPACITY).arg_int(*cap),
        BfInsert::Error(err) => cmd.arg(ERROR).arg_float(*err),
        BfInsert::Expansion(exp) => cmd.arg(EXPANSION).arg_int(*exp),
        BfInsert::NoCreate => cmd.arg(NOCREATE),
        BfInsert::NonScaling => cmd.arg(NONSCALING),
    }
}

fn apply_cf_reserve(cmd: Cmd, conf: &CfReserve) -> Cmd {
    match conf {
        CfReserve::BucketSize(bs) => cmd.arg(BUCKETSIZE).arg_int(*bs),
        CfReserve::MaxIterations(mi) => cmd.arg(MAXITERATIONS).arg_int(*mi),
        CfReserve::Expansion(exp) => cmd.arg(EXPANSION).arg_int(*exp),
    }
}

impl Client {
    pub async fn bf_reserve(
        &self,
        key: impl AsRef<[u8]>,
        error_rate: f64,
        capacity: u64,
        conf_li: impl AsRef<[BfReserve]>,
    ) -> Result<()> {
        let mut cmd = Cmd::new(BF_RESERVE)
            .arg_bytes(key)
            .arg_float(error_rate)
            .arg_int(capacity);
        for conf in conf_li.as_ref() {
            cmd = apply_bf_reserve(cmd, conf);
        }
        self.execute_cmd(cmd).await
    }

    pub async fn bf_add(&self, key: impl AsRef<[u8]>, item: impl AsRef<[u8]>) -> Result<bool> {
        self.execute_cmd(Cmd::new(BF_ADD).arg_bytes(key).arg_bytes(item))
            .await
    }

    pub async fn bf_madd<I: AsRef<[u8]>>(
        &self,
        key: impl AsRef<[u8]>,
        items: &[I],
    ) -> Result<Vec<bool>> {
        self.execute_cmd(Cmd::new(BF_MADD).arg_bytes(key).args_slice(items))
            .await
    }

    pub async fn bf_insert<I: AsRef<[u8]>>(
        &self,
        key: impl AsRef<[u8]>,
        items: &[I],
        conf_li: impl AsRef<[BfInsert]>,
    ) -> Result<Vec<bool>> {
        let mut cmd = Cmd::new(BF_INSERT).arg_bytes(key);
        for conf in conf_li.as_ref() {
            cmd = apply_bf_insert(cmd, conf);
        }
        cmd = cmd.arg(ITEMS).args_slice(items);
        self.execute_cmd(cmd).await
    }

    pub async fn bf_exists(&self, key: impl AsRef<[u8]>, item: impl AsRef<[u8]>) -> Result<bool> {
        self.execute_cmd(Cmd::new(BF_EXISTS).arg_bytes(key).arg_bytes(item))
            .await
    }

    pub async fn bf_mexists<I: AsRef<[u8]>>(
        &self,
        key: impl AsRef<[u8]>,
        items: &[I],
    ) -> Result<Vec<bool>> {
        self.execute_cmd(Cmd::new(BF_MEXISTS).arg_bytes(key).args_slice(items))
            .await
    }

    pub async fn bf_info(&self, key: impl AsRef<[u8]>) -> Result<Value> {
        self.bf_info_opt(key, None).await
    }

    pub async fn bf_info_opt(&self, key: impl AsRef<[u8]>, sub: Option<&str>) -> Result<Value> {
        self.execute(Cmd::new(BF_INFO).arg_bytes(key).arg_opt_bytes(sub))
            .await
    }

    pub async fn bf_card(&self, key: impl AsRef<[u8]>) -> Result<u64> {
        self.execute_cmd(Cmd::new(BF_CARD).arg_bytes(key)).await
    }

    pub async fn cf_reserve(
        &self,
        key: impl AsRef<[u8]>,
        capacity: u64,
        conf_li: impl AsRef<[CfReserve]>,
    ) -> Result<()> {
        let mut cmd = Cmd::new(CF_RESERVE).arg_bytes(key).arg_int(capacity);
        for conf in conf_li.as_ref() {
            cmd = apply_cf_reserve(cmd, conf);
        }
        self.execute_cmd(cmd).await
    }

    pub async fn cf_add(&self, key: impl AsRef<[u8]>, item: impl AsRef<[u8]>) -> Result<bool> {
        self.execute_cmd(Cmd::new(CF_ADD).arg_bytes(key).arg_bytes(item))
            .await
    }
}
