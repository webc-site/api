use rapidhash::RapidHashMap;

use crate::{
    client::{Client, helper::apply_scan_opts},
    error::Result,
    resp3::{
        Cmd, FromValue,
        constants::{
            EX, EXAT, FIELDS, FNX, FXX, GT, HDEL, HEXISTS, HEXPIRE, HEXPIREAT, HEXPIRETIME, HGET,
            HGETALL, HGETEX, HINCRBY, HINCRBYFLOAT, HKEYS, HLEN, HMGET, HMSET, HPERSIST, HPEXPIRE,
            HPEXPIREAT, HPEXPIRETIME, HPTTL, HRANDFIELD, HRANGEBYLEX, HSCAN, HSET, HSETEX,
            HSETEXPIRE, HSETNX, HSTRLEN, HTTL, HVALS, KEEPTTL, LIMIT, LT, NX, PERSIST, PX, PXAT,
            WITHVALUES, XX,
        },
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HScan<'a> {
    Match(&'a str),
    Count(usize),
    NoValues,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HRangeByLex {
    Limit(usize, usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HExpireCondition {
    Nx,
    Xx,
    Gt,
    Lt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HSetExOption {
    Ex(u64),
    Px(u64),
    ExAt(u64),
    PxAt(u64),
    KeepTtl,
    Fnx,
    Fxx,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HGetExOption {
    Ex(u64),
    Px(u64),
    ExAt(u64),
    PxAt(u64),
    Persist,
}

fn apply_hrangebylex_option(cmd: Cmd, conf: &HRangeByLex) -> Cmd {
    match conf {
        HRangeByLex::Limit(off, cnt) => cmd.arg(LIMIT).arg_int(*off).arg_int(*cnt),
    }
}

fn apply_hexpire_cond(cmd: Cmd, conf: &HExpireCondition) -> Cmd {
    match conf {
        HExpireCondition::Nx => cmd.arg(NX),
        HExpireCondition::Xx => cmd.arg(XX),
        HExpireCondition::Gt => cmd.arg(GT),
        HExpireCondition::Lt => cmd.arg(LT),
    }
}

fn apply_hsetex_option(cmd: Cmd, conf: &HSetExOption) -> Cmd {
    match conf {
        HSetExOption::Ex(s) => cmd.arg(EX).arg_int(*s),
        HSetExOption::Px(ms) => cmd.arg(PX).arg_int(*ms),
        HSetExOption::ExAt(ts) => cmd.arg(EXAT).arg_int(*ts),
        HSetExOption::PxAt(ts_ms) => cmd.arg(PXAT).arg_int(*ts_ms),
        HSetExOption::KeepTtl => cmd.arg(KEEPTTL),
        HSetExOption::Fnx => cmd.arg(FNX),
        HSetExOption::Fxx => cmd.arg(FXX),
    }
}

fn apply_hgetex_option(cmd: Cmd, conf: &HGetExOption) -> Cmd {
    match conf {
        HGetExOption::Ex(s) => cmd.arg(EX).arg_int(*s),
        HGetExOption::Px(ms) => cmd.arg(PX).arg_int(*ms),
        HGetExOption::ExAt(ts) => cmd.arg(EXAT).arg_int(*ts),
        HGetExOption::PxAt(ts_ms) => cmd.arg(PXAT).arg_int(*ts_ms),
        HGetExOption::Persist => cmd.arg(PERSIST),
    }
}

fn build_hexpire_cmd<F: AsRef<[u8]>>(
    name: &'static str,
    key: impl AsRef<[u8]>,
    ttl_val: u64,
    fields: &[F],
    conf_li: &[HExpireCondition],
) -> Cmd {
    let mut cmd = Cmd::new(name).arg_bytes(key).arg_int(ttl_val);
    for conf in conf_li {
        cmd = apply_hexpire_cond(cmd, conf);
    }
    cmd.arg(FIELDS).arg_int(fields.len()).args_slice(fields)
}

fn build_hfield_cmd<F: AsRef<[u8]>>(
    name: &'static str,
    key: impl AsRef<[u8]>,
    fields: &[F],
) -> Cmd {
    Cmd::new(name)
        .arg_bytes(key)
        .arg(FIELDS)
        .arg_int(fields.len())
        .args_slice(fields)
}

#[inline]
fn parse_first_positive(res: &[i64]) -> bool {
    res.first().copied().unwrap_or(-2) > 0
}

#[inline]
fn parse_first_or_default(res: &[i64], default: i64) -> i64 {
    res.first().copied().unwrap_or(default)
}

impl Client {
    pub async fn hget<T: FromValue>(
        &self,
        key: impl AsRef<[u8]>,
        field: impl AsRef<[u8]>,
    ) -> Result<Option<T>> {
        self.execute_cmd(Cmd::new(HGET).arg_bytes(key).arg_bytes(field))
            .await
    }

    pub async fn hset(
        &self,
        key: impl AsRef<[u8]>,
        field: impl AsRef<[u8]>,
        val: impl AsRef<[u8]>,
    ) -> Result<u64> {
        self.execute_cmd(
            Cmd::new(HSET)
                .arg_bytes(key)
                .arg_bytes(field)
                .arg_bytes(val),
        )
        .await
    }

    pub async fn hset_multiple<F: AsRef<[u8]>, V: AsRef<[u8]>>(
        &self,
        key: impl AsRef<[u8]>,
        fields: &[(F, V)],
    ) -> Result<u64> {
        self.execute_cmd(Cmd::new(HSET).arg_bytes(key).args_pairs(fields))
            .await
    }

    pub async fn hsetnx(
        &self,
        key: impl AsRef<[u8]>,
        field: impl AsRef<[u8]>,
        val: impl AsRef<[u8]>,
    ) -> Result<bool> {
        self.execute_cmd(
            Cmd::new(HSETNX)
                .arg_bytes(key)
                .arg_bytes(field)
                .arg_bytes(val),
        )
        .await
    }

    pub async fn hdel<F: AsRef<[u8]>>(&self, key: impl AsRef<[u8]>, fields: &[F]) -> Result<u64> {
        self.execute_cmd(Cmd::new(HDEL).arg_bytes(key).args_slice(fields))
            .await
    }

    pub async fn hexists(&self, key: impl AsRef<[u8]>, field: impl AsRef<[u8]>) -> Result<bool> {
        self.execute_cmd(Cmd::new(HEXISTS).arg_bytes(key).arg_bytes(field))
            .await
    }

    pub async fn hlen(&self, key: impl AsRef<[u8]>) -> Result<u64> {
        self.execute_cmd(Cmd::new(HLEN).arg_bytes(key)).await
    }

    /// Kvrocks specific: HLEN key [APPROX | REPAIR]
    pub async fn hlen_opt(&self, key: impl AsRef<[u8]>, mode: Option<&str>) -> Result<u64> {
        self.execute_cmd(Cmd::new(HLEN).arg_bytes(key).arg_opt_bytes(mode))
            .await
    }

    pub async fn hstrlen(&self, key: impl AsRef<[u8]>, field: impl AsRef<[u8]>) -> Result<u64> {
        self.execute_cmd(Cmd::new(HSTRLEN).arg_bytes(key).arg_bytes(field))
            .await
    }

    pub async fn hmget<T: FromValue, F: AsRef<[u8]>>(
        &self,
        key: impl AsRef<[u8]>,
        fields: &[F],
    ) -> Result<Vec<Option<T>>> {
        let cmd = Cmd::new(HMGET).arg_bytes(key).args_slice(fields);
        self.execute_cmd(cmd).await
    }

    pub async fn hmset<F: AsRef<[u8]>, V: AsRef<[u8]>>(
        &self,
        key: impl AsRef<[u8]>,
        fields: &[(F, V)],
    ) -> Result<()> {
        self.execute_cmd(Cmd::new(HMSET).arg_bytes(key).args_pairs(fields))
            .await
    }

    pub async fn hkeys(&self, key: impl AsRef<[u8]>) -> Result<Vec<String>> {
        self.execute_cmd(Cmd::new(HKEYS).arg_bytes(key)).await
    }

    pub async fn hvals<T: FromValue>(&self, key: impl AsRef<[u8]>) -> Result<Vec<T>> {
        self.execute_cmd(Cmd::new(HVALS).arg_bytes(key)).await
    }

    pub async fn hgetall<T: FromValue>(
        &self,
        key: impl AsRef<[u8]>,
    ) -> Result<RapidHashMap<String, T>> {
        self.execute_cmd(Cmd::new(HGETALL).arg_bytes(key)).await
    }

    pub async fn hincrby(
        &self,
        key: impl AsRef<[u8]>,
        field: impl AsRef<[u8]>,
        delta: i64,
    ) -> Result<i64> {
        self.execute_cmd(
            Cmd::new(HINCRBY)
                .arg_bytes(key)
                .arg_bytes(field)
                .arg_int(delta),
        )
        .await
    }

    pub async fn hincrbyfloat(
        &self,
        key: impl AsRef<[u8]>,
        field: impl AsRef<[u8]>,
        delta: f64,
    ) -> Result<f64> {
        self.execute_cmd(
            Cmd::new(HINCRBYFLOAT)
                .arg_bytes(key)
                .arg_bytes(field)
                .arg_float(delta),
        )
        .await
    }

    pub async fn hrandfield<T: FromValue>(
        &self,
        key: impl AsRef<[u8]>,
        count: Option<i64>,
    ) -> Result<Vec<T>> {
        self.exec_single_or_array(Cmd::new(HRANDFIELD).arg_bytes(key).arg_opt_int(count))
            .await
    }

    pub async fn hrandfield_withvalues(
        &self,
        key: impl AsRef<[u8]>,
        count: i64,
    ) -> Result<Vec<(String, String)>> {
        let cmd = Cmd::new(HRANDFIELD)
            .arg_bytes(key)
            .arg_int(count)
            .arg(WITHVALUES);
        self.exec_pair_array(cmd).await
    }

    pub async fn hscan(
        &self,
        key: impl AsRef<[u8]>,
        cursor: u64,
        conf_li: impl AsRef<[HScan<'_>]>,
    ) -> Result<(u64, Vec<(String, String)>)> {
        let mut r#match = None;
        let mut count = None;
        let mut no_values = false;
        for conf in conf_li.as_ref() {
            match conf {
                HScan::Match(p) => r#match = Some(*p),
                HScan::Count(c) => count = Some(*c),
                HScan::NoValues => no_values = true,
            }
        }
        let cmd = apply_scan_opts(
            Cmd::new(HSCAN).arg_bytes(key).arg_int(cursor),
            r#match,
            count,
            no_values,
        );
        self.exec_scan_pair(cmd).await
    }

    pub async fn hrangebylex(
        &self,
        key: impl AsRef<[u8]>,
        min: impl AsRef<[u8]>,
        max: impl AsRef<[u8]>,
        conf_li: impl AsRef<[HRangeByLex]>,
    ) -> Result<Vec<(String, String)>> {
        let mut cmd = Cmd::new(HRANGEBYLEX)
            .arg_bytes(key)
            .arg_bytes(min)
            .arg_bytes(max);
        for conf in conf_li.as_ref() {
            cmd = apply_hrangebylex_option(cmd, conf);
        }
        self.exec_pair_array(cmd).await
    }

    pub async fn hsetex<F: AsRef<[u8]>, V: AsRef<[u8]>>(
        &self,
        key: impl AsRef<[u8]>,
        fields: &[(F, V)],
        conf_li: impl AsRef<[HSetExOption]>,
    ) -> Result<u64> {
        let mut cmd = Cmd::new(HSETEX).arg_bytes(key);
        for conf in conf_li.as_ref() {
            cmd = apply_hsetex_option(cmd, conf);
        }
        cmd = cmd.arg(FIELDS).arg_int(fields.len()).args_pairs(fields);
        self.execute_cmd(cmd).await
    }

    pub async fn hgetex<T: FromValue, F: AsRef<[u8]>>(
        &self,
        key: impl AsRef<[u8]>,
        fields: &[F],
        conf_li: impl AsRef<[HGetExOption]>,
    ) -> Result<Vec<Option<T>>> {
        let mut cmd = Cmd::new(HGETEX).arg_bytes(key);
        for conf in conf_li.as_ref() {
            cmd = apply_hgetex_option(cmd, conf);
        }
        cmd = cmd.arg(FIELDS).arg_int(fields.len()).args_slice(fields);
        self.execute_cmd(cmd).await
    }

    pub async fn hexpire_one(
        &self,
        key: impl AsRef<[u8]>,
        field: impl AsRef<[u8]>,
        seconds: u64,
        conf_li: impl AsRef<[HExpireCondition]>,
    ) -> Result<bool> {
        let res = self.hexpire(key, seconds, &[field], conf_li).await?;
        Ok(parse_first_positive(&res))
    }

    pub async fn hpexpire_one(
        &self,
        key: impl AsRef<[u8]>,
        field: impl AsRef<[u8]>,
        milliseconds: u64,
        conf_li: impl AsRef<[HExpireCondition]>,
    ) -> Result<bool> {
        let res = self.hpexpire(key, milliseconds, &[field], conf_li).await?;
        Ok(parse_first_positive(&res))
    }

    pub async fn hexpireat_one(
        &self,
        key: impl AsRef<[u8]>,
        field: impl AsRef<[u8]>,
        timestamp: u64,
        conf_li: impl AsRef<[HExpireCondition]>,
    ) -> Result<bool> {
        let res = self.hexpireat(key, timestamp, &[field], conf_li).await?;
        Ok(parse_first_positive(&res))
    }

    pub async fn hpexpireat_one(
        &self,
        key: impl AsRef<[u8]>,
        field: impl AsRef<[u8]>,
        timestamp_ms: u64,
        conf_li: impl AsRef<[HExpireCondition]>,
    ) -> Result<bool> {
        let res = self
            .hpexpireat(key, timestamp_ms, &[field], conf_li)
            .await?;
        Ok(parse_first_positive(&res))
    }

    pub async fn httl_one(&self, key: impl AsRef<[u8]>, field: impl AsRef<[u8]>) -> Result<i64> {
        let res = self.httl(key, &[field]).await?;
        Ok(parse_first_or_default(&res, -2))
    }

    pub async fn hpttl_one(&self, key: impl AsRef<[u8]>, field: impl AsRef<[u8]>) -> Result<i64> {
        let res = self.hpttl(key, &[field]).await?;
        Ok(parse_first_or_default(&res, -2))
    }

    pub async fn hexpiretime_one(
        &self,
        key: impl AsRef<[u8]>,
        field: impl AsRef<[u8]>,
    ) -> Result<i64> {
        let res = self.hexpiretime(key, &[field]).await?;
        Ok(parse_first_or_default(&res, -2))
    }

    pub async fn hpexpiretime_one(
        &self,
        key: impl AsRef<[u8]>,
        field: impl AsRef<[u8]>,
    ) -> Result<i64> {
        let res = self.hpexpiretime(key, &[field]).await?;
        Ok(parse_first_or_default(&res, -2))
    }

    pub async fn hpersist_one(
        &self,
        key: impl AsRef<[u8]>,
        field: impl AsRef<[u8]>,
    ) -> Result<bool> {
        let res = self.hpersist(key, &[field]).await?;
        Ok(parse_first_positive(&res))
    }

    pub async fn hsetexpire<F: AsRef<[u8]>, V: AsRef<[u8]>>(
        &self,
        key: impl AsRef<[u8]>,
        ttl: u64,
        fields: &[(F, V)],
    ) -> Result<()> {
        self.execute_cmd(
            Cmd::new(HSETEXPIRE)
                .arg_bytes(key)
                .arg_int(ttl)
                .args_pairs(fields),
        )
        .await
    }

    pub async fn hexpire<F: AsRef<[u8]>>(
        &self,
        key: impl AsRef<[u8]>,
        seconds: u64,
        fields: &[F],
        conf_li: impl AsRef<[HExpireCondition]>,
    ) -> Result<Vec<i64>> {
        let cmd = build_hexpire_cmd(HEXPIRE, key, seconds, fields, conf_li.as_ref());
        self.execute_cmd(cmd).await
    }

    pub async fn hpexpire<F: AsRef<[u8]>>(
        &self,
        key: impl AsRef<[u8]>,
        milliseconds: u64,
        fields: &[F],
        conf_li: impl AsRef<[HExpireCondition]>,
    ) -> Result<Vec<i64>> {
        let cmd = build_hexpire_cmd(HPEXPIRE, key, milliseconds, fields, conf_li.as_ref());
        self.execute_cmd(cmd).await
    }

    pub async fn hexpireat<F: AsRef<[u8]>>(
        &self,
        key: impl AsRef<[u8]>,
        timestamp: u64,
        fields: &[F],
        conf_li: impl AsRef<[HExpireCondition]>,
    ) -> Result<Vec<i64>> {
        let cmd = build_hexpire_cmd(HEXPIREAT, key, timestamp, fields, conf_li.as_ref());
        self.execute_cmd(cmd).await
    }

    pub async fn hpexpireat<F: AsRef<[u8]>>(
        &self,
        key: impl AsRef<[u8]>,
        timestamp_ms: u64,
        fields: &[F],
        conf_li: impl AsRef<[HExpireCondition]>,
    ) -> Result<Vec<i64>> {
        let cmd = build_hexpire_cmd(HPEXPIREAT, key, timestamp_ms, fields, conf_li.as_ref());
        self.execute_cmd(cmd).await
    }

    pub async fn hpersist<F: AsRef<[u8]>>(
        &self,
        key: impl AsRef<[u8]>,
        fields: &[F],
    ) -> Result<Vec<i64>> {
        self.execute_cmd(build_hfield_cmd(HPERSIST, key, fields))
            .await
    }

    pub async fn httl<F: AsRef<[u8]>>(
        &self,
        key: impl AsRef<[u8]>,
        fields: &[F],
    ) -> Result<Vec<i64>> {
        self.execute_cmd(build_hfield_cmd(HTTL, key, fields)).await
    }

    pub async fn hpttl<F: AsRef<[u8]>>(
        &self,
        key: impl AsRef<[u8]>,
        fields: &[F],
    ) -> Result<Vec<i64>> {
        self.execute_cmd(build_hfield_cmd(HPTTL, key, fields)).await
    }

    pub async fn hexpiretime<F: AsRef<[u8]>>(
        &self,
        key: impl AsRef<[u8]>,
        fields: &[F],
    ) -> Result<Vec<i64>> {
        self.execute_cmd(build_hfield_cmd(HEXPIRETIME, key, fields))
            .await
    }

    pub async fn hpexpiretime<F: AsRef<[u8]>>(
        &self,
        key: impl AsRef<[u8]>,
        fields: &[F],
    ) -> Result<Vec<i64>> {
        self.execute_cmd(build_hfield_cmd(HPEXPIRETIME, key, fields))
            .await
    }
}
