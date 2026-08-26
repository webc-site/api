use crate::{
    client::Client,
    error::Result,
    resp3::{
        Cmd, Value,
        constants::{
            AGGREGATION, CHUNK_SIZE, COUNT, DUPLICATE_POLICY, FILTER, LABELS, LATEST, RETENTION,
            TIMESTAMP, TS_ADD, TS_ALTER, TS_CREATE, TS_CREATERULE, TS_DECRBY, TS_DEL,
            TS_DELETERULE, TS_GET, TS_INCRBY, TS_INFO, TS_MADD, TS_MGET, TS_MRANGE, TS_MREVRANGE,
            TS_QUERYINDEX, TS_RANGE, TS_REVRANGE, WITHLABELS,
        },
    },
};

#[derive(Debug, Clone, PartialEq)]
pub enum TsCreate<'a> {
    Retention(u64),
    ChunkSize(u64),
    DuplicatePolicy(&'a str),
    Labels(&'a [(&'a str, &'a str)]),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TsAlter<'a> {
    Retention(u64),
    ChunkSize(u64),
    DuplicatePolicy(&'a str),
    Labels(&'a [(&'a str, &'a str)]),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TsAdd<'a> {
    Retention(u64),
    ChunkSize(u64),
    OnDuplicate(&'a str),
    Labels(&'a [(&'a str, &'a str)]),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TsRange {
    Count(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TsMGet {
    Latest,
    WithLabels,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TsMRange {
    Latest,
    WithLabels,
    Count(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TsIncrBy<'a> {
    Timestamp(&'a str),
    Retention(u64),
    ChunkSize(u64),
    Labels(&'a [(&'a str, &'a str)]),
}

fn apply_ts_labels(mut cmd: Cmd, labels: &[(&str, &str)]) -> Cmd {
    if !labels.is_empty() {
        cmd = cmd.arg(LABELS);
        for (k, v) in labels {
            cmd = cmd.arg_bytes(k).arg_bytes(v);
        }
    }
    cmd
}

fn apply_ts_create_alter(
    mut cmd: Cmd,
    retention: Option<u64>,
    chunk_size: Option<u64>,
    duplicate_policy: Option<&str>,
    labels: &[(&str, &str)],
) -> Cmd {
    cmd = cmd
        .arg_keyword_opt_int(RETENTION, retention)
        .arg_keyword_opt_int(CHUNK_SIZE, chunk_size)
        .arg_keyword_opt_bytes(DUPLICATE_POLICY, duplicate_policy);
    apply_ts_labels(cmd, labels)
}

fn build_ts_range_cmd(
    name: &'static str,
    key: impl AsRef<[u8]>,
    from_ts: &str,
    to_ts: &str,
    conf_li: &[TsRange],
) -> Cmd {
    let mut count = None;
    for conf in conf_li {
        match conf {
            TsRange::Count(c) => count = Some(*c),
        }
    }
    Cmd::new(name)
        .arg_bytes(key)
        .arg_bytes(from_ts)
        .arg_bytes(to_ts)
        .arg_keyword_opt_int(COUNT, count)
}

fn apply_ts_mrange_opt(cmd: Cmd, conf: &TsMRange) -> Cmd {
    match conf {
        TsMRange::Latest => cmd.arg(LATEST),
        TsMRange::WithLabels => cmd.arg(WITHLABELS),
        TsMRange::Count(c) => cmd.arg(COUNT).arg_int(*c),
    }
}

fn build_ts_mrange_cmd<F: AsRef<[u8]>>(
    name: &'static str,
    from_ts: &str,
    to_ts: &str,
    filters: &[F],
    conf_li: &[TsMRange],
) -> Cmd {
    let mut cmd = Cmd::new(name).arg_bytes(from_ts).arg_bytes(to_ts);
    for conf in conf_li {
        cmd = apply_ts_mrange_opt(cmd, conf);
    }
    cmd.arg(FILTER).args_slice(filters)
}

fn apply_ts_incrby_opt(cmd: Cmd, conf: &TsIncrBy<'_>) -> Cmd {
    match conf {
        TsIncrBy::Timestamp(ts) => cmd.arg(TIMESTAMP).arg_bytes(ts),
        TsIncrBy::Retention(r) => cmd.arg(RETENTION).arg_int(*r),
        TsIncrBy::ChunkSize(cs) => cmd.arg(CHUNK_SIZE).arg_int(*cs),
        TsIncrBy::Labels(labels) => apply_ts_labels(cmd, labels),
    }
}

fn build_ts_incr_decr_cmd<'a>(
    name: &'static str,
    key: impl AsRef<[u8]>,
    val: f64,
    conf_li: &[TsIncrBy<'a>],
) -> Cmd {
    let mut cmd = Cmd::new(name).arg_bytes(key).arg_float(val);
    for conf in conf_li {
        cmd = apply_ts_incrby_opt(cmd, conf);
    }
    cmd
}

impl Client {
    pub async fn ts_create(
        &self,
        key: impl AsRef<[u8]>,
        conf_li: impl AsRef<[TsCreate<'_>]>,
    ) -> Result<()> {
        let mut retention = None;
        let mut chunk_size = None;
        let mut duplicate_policy = None;
        let mut labels = &[][..];
        for conf in conf_li.as_ref() {
            match conf {
                TsCreate::Retention(r) => retention = Some(*r),
                TsCreate::ChunkSize(cs) => chunk_size = Some(*cs),
                TsCreate::DuplicatePolicy(dp) => duplicate_policy = Some(*dp),
                TsCreate::Labels(l) => labels = l,
            }
        }
        let cmd = apply_ts_create_alter(
            Cmd::new(TS_CREATE).arg_bytes(key),
            retention,
            chunk_size,
            duplicate_policy,
            labels,
        );
        self.execute_cmd(cmd).await
    }

    pub async fn ts_alter(
        &self,
        key: impl AsRef<[u8]>,
        conf_li: impl AsRef<[TsAlter<'_>]>,
    ) -> Result<()> {
        let mut retention = None;
        let mut chunk_size = None;
        let mut duplicate_policy = None;
        let mut labels = &[][..];
        for conf in conf_li.as_ref() {
            match conf {
                TsAlter::Retention(r) => retention = Some(*r),
                TsAlter::ChunkSize(cs) => chunk_size = Some(*cs),
                TsAlter::DuplicatePolicy(dp) => duplicate_policy = Some(*dp),
                TsAlter::Labels(l) => labels = l,
            }
        }
        let cmd = apply_ts_create_alter(
            Cmd::new(TS_ALTER).arg_bytes(key),
            retention,
            chunk_size,
            duplicate_policy,
            labels,
        );
        self.execute_cmd(cmd).await
    }

    pub async fn ts_add(&self, key: impl AsRef<[u8]>, timestamp: &str, val: f64) -> Result<u64> {
        let cmd = Cmd::new(TS_ADD)
            .arg_bytes(key)
            .arg_bytes(timestamp)
            .arg_float(val);
        self.execute_cmd(cmd).await
    }

    pub async fn ts_madd<K: AsRef<[u8]>>(&self, items: &[(K, &str, f64)]) -> Result<Vec<u64>> {
        let mut cmd = Cmd::new(TS_MADD);
        for (k, ts, v) in items {
            cmd = cmd.arg_bytes(k).arg_bytes(ts).arg_float(*v);
        }
        self.execute_cmd(cmd).await
    }

    pub async fn ts_get(&self, key: impl AsRef<[u8]>) -> Result<Option<(u64, f64)>> {
        self.execute_cmd(Cmd::new(TS_GET).arg_bytes(key)).await
    }

    pub async fn ts_range(
        &self,
        key: impl AsRef<[u8]>,
        from_ts: &str,
        to_ts: &str,
        conf_li: impl AsRef<[TsRange]>,
    ) -> Result<Vec<(u64, f64)>> {
        let cmd = build_ts_range_cmd(TS_RANGE, key, from_ts, to_ts, conf_li.as_ref());
        self.execute_cmd(cmd).await
    }

    pub async fn ts_revrange(
        &self,
        key: impl AsRef<[u8]>,
        from_ts: &str,
        to_ts: &str,
        conf_li: impl AsRef<[TsRange]>,
    ) -> Result<Vec<(u64, f64)>> {
        let cmd = build_ts_range_cmd(TS_REVRANGE, key, from_ts, to_ts, conf_li.as_ref());
        self.execute_cmd(cmd).await
    }

    pub async fn ts_info(&self, key: impl AsRef<[u8]>) -> Result<Value> {
        self.execute(Cmd::new(TS_INFO).arg_bytes(key)).await
    }

    pub async fn ts_createrule(
        &self,
        src_key: impl AsRef<[u8]>,
        dst_key: impl AsRef<[u8]>,
        aggregation_type: &str,
        bucket_duration_ms: u64,
    ) -> Result<()> {
        let cmd = Cmd::new(TS_CREATERULE)
            .arg_bytes(src_key)
            .arg_bytes(dst_key)
            .arg(AGGREGATION)
            .arg_bytes(aggregation_type)
            .arg_int(bucket_duration_ms);
        self.execute_cmd(cmd).await
    }

    pub async fn ts_deleterule(
        &self,
        src_key: impl AsRef<[u8]>,
        dst_key: impl AsRef<[u8]>,
    ) -> Result<()> {
        let cmd = Cmd::new(TS_DELETERULE)
            .arg_bytes(src_key)
            .arg_bytes(dst_key);
        self.execute_cmd(cmd).await
    }

    pub async fn ts_mget<F: AsRef<[u8]>>(
        &self,
        filters: &[F],
        conf_li: impl AsRef<[TsMGet]>,
    ) -> Result<Value> {
        let mut cmd = Cmd::new(TS_MGET);
        for conf in conf_li.as_ref() {
            match conf {
                TsMGet::Latest => {
                    cmd = cmd.arg(LATEST);
                }
                TsMGet::WithLabels => {
                    cmd = cmd.arg(WITHLABELS);
                }
            }
        }
        cmd = cmd.arg(FILTER).args_slice(filters);
        self.execute(cmd).await
    }

    pub async fn ts_mrange<F: AsRef<[u8]>>(
        &self,
        from_ts: &str,
        to_ts: &str,
        filters: &[F],
        conf_li: impl AsRef<[TsMRange]>,
    ) -> Result<Value> {
        let cmd = build_ts_mrange_cmd(TS_MRANGE, from_ts, to_ts, filters, conf_li.as_ref());
        self.execute(cmd).await
    }

    pub async fn ts_mrevrange<F: AsRef<[u8]>>(
        &self,
        from_ts: &str,
        to_ts: &str,
        filters: &[F],
        conf_li: impl AsRef<[TsMRange]>,
    ) -> Result<Value> {
        let cmd = build_ts_mrange_cmd(TS_MREVRANGE, from_ts, to_ts, filters, conf_li.as_ref());
        self.execute(cmd).await
    }

    pub async fn ts_incrby(
        &self,
        key: impl AsRef<[u8]>,
        val: f64,
        conf_li: impl AsRef<[TsIncrBy<'_>]>,
    ) -> Result<u64> {
        let cmd = build_ts_incr_decr_cmd(TS_INCRBY, key, val, conf_li.as_ref());
        self.execute_cmd(cmd).await
    }

    pub async fn ts_decrby(
        &self,
        key: impl AsRef<[u8]>,
        val: f64,
        conf_li: impl AsRef<[TsIncrBy<'_>]>,
    ) -> Result<u64> {
        let cmd = build_ts_incr_decr_cmd(TS_DECRBY, key, val, conf_li.as_ref());
        self.execute_cmd(cmd).await
    }

    pub async fn ts_del(&self, key: impl AsRef<[u8]>, from_ts: u64, to_ts: u64) -> Result<u64> {
        let cmd = Cmd::new(TS_DEL)
            .arg_bytes(key)
            .arg_int(from_ts)
            .arg_int(to_ts);
        self.execute_cmd(cmd).await
    }

    pub async fn ts_queryindex<F: AsRef<[u8]>>(&self, filters: &[F]) -> Result<Vec<String>> {
        self.execute_cmd(Cmd::new(TS_QUERYINDEX).args_slice(filters))
            .await
    }
}
