use rapidhash::RapidHashMap as HashMap;

use crate::{
    client::{Client, helper::build_auth_cmd},
    error::Result,
    resp3::{
        Cmd, Value,
        constants::{
            ADD, APPLYBATCH, ASYNC, AUTH, BGSAVE, CLIENT, CLUSTER, COMMAND, COMPACT, CONFIG,
            DBSIZE, DEBUG, DEL, DISK, DUMP, ECHO, FLUSHALL, FLUSHBACKUP, FLUSHBLOCKCACHE, FLUSHDB,
            FLUSHMEMTABLE, FORMAT, GET, GETNAME, HELLO, INFO, KILL, KPROFILE, LASTSAVE, LATENCY,
            LEN, LIST, LOWPRI, MAX, MEMORY, MONITOR, NAMESPACE, NO, ONE, PERFLOG, PING,
            POLLUPDATES, QUIT, RAW, RDB, REPLACE, REPLICAOF, RESET, RESETSTAT, RESP, RESTORE,
            REWRITE, ROLE, SELECT, SET, SETNAME, SHUTDOWN, SLAVEOF, SLOWLOG, SST, STATS, STRICT,
            TIME, TYPE, USAGE,
        },
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PollUpdates {
    Max(usize),
    Strict,
    FormatRaw,
    FormatResp,
}

fn apply_pollupdates_opt(cmd: Cmd, conf: &PollUpdates) -> Cmd {
    match conf {
        PollUpdates::Max(c) => cmd.arg(MAX).arg_int(*c),
        PollUpdates::Strict => cmd.arg(STRICT),
        PollUpdates::FormatRaw => cmd.arg(FORMAT).arg(RAW),
        PollUpdates::FormatResp => cmd.arg(FORMAT).arg(RESP),
    }
}

impl Client {
    pub async fn auth(&self, username: Option<&str>, password: &str) -> Result<()> {
        self.execute_cmd(build_auth_cmd(username, password)).await
    }

    pub async fn select(&self, db: u8) -> Result<()> {
        self.execute_cmd(Cmd::new(SELECT).arg_int(db)).await
    }

    pub async fn ping(&self, msg: Option<&str>) -> Result<String> {
        self.execute_cmd(Cmd::new(PING).arg_opt_bytes(msg)).await
    }

    pub async fn echo(&self, message: impl AsRef<[u8]>) -> Result<String> {
        self.execute_cmd(Cmd::new(ECHO).arg_bytes(message)).await
    }

    pub async fn time(&self) -> Result<(u64, u64)> {
        self.execute_cmd(Cmd::new(TIME)).await
    }

    pub async fn dbsize(&self) -> Result<u64> {
        self.execute_cmd(Cmd::new(DBSIZE)).await
    }

    pub async fn flushdb(&self, async_flush: bool) -> Result<()> {
        self.execute_cmd(Cmd::new(FLUSHDB).arg_if(async_flush, ASYNC))
            .await
    }

    pub async fn flushall(&self, async_flush: bool) -> Result<()> {
        self.execute_cmd(Cmd::new(FLUSHALL).arg_if(async_flush, ASYNC))
            .await
    }

    pub async fn info(&self, section: Option<&str>) -> Result<String> {
        self.execute_cmd(Cmd::new(INFO).arg_opt_bytes(section))
            .await
    }

    pub async fn role(&self) -> Result<Value> {
        self.execute(Cmd::new(ROLE)).await
    }

    pub async fn config_get(&self, parameter: &str) -> Result<HashMap<String, String>> {
        self.execute_cmd(Cmd::new(CONFIG).arg(GET).arg_bytes(parameter))
            .await
    }

    pub async fn config_set(&self, parameter: &str, value: &str) -> Result<()> {
        self.execute_cmd(
            Cmd::new(CONFIG)
                .arg(SET)
                .arg_bytes(parameter)
                .arg_bytes(value),
        )
        .await
    }

    pub async fn config_rewrite(&self) -> Result<()> {
        self.execute_cmd(Cmd::new(CONFIG).arg(REWRITE)).await
    }

    pub async fn config_resetstat(&self) -> Result<()> {
        self.execute_cmd(Cmd::new(CONFIG).arg(RESETSTAT)).await
    }

    /// Kvrocks specific: NAMESPACE ADD
    pub async fn namespace_add(&self, ns: &str, token: &str) -> Result<()> {
        self.execute_cmd(Cmd::new(NAMESPACE).arg(ADD).arg_bytes(ns).arg_bytes(token))
            .await
    }

    /// Kvrocks specific: NAMESPACE SET
    pub async fn namespace_set(&self, ns: &str, token: &str) -> Result<()> {
        self.execute_cmd(Cmd::new(NAMESPACE).arg(SET).arg_bytes(ns).arg_bytes(token))
            .await
    }

    /// Kvrocks specific: NAMESPACE DEL
    pub async fn namespace_del(&self, ns: &str) -> Result<()> {
        self.execute_cmd(Cmd::new(NAMESPACE).arg(DEL).arg_bytes(ns))
            .await
    }

    /// Kvrocks specific: NAMESPACE GET
    pub async fn namespace_get(&self, ns: Option<&str>) -> Result<HashMap<String, String>> {
        self.execute_cmd(Cmd::new(NAMESPACE).arg(GET).arg_bytes(ns.unwrap_or("*")))
            .await
    }

    /// Kvrocks specific: COMPACT
    pub async fn compact(&self, cluster: bool) -> Result<()> {
        self.execute_cmd(Cmd::new(COMPACT).arg_if(cluster, CLUSTER))
            .await
    }

    pub async fn bgsave(&self) -> Result<String> {
        self.execute_cmd(Cmd::new(BGSAVE)).await
    }

    pub async fn lastsave(&self) -> Result<u64> {
        self.execute_cmd(Cmd::new(LASTSAVE)).await
    }

    pub async fn slowlog_get(&self, count: Option<usize>) -> Result<Value> {
        self.execute(Cmd::new(SLOWLOG).arg(GET).arg_opt_int(count))
            .await
    }

    pub async fn slowlog_len(&self) -> Result<u64> {
        self.execute_cmd(Cmd::new(SLOWLOG).arg(LEN)).await
    }

    pub async fn slowlog_reset(&self) -> Result<()> {
        self.execute_cmd(Cmd::new(SLOWLOG).arg(RESET)).await
    }

    pub async fn client_list(&self, client_type: Option<&str>) -> Result<String> {
        let cmd = Cmd::new(CLIENT)
            .arg(LIST)
            .arg_keyword_opt_bytes(TYPE, client_type);
        self.execute_cmd(cmd).await
    }

    pub async fn client_getname(&self) -> Result<Option<String>> {
        self.execute_cmd(Cmd::new(CLIENT).arg(GETNAME)).await
    }

    pub async fn client_setname(&self, name: &str) -> Result<()> {
        self.execute_cmd(Cmd::new(CLIENT).arg(SETNAME).arg_bytes(name))
            .await
    }

    pub async fn client_kill(&self, ip_port: &str) -> Result<bool> {
        self.execute_cmd(Cmd::new(CLIENT).arg(KILL).arg_bytes(ip_port))
            .await
    }

    /// Kvrocks specific: DISK USAGE
    pub async fn disk_usage(&self, key: impl AsRef<[u8]>) -> Result<Option<u64>> {
        self.execute_cmd(Cmd::new(DISK).arg(USAGE).arg_bytes(key))
            .await
    }

    /// Kvrocks specific: MEMORY USAGE
    pub async fn memory_usage(&self, key: impl AsRef<[u8]>) -> Result<Option<u64>> {
        self.execute_cmd(Cmd::new(MEMORY).arg(USAGE).arg_bytes(key))
            .await
    }

    /// Kvrocks specific: POLLUPDATES <sequence> [MAX <count>] [STRICT] [FORMAT <RAW|RESP>]
    pub async fn pollupdates(
        &self,
        sequence: u64,
        conf_li: impl AsRef<[PollUpdates]>,
    ) -> Result<Value> {
        let mut cmd = Cmd::new(POLLUPDATES).arg_int(sequence);
        for conf in conf_li.as_ref() {
            cmd = apply_pollupdates_opt(cmd, conf);
        }
        self.execute(cmd).await
    }

    /// Kvrocks specific: KPROFILE
    pub async fn kprofile<A: AsRef<[u8]>>(&self, subcommand: &str, args: &[A]) -> Result<Value> {
        self.execute(Cmd::new(KPROFILE).arg_bytes(subcommand).args_slice(args))
            .await
    }

    /// Kvrocks specific: PERFLOG
    pub async fn perflog<A: AsRef<[u8]>>(&self, subcommand: &str, args: &[A]) -> Result<Value> {
        self.execute(Cmd::new(PERFLOG).arg_bytes(subcommand).args_slice(args))
            .await
    }

    pub async fn monitor(&self) -> Result<Value> {
        self.execute(Cmd::new(MONITOR)).await
    }

    pub async fn shutdown(&self) -> Result<()> {
        self.execute_cmd(Cmd::new(SHUTDOWN)).await
    }

    pub async fn quit(&self) -> Result<()> {
        self.execute_cmd(Cmd::new(QUIT)).await
    }

    pub async fn debug<A: AsRef<[u8]>>(&self, subcommand: &str, args: &[A]) -> Result<Value> {
        self.execute(Cmd::new(DEBUG).arg_bytes(subcommand).args_slice(args))
            .await
    }

    pub async fn command<A: AsRef<[u8]>>(&self, args: &[A]) -> Result<Value> {
        self.execute(Cmd::new(COMMAND).args_slice(args)).await
    }

    pub async fn hello(&self, protover: u8, auth: Option<(&str, &str)>) -> Result<Value> {
        let mut cmd = Cmd::new(HELLO).arg_int(protover);
        if let Some((u, p)) = auth {
            cmd = cmd.arg(AUTH).arg_bytes(u).arg_bytes(p);
        }
        self.execute(cmd).await
    }

    pub async fn restore(
        &self,
        key: impl AsRef<[u8]>,
        ttl: u64,
        serialized_value: impl AsRef<[u8]>,
        replace: bool,
    ) -> Result<()> {
        let cmd = Cmd::new(RESTORE)
            .arg_bytes(key)
            .arg_int(ttl)
            .arg_bytes(serialized_value)
            .arg_if(replace, REPLACE);
        self.execute_cmd(cmd).await
    }

    pub async fn flushbackup(&self) -> Result<()> {
        self.execute_cmd(Cmd::new(FLUSHBACKUP)).await
    }

    pub async fn slaveof(&self, host: &str, port: u16) -> Result<String> {
        self.execute_cmd(Cmd::new(SLAVEOF).arg_bytes(host).arg_int(port))
            .await
    }

    pub async fn slaveof_no_one(&self) -> Result<String> {
        self.execute_cmd(Cmd::new(SLAVEOF).arg(NO).arg(ONE)).await
    }

    pub async fn replicaof(&self, host: &str, port: u16) -> Result<String> {
        self.execute_cmd(Cmd::new(REPLICAOF).arg_bytes(host).arg_int(port))
            .await
    }

    pub async fn replicaof_no_one(&self) -> Result<String> {
        self.execute_cmd(Cmd::new(REPLICAOF).arg(NO).arg(ONE)).await
    }

    /// Kvrocks specific: STATS
    pub async fn stats(&self) -> Result<String> {
        self.execute_cmd(Cmd::new(STATS)).await
    }

    /// Kvrocks specific: RDB
    pub async fn rdb<A: AsRef<[u8]>>(&self, subcommand: &str, args: &[A]) -> Result<Value> {
        self.execute(Cmd::new(RDB).arg_bytes(subcommand).args_slice(args))
            .await
    }

    pub async fn reset(&self) -> Result<String> {
        self.execute_cmd(Cmd::new(RESET)).await
    }

    /// Kvrocks specific: APPLYBATCH <raw_batch> [LOWPRI]
    pub async fn applybatch(&self, raw_batch: impl AsRef<[u8]>, lowpri: bool) -> Result<u64> {
        let cmd = Cmd::new(APPLYBATCH)
            .arg_bytes(raw_batch)
            .arg_if(lowpri, LOWPRI);
        self.execute_cmd(cmd).await
    }

    pub async fn dump(&self, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>> {
        self.execute_cmd(Cmd::new(DUMP).arg_bytes(key)).await
    }

    /// Kvrocks specific: SST
    pub async fn sst<A: AsRef<[u8]>>(&self, subcommand: &str, args: &[A]) -> Result<Value> {
        self.execute(Cmd::new(SST).arg_bytes(subcommand).args_slice(args))
            .await
    }

    /// Kvrocks specific: FLUSHMEMTABLE [ASYNC]
    pub async fn flushmemtable(&self, async_flush: bool) -> Result<()> {
        self.execute_cmd(Cmd::new(FLUSHMEMTABLE).arg_if(async_flush, ASYNC))
            .await
    }

    /// Kvrocks specific: FLUSHBLOCKCACHE
    pub async fn flushblockcache(&self) -> Result<()> {
        self.execute_cmd(Cmd::new(FLUSHBLOCKCACHE)).await
    }

    pub async fn latency<A: AsRef<[u8]>>(&self, subcommand: &str, args: &[A]) -> Result<Value> {
        self.execute(Cmd::new(LATENCY).arg_bytes(subcommand).args_slice(args))
            .await
    }
}
