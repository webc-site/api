use crate::{
    client::{Client, helper::apply_scan_opts},
    error::Result,
    resp3::{
        Cmd, Value,
        constants::{
            ALPHA, ASC, BY, COPY, DEL, DESC, EXISTS, EXPIRE, EXPIREAT, EXPIRETIME, GET, KEYS,
            KMETADATA, LIMIT, MOVE, MOVEX, OBJECT, PERSIST, PEXPIRE, PEXPIREAT, PEXPIRETIME, PTTL,
            RANDOMKEY, RENAME, RENAMENX, REPLACE, SCAN, SORT, SORT_RO, STORE, TTL, TYPE, UNLINK,
        },
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scan<'a> {
    Match(&'a str),
    Count(usize),
    Type(&'a str),
    NoValues,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Sort<'a> {
    By(&'a str),
    Limit(usize, usize),
    Get(&'a str),
    Asc,
    Desc,
    Alpha,
    Store(&'a str),
}

fn apply_sort_opt(cmd: Cmd, conf: &Sort<'_>, is_ro: bool) -> Cmd {
    match conf {
        Sort::By(pattern) => cmd.arg(BY).arg_bytes(pattern),
        Sort::Limit(offset, count) => cmd.arg(LIMIT).arg_int(*offset).arg_int(*count),
        Sort::Get(pattern) => cmd.arg(GET).arg_bytes(pattern),
        Sort::Asc => cmd.arg(ASC),
        Sort::Desc => cmd.arg(DESC),
        Sort::Alpha => cmd.arg(ALPHA),
        Sort::Store(dst) => {
            if is_ro {
                cmd
            } else {
                cmd.arg(STORE).arg_bytes(dst)
            }
        }
    }
}

fn build_sort_cmd<'a>(name: &'static str, key: impl AsRef<[u8]>, conf_li: &[Sort<'a>]) -> Cmd {
    let mut cmd = Cmd::new(name).arg_bytes(key);
    let is_ro = name == SORT_RO;
    for conf in conf_li {
        cmd = apply_sort_opt(cmd, conf, is_ro);
    }
    cmd
}

impl Client {
    pub async fn del(&self, key: impl AsRef<[u8]>) -> Result<u64> {
        self.execute_cmd(Cmd::new(DEL).arg_bytes(key)).await
    }

    pub async fn mdel<K: AsRef<[u8]>>(&self, keys: &[K]) -> Result<u64> {
        self.execute_cmd(Cmd::new(DEL).args_slice(keys)).await
    }

    pub async fn unlink(&self, key: impl AsRef<[u8]>) -> Result<u64> {
        self.execute_cmd(Cmd::new(UNLINK).arg_bytes(key)).await
    }

    pub async fn munlink<K: AsRef<[u8]>>(&self, keys: &[K]) -> Result<u64> {
        self.execute_cmd(Cmd::new(UNLINK).args_slice(keys)).await
    }

    pub async fn exists(&self, key: impl AsRef<[u8]>) -> Result<bool> {
        self.execute_cmd(Cmd::new(EXISTS).arg_bytes(key)).await
    }

    pub async fn mexists<K: AsRef<[u8]>>(&self, keys: &[K]) -> Result<u64> {
        self.execute_cmd(Cmd::new(EXISTS).args_slice(keys)).await
    }

    pub async fn expire(&self, key: impl AsRef<[u8]>, seconds: u64) -> Result<bool> {
        self.execute_cmd(Cmd::new(EXPIRE).arg_bytes(key).arg_int(seconds))
            .await
    }

    pub async fn pexpire(&self, key: impl AsRef<[u8]>, milliseconds: u64) -> Result<bool> {
        self.execute_cmd(Cmd::new(PEXPIRE).arg_bytes(key).arg_int(milliseconds))
            .await
    }

    pub async fn expireat(&self, key: impl AsRef<[u8]>, timestamp: u64) -> Result<bool> {
        self.execute_cmd(Cmd::new(EXPIREAT).arg_bytes(key).arg_int(timestamp))
            .await
    }

    pub async fn pexpireat(&self, key: impl AsRef<[u8]>, timestamp_ms: u64) -> Result<bool> {
        self.execute_cmd(Cmd::new(PEXPIREAT).arg_bytes(key).arg_int(timestamp_ms))
            .await
    }

    pub async fn expiretime(&self, key: impl AsRef<[u8]>) -> Result<i64> {
        self.execute_cmd(Cmd::new(EXPIRETIME).arg_bytes(key)).await
    }

    pub async fn pexpiretime(&self, key: impl AsRef<[u8]>) -> Result<i64> {
        self.execute_cmd(Cmd::new(PEXPIRETIME).arg_bytes(key)).await
    }

    pub async fn ttl(&self, key: impl AsRef<[u8]>) -> Result<i64> {
        self.execute_cmd(Cmd::new(TTL).arg_bytes(key)).await
    }

    pub async fn pttl(&self, key: impl AsRef<[u8]>) -> Result<i64> {
        self.execute_cmd(Cmd::new(PTTL).arg_bytes(key)).await
    }

    pub async fn persist(&self, key: impl AsRef<[u8]>) -> Result<bool> {
        self.execute_cmd(Cmd::new(PERSIST).arg_bytes(key)).await
    }

    pub async fn key_type(&self, key: impl AsRef<[u8]>) -> Result<String> {
        self.execute_cmd(Cmd::new(TYPE).arg_bytes(key)).await
    }

    pub async fn rename(&self, key: impl AsRef<[u8]>, new_key: impl AsRef<[u8]>) -> Result<()> {
        self.execute_cmd(Cmd::new(RENAME).arg_bytes(key).arg_bytes(new_key))
            .await
    }

    pub async fn renamenx(&self, key: impl AsRef<[u8]>, new_key: impl AsRef<[u8]>) -> Result<bool> {
        self.execute_cmd(Cmd::new(RENAMENX).arg_bytes(key).arg_bytes(new_key))
            .await
    }

    pub async fn move_to_db(&self, key: impl AsRef<[u8]>, db: u64) -> Result<bool> {
        self.execute_cmd(Cmd::new(MOVE).arg_bytes(key).arg_int(db))
            .await
    }

    pub async fn movex(&self, key: impl AsRef<[u8]>, namespace: impl AsRef<[u8]>) -> Result<bool> {
        self.execute_cmd(Cmd::new(MOVEX).arg_bytes(key).arg_bytes(namespace))
            .await
    }

    pub async fn object(&self, subcommand: &str, key: impl AsRef<[u8]>) -> Result<Value> {
        self.execute(Cmd::new(OBJECT).arg_bytes(subcommand).arg_bytes(key))
            .await
    }

    pub async fn keys(&self, pattern: impl AsRef<str>) -> Result<Vec<String>> {
        self.execute_cmd(Cmd::new(KEYS).arg_bytes(pattern.as_ref()))
            .await
    }

    pub async fn scan(
        &self,
        cursor: u64,
        conf_li: impl AsRef<[Scan<'_>]>,
    ) -> Result<(u64, Vec<String>)> {
        let mut cmd = Cmd::new(SCAN).arg_int(cursor);
        let mut r#match = None;
        let mut count = None;
        let mut no_values = false;
        for conf in conf_li.as_ref() {
            match conf {
                Scan::Match(p) => r#match = Some(*p),
                Scan::Count(c) => count = Some(*c),
                Scan::Type(t) => cmd = cmd.arg(TYPE).arg_bytes(t),
                Scan::NoValues => no_values = true,
            }
        }
        self.exec_scan(apply_scan_opts(cmd, r#match, count, no_values))
            .await
    }

    pub async fn randomkey(&self) -> Result<Option<String>> {
        self.execute_cmd(Cmd::new(RANDOMKEY)).await
    }

    pub async fn copy(
        &self,
        source: impl AsRef<[u8]>,
        destination: impl AsRef<[u8]>,
        replace: bool,
    ) -> Result<bool> {
        let cmd = Cmd::new(COPY)
            .arg_bytes(source)
            .arg_bytes(destination)
            .arg_if(replace, REPLACE);
        self.execute_cmd(cmd).await
    }

    pub async fn sort(
        &self,
        key: impl AsRef<[u8]>,
        conf_li: impl AsRef<[Sort<'_>]>,
    ) -> Result<Value> {
        let cmd = build_sort_cmd(SORT, key, conf_li.as_ref());
        self.execute(cmd).await
    }

    pub async fn sort_ro(
        &self,
        key: impl AsRef<[u8]>,
        conf_li: impl AsRef<[Sort<'_>]>,
    ) -> Result<Value> {
        let cmd = build_sort_cmd(SORT_RO, key, conf_li.as_ref());
        self.execute(cmd).await
    }

    pub async fn kmetadata(&self, key: impl AsRef<[u8]>) -> Result<Value> {
        self.execute(Cmd::new(KMETADATA).arg_bytes(key)).await
    }
}
