use crate::{
    client::{Client, helper::apply_scan_opts},
    error::Result,
    resp3::{
        Cmd, FromValue, Value,
        constants::{
            AGGREGATE, BYLEX, BYSCORE, BZMPOP, BZPOPMAX, BZPOPMIN, CH, COUNT, GT, INCR, LIMIT, LT,
            MAX, MIN, NX, REV, SUM, WEIGHTS, WITHSCORES, XX, ZADD, ZCARD, ZCOUNT, ZDIFF,
            ZDIFFSTORE, ZINCRBY, ZINTER, ZINTERCARD, ZINTERSTORE, ZLEXCOUNT, ZMPOP, ZMSCORE,
            ZPOPMAX, ZPOPMIN, ZRANDMEMBER, ZRANGE, ZRANGEBYLEX, ZRANGEBYSCORE, ZRANGESTORE, ZRANK,
            ZREM, ZREMRANGEBYLEX, ZREMRANGEBYRANK, ZREMRANGEBYSCORE, ZREVRANGE, ZREVRANGEBYLEX,
            ZREVRANGEBYSCORE, ZREVRANK, ZSCAN, ZSCORE, ZUNION, ZUNIONSTORE,
        },
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZAddOption {
    Nx,
    Xx,
    Gt,
    Lt,
    Ch,
    Incr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZRangeByScore {
    Limit(usize, usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ZScan<'a> {
    Match(&'a str),
    Count(usize),
    NoValues,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Aggregate {
    Sum,
    Min,
    Max,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopDirection {
    Min,
    Max,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ZRangeStore {
    ByScore,
    ByLex,
    Rev,
    Limit(usize, usize),
}

fn apply_zrangestore_opt(cmd: Cmd, conf: &ZRangeStore) -> Cmd {
    match conf {
        ZRangeStore::ByScore => cmd.arg(BYSCORE),
        ZRangeStore::ByLex => cmd.arg(BYLEX),
        ZRangeStore::Rev => cmd.arg(REV),
        ZRangeStore::Limit(off, cnt) => cmd.arg(LIMIT).arg_int(*off).arg_int(*cnt),
    }
}

fn build_zrange_lex_score_cmd(
    name: &'static str,
    key: impl AsRef<[u8]>,
    arg1: &str,
    arg2: &str,
    conf_li: &[ZRangeByScore],
) -> Cmd {
    let mut cmd = Cmd::new(name)
        .arg_bytes(key)
        .arg_bytes(arg1)
        .arg_bytes(arg2);
    for conf in conf_li {
        match conf {
            ZRangeByScore::Limit(off, cnt) => {
                cmd = cmd.arg(LIMIT).arg_int(*off).arg_int(*cnt);
            }
        }
    }
    cmd
}

fn build_zunion_inter_cmd<K: AsRef<[u8]>>(
    name: &'static str,
    dst: Option<impl AsRef<[u8]>>,
    keys: &[K],
    weights: Option<&[f64]>,
    aggregate: Option<Aggregate>,
) -> Cmd {
    let mut cmd = Cmd::new(name);
    if let Some(d) = dst {
        cmd = cmd.arg_bytes(d);
    }
    cmd = cmd.arg_int(keys.len()).args_slice(keys);
    if let Some(ws) = weights {
        cmd = cmd.arg(WEIGHTS).args_floats(ws);
    }
    if let Some(agg) = aggregate {
        cmd = cmd.arg(AGGREGATE).arg(match agg {
            Aggregate::Sum => SUM,
            Aggregate::Min => MIN,
            Aggregate::Max => MAX,
        });
    }
    cmd
}

fn build_zpop_cmd(name: &'static str, key: impl AsRef<[u8]>, count: Option<usize>) -> Cmd {
    Cmd::new(name).arg_bytes(key).arg_opt_int(count)
}

fn build_bzpop_cmd<K: AsRef<[u8]>>(name: &'static str, keys: &[K], timeout_sec: f64) -> Cmd {
    Cmd::new(name).args_slice(keys).arg_float(timeout_sec)
}

fn build_zmpop_cmd<K: AsRef<[u8]>>(
    name: &'static str,
    timeout: Option<f64>,
    keys: &[K],
    dir: PopDirection,
    count: Option<usize>,
) -> Cmd {
    Cmd::new(name)
        .arg_opt_float(timeout)
        .arg_int(keys.len())
        .args_slice(keys)
        .arg(match dir {
            PopDirection::Min => MIN,
            PopDirection::Max => MAX,
        })
        .arg_keyword_opt_int(COUNT, count)
}

fn apply_zadd_opt(cmd: Cmd, conf: &ZAddOption) -> Cmd {
    match conf {
        ZAddOption::Nx => cmd.arg(NX),
        ZAddOption::Xx => cmd.arg(XX),
        ZAddOption::Gt => cmd.arg(GT),
        ZAddOption::Lt => cmd.arg(LT),
        ZAddOption::Ch => cmd.arg(CH),
        ZAddOption::Incr => cmd.arg(INCR),
    }
}

impl Client {
    pub async fn zadd<M: AsRef<[u8]>>(
        &self,
        key: impl AsRef<[u8]>,
        score_members: &[(f64, M)],
    ) -> Result<u64> {
        self.zadd_opt(key, score_members, &[]).await
    }

    pub async fn zadd_opt<M: AsRef<[u8]>>(
        &self,
        key: impl AsRef<[u8]>,
        score_members: &[(f64, M)],
        conf_li: impl AsRef<[ZAddOption]>,
    ) -> Result<u64> {
        let mut cmd = Cmd::new(ZADD).arg_bytes(key);
        for conf in conf_li.as_ref() {
            cmd = apply_zadd_opt(cmd, conf);
        }
        for (score, member) in score_members {
            cmd = cmd.arg_float(*score).arg_bytes(member);
        }
        self.execute_cmd(cmd).await
    }

    pub async fn zadd_incr(
        &self,
        key: impl AsRef<[u8]>,
        score: f64,
        member: impl AsRef<[u8]>,
        conf_li: impl AsRef<[ZAddOption]>,
    ) -> Result<Option<f64>> {
        let mut cmd = Cmd::new(ZADD).arg_bytes(key).arg(INCR);
        for conf in conf_li.as_ref() {
            if *conf != ZAddOption::Incr {
                cmd = apply_zadd_opt(cmd, conf);
            }
        }
        cmd = cmd.arg_float(score).arg_bytes(member);
        self.execute_cmd(cmd).await
    }

    pub async fn zrem<M: AsRef<[u8]>>(&self, key: impl AsRef<[u8]>, members: &[M]) -> Result<u64> {
        self.execute_cmd(Cmd::new(ZREM).arg_bytes(key).args_slice(members))
            .await
    }

    pub async fn zscore(
        &self,
        key: impl AsRef<[u8]>,
        member: impl AsRef<[u8]>,
    ) -> Result<Option<f64>> {
        self.execute_cmd(Cmd::new(ZSCORE).arg_bytes(key).arg_bytes(member))
            .await
    }

    pub async fn zmscore<M: AsRef<[u8]>>(
        &self,
        key: impl AsRef<[u8]>,
        members: &[M],
    ) -> Result<Vec<Option<f64>>> {
        let cmd = Cmd::new(ZMSCORE).arg_bytes(key).args_slice(members);
        self.execute_cmd(cmd).await
    }

    pub async fn zincrby(
        &self,
        key: impl AsRef<[u8]>,
        delta: f64,
        member: impl AsRef<[u8]>,
    ) -> Result<f64> {
        self.execute_cmd(
            Cmd::new(ZINCRBY)
                .arg_bytes(key)
                .arg_float(delta)
                .arg_bytes(member),
        )
        .await
    }

    pub async fn zcard(&self, key: impl AsRef<[u8]>) -> Result<u64> {
        self.execute_cmd(Cmd::new(ZCARD).arg_bytes(key)).await
    }

    pub async fn zcount(&self, key: impl AsRef<[u8]>, min: &str, max: &str) -> Result<u64> {
        self.execute_cmd(
            Cmd::new(ZCOUNT)
                .arg_bytes(key)
                .arg_bytes(min)
                .arg_bytes(max),
        )
        .await
    }

    pub async fn zlexcount(&self, key: impl AsRef<[u8]>, min: &str, max: &str) -> Result<u64> {
        self.execute_cmd(
            Cmd::new(ZLEXCOUNT)
                .arg_bytes(key)
                .arg_bytes(min)
                .arg_bytes(max),
        )
        .await
    }

    pub async fn zrange<T: FromValue>(
        &self,
        key: impl AsRef<[u8]>,
        start: i64,
        stop: i64,
    ) -> Result<Vec<T>> {
        let cmd = Cmd::new(ZRANGE).arg_bytes(key).arg_int(start).arg_int(stop);
        self.execute_cmd(cmd).await
    }

    pub async fn zrange_withscores(
        &self,
        key: impl AsRef<[u8]>,
        start: i64,
        stop: i64,
    ) -> Result<Vec<(String, f64)>> {
        let cmd = Cmd::new(ZRANGE)
            .arg_bytes(key)
            .arg_int(start)
            .arg_int(stop)
            .arg(WITHSCORES);
        self.exec_pair_array(cmd).await
    }

    pub async fn zrevrange<T: FromValue>(
        &self,
        key: impl AsRef<[u8]>,
        start: i64,
        stop: i64,
    ) -> Result<Vec<T>> {
        let cmd = Cmd::new(ZREVRANGE)
            .arg_bytes(key)
            .arg_int(start)
            .arg_int(stop);
        self.execute_cmd(cmd).await
    }

    pub async fn zrevrange_withscores(
        &self,
        key: impl AsRef<[u8]>,
        start: i64,
        stop: i64,
    ) -> Result<Vec<(String, f64)>> {
        let cmd = Cmd::new(ZREVRANGE)
            .arg_bytes(key)
            .arg_int(start)
            .arg_int(stop)
            .arg(WITHSCORES);
        self.exec_pair_array(cmd).await
    }

    pub async fn zrangebyscore<T: FromValue>(
        &self,
        key: impl AsRef<[u8]>,
        min: &str,
        max: &str,
        conf_li: impl AsRef<[ZRangeByScore]>,
    ) -> Result<Vec<T>> {
        let cmd = build_zrange_lex_score_cmd(ZRANGEBYSCORE, key, min, max, conf_li.as_ref());
        self.execute_cmd(cmd).await
    }

    pub async fn zrangebylex<T: FromValue>(
        &self,
        key: impl AsRef<[u8]>,
        min: &str,
        max: &str,
        conf_li: impl AsRef<[ZRangeByScore]>,
    ) -> Result<Vec<T>> {
        let cmd = build_zrange_lex_score_cmd(ZRANGEBYLEX, key, min, max, conf_li.as_ref());
        self.execute_cmd(cmd).await
    }

    pub async fn zrevrangebyscore<T: FromValue>(
        &self,
        key: impl AsRef<[u8]>,
        max: &str,
        min: &str,
        conf_li: impl AsRef<[ZRangeByScore]>,
    ) -> Result<Vec<T>> {
        let cmd = build_zrange_lex_score_cmd(ZREVRANGEBYSCORE, key, max, min, conf_li.as_ref());
        self.execute_cmd(cmd).await
    }

    pub async fn zrevrangebylex<T: FromValue>(
        &self,
        key: impl AsRef<[u8]>,
        max: &str,
        min: &str,
        conf_li: impl AsRef<[ZRangeByScore]>,
    ) -> Result<Vec<T>> {
        let cmd = build_zrange_lex_score_cmd(ZREVRANGEBYLEX, key, max, min, conf_li.as_ref());
        self.execute_cmd(cmd).await
    }

    pub async fn zrank(
        &self,
        key: impl AsRef<[u8]>,
        member: impl AsRef<[u8]>,
    ) -> Result<Option<u64>> {
        self.execute_cmd(Cmd::new(ZRANK).arg_bytes(key).arg_bytes(member))
            .await
    }

    pub async fn zrevrank(
        &self,
        key: impl AsRef<[u8]>,
        member: impl AsRef<[u8]>,
    ) -> Result<Option<u64>> {
        self.execute_cmd(Cmd::new(ZREVRANK).arg_bytes(key).arg_bytes(member))
            .await
    }

    pub async fn zremrangebyrank(
        &self,
        key: impl AsRef<[u8]>,
        start: i64,
        stop: i64,
    ) -> Result<u64> {
        let cmd = Cmd::new(ZREMRANGEBYRANK)
            .arg_bytes(key)
            .arg_int(start)
            .arg_int(stop);
        self.execute_cmd(cmd).await
    }

    pub async fn zremrangebyscore(
        &self,
        key: impl AsRef<[u8]>,
        min: &str,
        max: &str,
    ) -> Result<u64> {
        self.execute_cmd(
            Cmd::new(ZREMRANGEBYSCORE)
                .arg_bytes(key)
                .arg_bytes(min)
                .arg_bytes(max),
        )
        .await
    }

    pub async fn zremrangebylex(&self, key: impl AsRef<[u8]>, min: &str, max: &str) -> Result<u64> {
        self.execute_cmd(
            Cmd::new(ZREMRANGEBYLEX)
                .arg_bytes(key)
                .arg_bytes(min)
                .arg_bytes(max),
        )
        .await
    }

    pub async fn zpopmin<T: FromValue>(
        &self,
        key: impl AsRef<[u8]>,
        count: Option<usize>,
    ) -> Result<Vec<(T, f64)>> {
        self.exec_pair_array(build_zpop_cmd(ZPOPMIN, key, count))
            .await
    }

    pub async fn zpopmax<T: FromValue>(
        &self,
        key: impl AsRef<[u8]>,
        count: Option<usize>,
    ) -> Result<Vec<(T, f64)>> {
        self.exec_pair_array(build_zpop_cmd(ZPOPMAX, key, count))
            .await
    }

    pub async fn bzpopmin<T: FromValue, K: AsRef<[u8]>>(
        &self,
        keys: &[K],
        timeout_sec: f64,
    ) -> Result<Option<(String, T, f64)>> {
        self.execute_cmd(build_bzpop_cmd(BZPOPMIN, keys, timeout_sec))
            .await
    }

    pub async fn bzpopmax<T: FromValue, K: AsRef<[u8]>>(
        &self,
        keys: &[K],
        timeout_sec: f64,
    ) -> Result<Option<(String, T, f64)>> {
        self.execute_cmd(build_bzpop_cmd(BZPOPMAX, keys, timeout_sec))
            .await
    }

    pub async fn zmpop<K: AsRef<[u8]>>(
        &self,
        keys: &[K],
        dir: PopDirection,
        count: Option<usize>,
    ) -> Result<Value> {
        let cmd = build_zmpop_cmd(ZMPOP, None, keys, dir, count);
        self.execute(cmd).await
    }

    pub async fn bzmpop<K: AsRef<[u8]>>(
        &self,
        timeout_sec: f64,
        keys: &[K],
        dir: PopDirection,
        count: Option<usize>,
    ) -> Result<Value> {
        let cmd = build_zmpop_cmd(BZMPOP, Some(timeout_sec), keys, dir, count);
        self.execute(cmd).await
    }

    pub async fn zrangestore(
        &self,
        dst: impl AsRef<[u8]>,
        src: impl AsRef<[u8]>,
        min: &str,
        max: &str,
        conf_li: impl AsRef<[ZRangeStore]>,
    ) -> Result<u64> {
        let mut cmd = Cmd::new(ZRANGESTORE)
            .arg_bytes(dst)
            .arg_bytes(src)
            .arg_bytes(min)
            .arg_bytes(max);
        for conf in conf_li.as_ref() {
            cmd = apply_zrangestore_opt(cmd, conf);
        }
        self.execute_cmd(cmd).await
    }

    pub async fn zunionstore<K: AsRef<[u8]>>(
        &self,
        dst: impl AsRef<[u8]>,
        keys: &[K],
        weights: Option<&[f64]>,
        aggregate: Option<Aggregate>,
    ) -> Result<u64> {
        let cmd = build_zunion_inter_cmd(ZUNIONSTORE, Some(dst), keys, weights, aggregate);
        self.execute_cmd(cmd).await
    }

    pub async fn zunion<T: FromValue, K: AsRef<[u8]>>(
        &self,
        keys: &[K],
        weights: Option<&[f64]>,
        aggregate: Option<Aggregate>,
    ) -> Result<Vec<T>> {
        let cmd = build_zunion_inter_cmd::<K>(ZUNION, None::<&[u8]>, keys, weights, aggregate);
        self.execute_cmd(cmd).await
    }

    pub async fn zinterstore<K: AsRef<[u8]>>(
        &self,
        dst: impl AsRef<[u8]>,
        keys: &[K],
        weights: Option<&[f64]>,
        aggregate: Option<Aggregate>,
    ) -> Result<u64> {
        let cmd = build_zunion_inter_cmd(ZINTERSTORE, Some(dst), keys, weights, aggregate);
        self.execute_cmd(cmd).await
    }

    pub async fn zinter<T: FromValue, K: AsRef<[u8]>>(
        &self,
        keys: &[K],
        weights: Option<&[f64]>,
        aggregate: Option<Aggregate>,
    ) -> Result<Vec<T>> {
        let cmd = build_zunion_inter_cmd::<K>(ZINTER, None::<&[u8]>, keys, weights, aggregate);
        self.execute_cmd(cmd).await
    }

    pub async fn zintercard<K: AsRef<[u8]>>(
        &self,
        keys: &[K],
        limit: Option<usize>,
    ) -> Result<u64> {
        let cmd = Cmd::new(ZINTERCARD)
            .arg_int(keys.len())
            .args_slice(keys)
            .arg_keyword_opt_int(LIMIT, limit);
        self.execute_cmd(cmd).await
    }

    pub async fn zdiff<T: FromValue, K: AsRef<[u8]>>(&self, keys: &[K]) -> Result<Vec<T>> {
        self.execute_cmd(Cmd::new(ZDIFF).arg_int(keys.len()).args_slice(keys))
            .await
    }

    pub async fn zdiffstore<K: AsRef<[u8]>>(
        &self,
        dst: impl AsRef<[u8]>,
        keys: &[K],
    ) -> Result<u64> {
        self.execute_cmd(
            Cmd::new(ZDIFFSTORE)
                .arg_bytes(dst)
                .arg_int(keys.len())
                .args_slice(keys),
        )
        .await
    }

    pub async fn zrandmember<T: FromValue>(
        &self,
        key: impl AsRef<[u8]>,
        count: Option<i64>,
    ) -> Result<Vec<T>> {
        self.exec_single_or_array(Cmd::new(ZRANDMEMBER).arg_bytes(key).arg_opt_int(count))
            .await
    }

    pub async fn zscan(
        &self,
        key: impl AsRef<[u8]>,
        cursor: u64,
        conf_li: impl AsRef<[ZScan<'_>]>,
    ) -> Result<(u64, Vec<(String, f64)>)> {
        let mut r#match = None;
        let mut count = None;
        let mut no_values = false;
        for conf in conf_li.as_ref() {
            match conf {
                ZScan::Match(p) => r#match = Some(*p),
                ZScan::Count(c) => count = Some(*c),
                ZScan::NoValues => no_values = true,
            }
        }
        let cmd = apply_scan_opts(
            Cmd::new(ZSCAN).arg_bytes(key).arg_int(cursor),
            r#match,
            count,
            no_values,
        );
        self.exec_scan_pair(cmd).await
    }
}
