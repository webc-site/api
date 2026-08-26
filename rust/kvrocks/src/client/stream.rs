use crate::{
    client::Client,
    error::Result,
    resp3::{
        Cmd, Value,
        constants::{
            BLOCK, CONSUMERS, COUNT, CREATE, DESTROY, FORCE, GROUP, GROUPS, IDLE, JUSTID, LIMIT,
            MAXLEN, MINID, MKSTREAM, NOACK, NOMKSTREAM, RETRYCOUNT, SETID, STREAM, STREAMS, TILDE,
            TIME, XACK, XADD, XAUTOCLAIM, XCLAIM, XDEL, XGROUP, XINFO, XLEN, XPENDING, XRANGE,
            XREAD, XREADGROUP, XREVRANGE, XSETID, XTRIM,
        },
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XAddOption<'a> {
    Nomkstream,
    MaxLen(u64, bool),    // (threshold, approximate)
    MinId(&'a str, bool), // (min_id, approximate)
    Limit(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XClaim {
    Idle(u64),
    Time(u64),
    RetryCount(u64),
    Force,
    JustId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XAutoClaim {
    Count(usize),
    JustId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XReadGroup {
    Count(usize),
    Block(u64),
    NoAck,
}

fn append_stream_keys_and_ids<K: AsRef<[u8]>, I: AsRef<[u8]>>(
    mut cmd: Cmd,
    streams: &[(K, I)],
) -> Cmd {
    cmd = cmd.arg(STREAMS);
    for (k, _) in streams {
        cmd = cmd.arg_bytes(k);
    }
    for (_, id) in streams {
        cmd = cmd.arg_bytes(id);
    }
    cmd
}

fn apply_xadd_opt(cmd: Cmd, conf: &XAddOption<'_>) -> Cmd {
    match conf {
        XAddOption::Nomkstream => cmd.arg(NOMKSTREAM),
        XAddOption::MaxLen(threshold, approx) => {
            cmd.arg(MAXLEN).arg_if(*approx, TILDE).arg_int(*threshold)
        }
        XAddOption::MinId(min_id, approx) => {
            cmd.arg(MINID).arg_if(*approx, TILDE).arg_bytes(min_id)
        }
        XAddOption::Limit(limit) => cmd.arg(LIMIT).arg_int(*limit),
    }
}

fn apply_xreadgroup_opt(cmd: Cmd, conf: &XReadGroup) -> Cmd {
    match conf {
        XReadGroup::Count(c) => cmd.arg(COUNT).arg_int(*c),
        XReadGroup::Block(b) => cmd.arg(BLOCK).arg_int(*b),
        XReadGroup::NoAck => cmd.arg(NOACK),
    }
}

fn apply_xclaim_opt(cmd: Cmd, conf: &XClaim) -> Cmd {
    match conf {
        XClaim::Idle(ms) => cmd.arg(IDLE).arg_int(*ms),
        XClaim::Time(ms) => cmd.arg(TIME).arg_int(*ms),
        XClaim::RetryCount(c) => cmd.arg(RETRYCOUNT).arg_int(*c),
        XClaim::Force => cmd.arg(FORCE),
        XClaim::JustId => cmd.arg(JUSTID),
    }
}

fn apply_xautoclaim_opt(cmd: Cmd, conf: &XAutoClaim) -> Cmd {
    match conf {
        XAutoClaim::Count(c) => cmd.arg(COUNT).arg_int(*c),
        XAutoClaim::JustId => cmd.arg(JUSTID),
    }
}

impl Client {
    pub async fn xadd<F: AsRef<[u8]>, V: AsRef<[u8]>>(
        &self,
        key: impl AsRef<[u8]>,
        id: &str,
        fields: &[(F, V)],
    ) -> Result<Option<String>> {
        self.xadd_opt(key, id, fields, &[]).await
    }

    pub async fn xadd_opt<F: AsRef<[u8]>, V: AsRef<[u8]>>(
        &self,
        key: impl AsRef<[u8]>,
        id: &str,
        fields: &[(F, V)],
        conf_li: impl AsRef<[XAddOption<'_>]>,
    ) -> Result<Option<String>> {
        let mut cmd = Cmd::new(XADD).arg_bytes(key);
        for conf in conf_li.as_ref() {
            cmd = apply_xadd_opt(cmd, conf);
        }
        cmd = cmd.arg_bytes(id).args_pairs(fields);
        self.execute_cmd(cmd).await
    }

    pub async fn xlen(&self, key: impl AsRef<[u8]>) -> Result<u64> {
        self.execute_cmd(Cmd::new(XLEN).arg_bytes(key)).await
    }

    pub async fn xdel<I: AsRef<[u8]>>(&self, key: impl AsRef<[u8]>, ids: &[I]) -> Result<u64> {
        self.execute_cmd(Cmd::new(XDEL).arg_bytes(key).args_slice(ids))
            .await
    }

    pub async fn xtrim(
        &self,
        key: impl AsRef<[u8]>,
        threshold: u64,
        approximate: bool,
    ) -> Result<u64> {
        let cmd = Cmd::new(XTRIM)
            .arg_bytes(key)
            .arg(MAXLEN)
            .arg_if(approximate, TILDE)
            .arg_int(threshold);
        self.execute_cmd(cmd).await
    }

    pub async fn xrange(
        &self,
        key: impl AsRef<[u8]>,
        start: &str,
        end: &str,
        count: Option<usize>,
    ) -> Result<Value> {
        let cmd = Cmd::new(XRANGE)
            .arg_bytes(key)
            .arg_bytes(start)
            .arg_bytes(end)
            .arg_keyword_opt_int(COUNT, count);
        self.execute(cmd).await
    }

    pub async fn xrevrange(
        &self,
        key: impl AsRef<[u8]>,
        end: &str,
        start: &str,
        count: Option<usize>,
    ) -> Result<Value> {
        let cmd = Cmd::new(XREVRANGE)
            .arg_bytes(key)
            .arg_bytes(end)
            .arg_bytes(start)
            .arg_keyword_opt_int(COUNT, count);
        self.execute(cmd).await
    }

    pub async fn xread<K: AsRef<[u8]>, I: AsRef<[u8]>>(
        &self,
        count: Option<usize>,
        block: Option<u64>,
        streams: &[(K, I)],
    ) -> Result<Value> {
        let cmd = Cmd::new(XREAD)
            .arg_keyword_opt_int(COUNT, count)
            .arg_keyword_opt_int(BLOCK, block);
        let cmd = append_stream_keys_and_ids(cmd, streams);
        self.execute(cmd).await
    }

    pub async fn xreadgroup<K: AsRef<[u8]>, I: AsRef<[u8]>>(
        &self,
        group: impl AsRef<[u8]>,
        consumer: impl AsRef<[u8]>,
        streams: &[(K, I)],
        conf_li: impl AsRef<[XReadGroup]>,
    ) -> Result<Value> {
        let mut cmd = Cmd::new(XREADGROUP)
            .arg(GROUP)
            .arg_bytes(group)
            .arg_bytes(consumer);
        for conf in conf_li.as_ref() {
            cmd = apply_xreadgroup_opt(cmd, conf);
        }
        let cmd = append_stream_keys_and_ids(cmd, streams);
        self.execute(cmd).await
    }

    pub async fn xclaim<I: AsRef<[u8]>>(
        &self,
        key: impl AsRef<[u8]>,
        group: impl AsRef<[u8]>,
        consumer: impl AsRef<[u8]>,
        min_idle_time_ms: u64,
        ids: &[I],
        conf_li: impl AsRef<[XClaim]>,
    ) -> Result<Value> {
        let mut cmd = Cmd::new(XCLAIM)
            .arg_bytes(key)
            .arg_bytes(group)
            .arg_bytes(consumer)
            .arg_int(min_idle_time_ms)
            .args_slice(ids);
        for conf in conf_li.as_ref() {
            cmd = apply_xclaim_opt(cmd, conf);
        }
        self.execute(cmd).await
    }

    pub async fn xautoclaim(
        &self,
        key: impl AsRef<[u8]>,
        group: impl AsRef<[u8]>,
        consumer: impl AsRef<[u8]>,
        min_idle_time_ms: u64,
        start: &str,
        conf_li: impl AsRef<[XAutoClaim]>,
    ) -> Result<Value> {
        let mut cmd = Cmd::new(XAUTOCLAIM)
            .arg_bytes(key)
            .arg_bytes(group)
            .arg_bytes(consumer)
            .arg_int(min_idle_time_ms)
            .arg_bytes(start);
        for conf in conf_li.as_ref() {
            cmd = apply_xautoclaim_opt(cmd, conf);
        }
        self.execute(cmd).await
    }

    pub async fn xpending(
        &self,
        key: impl AsRef<[u8]>,
        group: impl AsRef<[u8]>,
        range: Option<(&str, &str, usize)>,
        consumer: Option<&str>,
    ) -> Result<Value> {
        let mut cmd = Cmd::new(XPENDING).arg_bytes(key).arg_bytes(group);
        if let Some((start, end, count)) = range {
            cmd = cmd.arg_bytes(start).arg_bytes(end).arg_int(count);
            if let Some(c) = consumer {
                cmd = cmd.arg_bytes(c);
            }
        }
        self.execute(cmd).await
    }

    pub async fn xack<I: AsRef<[u8]>>(
        &self,
        key: impl AsRef<[u8]>,
        group: impl AsRef<[u8]>,
        ids: &[I],
    ) -> Result<u64> {
        self.execute_cmd(
            Cmd::new(XACK)
                .arg_bytes(key)
                .arg_bytes(group)
                .args_slice(ids),
        )
        .await
    }

    pub async fn xgroup_create(
        &self,
        key: impl AsRef<[u8]>,
        group: impl AsRef<[u8]>,
        id: &str,
        mkstream: bool,
    ) -> Result<()> {
        let cmd = Cmd::new(XGROUP)
            .arg(CREATE)
            .arg_bytes(key)
            .arg_bytes(group)
            .arg_bytes(id)
            .arg_if(mkstream, MKSTREAM);
        self.execute_cmd(cmd).await
    }

    pub async fn xgroup_destroy(
        &self,
        key: impl AsRef<[u8]>,
        group: impl AsRef<[u8]>,
    ) -> Result<bool> {
        self.execute_cmd(
            Cmd::new(XGROUP)
                .arg(DESTROY)
                .arg_bytes(key)
                .arg_bytes(group),
        )
        .await
    }

    pub async fn xgroup_setid(
        &self,
        key: impl AsRef<[u8]>,
        group: impl AsRef<[u8]>,
        id: &str,
    ) -> Result<()> {
        self.execute_cmd(
            Cmd::new(XGROUP)
                .arg(SETID)
                .arg_bytes(key)
                .arg_bytes(group)
                .arg_bytes(id),
        )
        .await
    }

    pub async fn xsetid(&self, key: impl AsRef<[u8]>, last_id: &str) -> Result<()> {
        self.execute_cmd(Cmd::new(XSETID).arg_bytes(key).arg_bytes(last_id))
            .await
    }

    pub async fn xinfo_stream(&self, key: impl AsRef<[u8]>) -> Result<Value> {
        self.execute(Cmd::new(XINFO).arg(STREAM).arg_bytes(key))
            .await
    }

    pub async fn xinfo_groups(&self, key: impl AsRef<[u8]>) -> Result<Value> {
        self.execute(Cmd::new(XINFO).arg(GROUPS).arg_bytes(key))
            .await
    }

    pub async fn xinfo_consumers(
        &self,
        key: impl AsRef<[u8]>,
        group: impl AsRef<[u8]>,
    ) -> Result<Value> {
        self.execute(
            Cmd::new(XINFO)
                .arg(CONSUMERS)
                .arg_bytes(key)
                .arg_bytes(group),
        )
        .await
    }
}
