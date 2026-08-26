use crate::{
    client::Client,
    error::Result,
    resp3::{
        Cmd, FromValue, Value,
        constants::{
            APPEND, CAD, CAS, DECR, DECRBY, DELEX, DIGEST, EX, EXAT, GET, GETDEL, GETEX, GETRANGE,
            GETSET, IDX, IFDEQ, IFDNE, IFEQ, IFNE, INCR, INCRBY, INCRBYFLOAT, KEEPTTL, LCS, LEN,
            MGET, MINMATCHLEN, MSET, MSETEX, MSETNX, NX, PERSIST, PSETEX, PX, PXAT, SET, SETEX,
            SETNX, SETRANGE, STRLEN, SUBSTR, WITHMATCHLEN, XX,
        },
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Set<'a> {
    Ex(u64),
    Px(u64),
    ExAt(u64),
    PxAt(u64),
    KeepTtl,
    Nx,
    Xx,
    IfEq(&'a str),
    IfNe(&'a str),
    IfDeq(&'a str),
    IfDne(&'a str),
    Get,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GetEx {
    Ex(u64),
    Px(u64),
    ExAt(u64),
    PxAt(u64),
    Persist,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DelEx<'a> {
    IfEq(&'a str),
    IfNe(&'a str),
    IfDeq(&'a str),
    IfDne(&'a str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LcsOption {
    Len,
    Idx,
    WithMatchLen,
    MinMatchLen(i64),
}

fn apply_set_option(cmd: Cmd, conf: &Set<'_>) -> Cmd {
    match conf {
        Set::Ex(s) => cmd.arg(EX).arg_int(*s),
        Set::Px(ms) => cmd.arg(PX).arg_int(*ms),
        Set::ExAt(ts) => cmd.arg(EXAT).arg_int(*ts),
        Set::PxAt(ts_ms) => cmd.arg(PXAT).arg_int(*ts_ms),
        Set::KeepTtl => cmd.arg(KEEPTTL),
        Set::Nx => cmd.arg(NX),
        Set::Xx => cmd.arg(XX),
        Set::IfEq(v) => cmd.arg(IFEQ).arg_bytes(v),
        Set::IfNe(v) => cmd.arg(IFNE).arg_bytes(v),
        Set::IfDeq(d) => cmd.arg(IFDEQ).arg_bytes(d),
        Set::IfDne(d) => cmd.arg(IFDNE).arg_bytes(d),
        Set::Get => cmd.arg(GET),
    }
}

fn apply_getex_option(cmd: Cmd, conf: &GetEx) -> Cmd {
    match conf {
        GetEx::Ex(s) => cmd.arg(EX).arg_int(*s),
        GetEx::Px(ms) => cmd.arg(PX).arg_int(*ms),
        GetEx::ExAt(ts) => cmd.arg(EXAT).arg_int(*ts),
        GetEx::PxAt(ts_ms) => cmd.arg(PXAT).arg_int(*ts_ms),
        GetEx::Persist => cmd.arg(PERSIST),
    }
}

fn apply_delex_option(cmd: Cmd, conf: &DelEx<'_>) -> Cmd {
    match conf {
        DelEx::IfEq(val) => cmd.arg(IFEQ).arg_bytes(val),
        DelEx::IfNe(val) => cmd.arg(IFNE).arg_bytes(val),
        DelEx::IfDeq(digest) => cmd.arg(IFDEQ).arg_bytes(digest),
        DelEx::IfDne(digest) => cmd.arg(IFDNE).arg_bytes(digest),
    }
}

fn apply_lcs_option(cmd: Cmd, conf: &LcsOption) -> Cmd {
    match conf {
        LcsOption::Len => cmd.arg(LEN),
        LcsOption::Idx => cmd.arg(IDX),
        LcsOption::WithMatchLen => cmd.arg(WITHMATCHLEN),
        LcsOption::MinMatchLen(m) => cmd.arg(MINMATCHLEN).arg_int(*m),
    }
}

impl Client {
    pub async fn get<T: FromValue>(&self, key: impl AsRef<[u8]>) -> Result<Option<T>> {
        self.execute_cmd(Cmd::new(GET).arg_bytes(key)).await
    }

    pub async fn set(
        &self,
        key: impl AsRef<[u8]>,
        val: impl AsRef<[u8]>,
        conf_li: impl AsRef<[Set<'_>]>,
    ) -> Result<Option<String>> {
        self.set_typed(key, val, conf_li).await
    }

    pub async fn set_typed<T: FromValue>(
        &self,
        key: impl AsRef<[u8]>,
        val: impl AsRef<[u8]>,
        conf_li: impl AsRef<[Set<'_>]>,
    ) -> Result<Option<T>> {
        let mut cmd = Cmd::new(SET).arg_bytes(key).arg_bytes(val);
        for conf in conf_li.as_ref() {
            cmd = apply_set_option(cmd, conf);
        }
        self.execute_cmd(cmd).await
    }

    pub async fn set_get<T: FromValue>(
        &self,
        key: impl AsRef<[u8]>,
        val: impl AsRef<[u8]>,
        conf_li: impl AsRef<[Set<'_>]>,
    ) -> Result<Option<T>> {
        let mut opts = conf_li.as_ref().to_vec();
        if !opts.contains(&Set::Get) {
            opts.push(Set::Get);
        }
        self.set_typed(key, val, &opts).await
    }

    pub async fn set_val(&self, key: impl AsRef<[u8]>, val: impl AsRef<[u8]>) -> Result<bool> {
        let res = self.set(key, val, &[]).await?;
        Ok(res.is_some())
    }

    pub async fn setex(
        &self,
        key: impl AsRef<[u8]>,
        seconds: u64,
        val: impl AsRef<[u8]>,
    ) -> Result<()> {
        self.execute_cmd(
            Cmd::new(SETEX)
                .arg_bytes(key)
                .arg_int(seconds)
                .arg_bytes(val),
        )
        .await
    }

    pub async fn psetex(
        &self,
        key: impl AsRef<[u8]>,
        milliseconds: u64,
        val: impl AsRef<[u8]>,
    ) -> Result<()> {
        self.execute_cmd(
            Cmd::new(PSETEX)
                .arg_bytes(key)
                .arg_int(milliseconds)
                .arg_bytes(val),
        )
        .await
    }

    pub async fn setnx(&self, key: impl AsRef<[u8]>, val: impl AsRef<[u8]>) -> Result<bool> {
        self.execute_cmd(Cmd::new(SETNX).arg_bytes(key).arg_bytes(val))
            .await
    }

    pub async fn mget<T: FromValue, K: AsRef<[u8]>>(&self, keys: &[K]) -> Result<Vec<Option<T>>> {
        let cmd = Cmd::new(MGET).args_slice(keys);
        self.execute_cmd(cmd).await
    }

    pub async fn mset<K: AsRef<[u8]>, V: AsRef<[u8]>>(&self, kvs: &[(K, V)]) -> Result<()> {
        self.execute_cmd(Cmd::new(MSET).args_pairs(kvs)).await
    }

    pub async fn msetnx<K: AsRef<[u8]>, V: AsRef<[u8]>>(&self, kvs: &[(K, V)]) -> Result<bool> {
        self.execute_cmd(Cmd::new(MSETNX).args_pairs(kvs)).await
    }

    pub async fn getset<T: FromValue>(
        &self,
        key: impl AsRef<[u8]>,
        val: impl AsRef<[u8]>,
    ) -> Result<Option<T>> {
        self.execute_cmd(Cmd::new(GETSET).arg_bytes(key).arg_bytes(val))
            .await
    }

    pub async fn getdel<T: FromValue>(&self, key: impl AsRef<[u8]>) -> Result<Option<T>> {
        self.execute_cmd(Cmd::new(GETDEL).arg_bytes(key)).await
    }

    pub async fn getex<T: FromValue>(
        &self,
        key: impl AsRef<[u8]>,
        conf_li: impl AsRef<[GetEx]>,
    ) -> Result<Option<T>> {
        let mut cmd = Cmd::new(GETEX).arg_bytes(key);
        for conf in conf_li.as_ref() {
            cmd = apply_getex_option(cmd, conf);
        }
        self.execute_cmd(cmd).await
    }

    pub async fn append(&self, key: impl AsRef<[u8]>, val: impl AsRef<[u8]>) -> Result<u64> {
        self.execute_cmd(Cmd::new(APPEND).arg_bytes(key).arg_bytes(val))
            .await
    }

    pub async fn strlen(&self, key: impl AsRef<[u8]>) -> Result<u64> {
        self.execute_cmd(Cmd::new(STRLEN).arg_bytes(key)).await
    }

    pub async fn getrange(&self, key: impl AsRef<[u8]>, start: i64, end: i64) -> Result<String> {
        let cmd = Cmd::new(GETRANGE)
            .arg_bytes(key)
            .arg_int(start)
            .arg_int(end);
        self.execute_cmd(cmd).await
    }

    pub async fn setrange(
        &self,
        key: impl AsRef<[u8]>,
        offset: u64,
        val: impl AsRef<[u8]>,
    ) -> Result<u64> {
        self.execute_cmd(
            Cmd::new(SETRANGE)
                .arg_bytes(key)
                .arg_int(offset)
                .arg_bytes(val),
        )
        .await
    }

    pub async fn incr(&self, key: impl AsRef<[u8]>) -> Result<i64> {
        self.execute_cmd(Cmd::new(INCR).arg_bytes(key)).await
    }

    pub async fn incrby(&self, key: impl AsRef<[u8]>, delta: i64) -> Result<i64> {
        self.execute_cmd(Cmd::new(INCRBY).arg_bytes(key).arg_int(delta))
            .await
    }

    pub async fn incrbyfloat(&self, key: impl AsRef<[u8]>, delta: f64) -> Result<f64> {
        self.execute_cmd(Cmd::new(INCRBYFLOAT).arg_bytes(key).arg_float(delta))
            .await
    }

    pub async fn decr(&self, key: impl AsRef<[u8]>) -> Result<i64> {
        self.execute_cmd(Cmd::new(DECR).arg_bytes(key)).await
    }

    pub async fn decrby(&self, key: impl AsRef<[u8]>, delta: i64) -> Result<i64> {
        self.execute_cmd(Cmd::new(DECRBY).arg_bytes(key).arg_int(delta))
            .await
    }

    pub async fn cas(
        &self,
        key: impl AsRef<[u8]>,
        old_val: impl AsRef<[u8]>,
        new_val: impl AsRef<[u8]>,
        ex: Option<u64>,
    ) -> Result<i64> {
        let cmd = Cmd::new(CAS)
            .arg_bytes(key)
            .arg_bytes(old_val)
            .arg_bytes(new_val)
            .arg_keyword_opt_int(EX, ex);
        self.execute_cmd(cmd).await
    }

    pub async fn cad(&self, key: impl AsRef<[u8]>, val: impl AsRef<[u8]>) -> Result<i64> {
        self.execute_cmd(Cmd::new(CAD).arg_bytes(key).arg_bytes(val))
            .await
    }

    pub async fn lcs(&self, key1: impl AsRef<[u8]>, key2: impl AsRef<[u8]>) -> Result<String> {
        self.execute_cmd(Cmd::new(LCS).arg_bytes(key1).arg_bytes(key2))
            .await
    }

    pub async fn lcs_opt(
        &self,
        key1: impl AsRef<[u8]>,
        key2: impl AsRef<[u8]>,
        conf_li: impl AsRef<[LcsOption]>,
    ) -> Result<Value> {
        let mut cmd = Cmd::new(LCS).arg_bytes(key1).arg_bytes(key2);
        for conf in conf_li.as_ref() {
            cmd = apply_lcs_option(cmd, conf);
        }
        self.execute(cmd).await
    }

    pub async fn digest(&self, key: impl AsRef<[u8]>) -> Result<Option<String>> {
        self.execute_cmd(Cmd::new(DIGEST).arg_bytes(key)).await
    }

    pub async fn substr(&self, key: impl AsRef<[u8]>, start: i64, end: i64) -> Result<String> {
        let cmd = Cmd::new(SUBSTR).arg_bytes(key).arg_int(start).arg_int(end);
        self.execute_cmd(cmd).await
    }

    pub async fn delex(
        &self,
        key: impl AsRef<[u8]>,
        conf_li: impl AsRef<[DelEx<'_>]>,
    ) -> Result<i64> {
        let mut cmd = Cmd::new(DELEX).arg_bytes(key);
        for conf in conf_li.as_ref() {
            cmd = apply_delex_option(cmd, conf);
        }
        self.execute_cmd(cmd).await
    }

    pub async fn msetex<K: AsRef<[u8]>, V: AsRef<[u8]>>(
        &self,
        kvs: &[(K, V)],
        conf_li: impl AsRef<[Set<'_>]>,
    ) -> Result<bool> {
        let mut cmd = Cmd::new(MSETEX).arg_int(kvs.len()).args_pairs(kvs);
        for conf in conf_li.as_ref() {
            cmd = apply_set_option(cmd, conf);
        }
        self.execute_cmd(cmd).await
    }
}
