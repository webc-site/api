use crate::{
    client::{Client, helper::apply_scan_opts},
    error::Result,
    resp3::{
        Cmd, FromValue,
        constants::{
            LIMIT, SADD, SCARD, SDIFF, SDIFFSTORE, SINTER, SINTERCARD, SINTERSTORE, SISMEMBER,
            SMEMBERS, SMISMEMBER, SMOVE, SPOP, SRANDMEMBER, SREM, SSCAN, SUNION, SUNIONSTORE,
        },
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SScan<'a> {
    Match(&'a str),
    Count(usize),
    NoValues,
}

impl Client {
    pub async fn sadd<V: AsRef<[u8]>>(&self, key: impl AsRef<[u8]>, members: &[V]) -> Result<u64> {
        self.execute_cmd(Cmd::new(SADD).arg_bytes(key).args_slice(members))
            .await
    }

    pub async fn srem<V: AsRef<[u8]>>(&self, key: impl AsRef<[u8]>, members: &[V]) -> Result<u64> {
        self.execute_cmd(Cmd::new(SREM).arg_bytes(key).args_slice(members))
            .await
    }

    pub async fn scard(&self, key: impl AsRef<[u8]>) -> Result<u64> {
        self.execute_cmd(Cmd::new(SCARD).arg_bytes(key)).await
    }

    pub async fn smembers<T: FromValue>(&self, key: impl AsRef<[u8]>) -> Result<Vec<T>> {
        self.execute_cmd(Cmd::new(SMEMBERS).arg_bytes(key)).await
    }

    pub async fn sismember(&self, key: impl AsRef<[u8]>, member: impl AsRef<[u8]>) -> Result<bool> {
        self.execute_cmd(Cmd::new(SISMEMBER).arg_bytes(key).arg_bytes(member))
            .await
    }

    pub async fn smismember<M: AsRef<[u8]>>(
        &self,
        key: impl AsRef<[u8]>,
        members: &[M],
    ) -> Result<Vec<bool>> {
        self.execute_cmd(Cmd::new(SMISMEMBER).arg_bytes(key).args_slice(members))
            .await
    }

    pub async fn spop<T: FromValue>(
        &self,
        key: impl AsRef<[u8]>,
        count: Option<usize>,
    ) -> Result<Vec<T>> {
        self.exec_single_or_array(Cmd::new(SPOP).arg_bytes(key).arg_opt_int(count))
            .await
    }

    pub async fn srandmember<T: FromValue>(
        &self,
        key: impl AsRef<[u8]>,
        count: Option<i64>,
    ) -> Result<Vec<T>> {
        self.exec_single_or_array(Cmd::new(SRANDMEMBER).arg_bytes(key).arg_opt_int(count))
            .await
    }

    pub async fn smove(
        &self,
        src: impl AsRef<[u8]>,
        dst: impl AsRef<[u8]>,
        member: impl AsRef<[u8]>,
    ) -> Result<bool> {
        self.execute_cmd(
            Cmd::new(SMOVE)
                .arg_bytes(src)
                .arg_bytes(dst)
                .arg_bytes(member),
        )
        .await
    }

    pub async fn sinter<T: FromValue, K: AsRef<[u8]>>(&self, keys: &[K]) -> Result<Vec<T>> {
        self.execute_cmd(Cmd::new(SINTER).args_slice(keys)).await
    }

    pub async fn sinterstore<K: AsRef<[u8]>>(
        &self,
        dst: impl AsRef<[u8]>,
        keys: &[K],
    ) -> Result<u64> {
        self.execute_cmd(Cmd::new(SINTERSTORE).arg_bytes(dst).args_slice(keys))
            .await
    }

    pub async fn sintercard<K: AsRef<[u8]>>(
        &self,
        keys: &[K],
        limit: Option<usize>,
    ) -> Result<u64> {
        let cmd = Cmd::new(SINTERCARD)
            .arg_int(keys.len())
            .args_slice(keys)
            .arg_keyword_opt_int(LIMIT, limit);
        self.execute_cmd(cmd).await
    }

    pub async fn sunion<T: FromValue, K: AsRef<[u8]>>(&self, keys: &[K]) -> Result<Vec<T>> {
        self.execute_cmd(Cmd::new(SUNION).args_slice(keys)).await
    }

    pub async fn sunionstore<K: AsRef<[u8]>>(
        &self,
        dst: impl AsRef<[u8]>,
        keys: &[K],
    ) -> Result<u64> {
        self.execute_cmd(Cmd::new(SUNIONSTORE).arg_bytes(dst).args_slice(keys))
            .await
    }

    pub async fn sdiff<T: FromValue, K: AsRef<[u8]>>(&self, keys: &[K]) -> Result<Vec<T>> {
        self.execute_cmd(Cmd::new(SDIFF).args_slice(keys)).await
    }

    pub async fn sdiffstore<K: AsRef<[u8]>>(
        &self,
        dst: impl AsRef<[u8]>,
        keys: &[K],
    ) -> Result<u64> {
        self.execute_cmd(Cmd::new(SDIFFSTORE).arg_bytes(dst).args_slice(keys))
            .await
    }

    pub async fn sscan(
        &self,
        key: impl AsRef<[u8]>,
        cursor: u64,
        conf_li: impl AsRef<[SScan<'_>]>,
    ) -> Result<(u64, Vec<String>)> {
        let mut r#match = None;
        let mut count = None;
        let mut no_values = false;
        for conf in conf_li.as_ref() {
            match conf {
                SScan::Match(p) => r#match = Some(*p),
                SScan::Count(c) => count = Some(*c),
                SScan::NoValues => no_values = true,
            }
        }
        let cmd = apply_scan_opts(
            Cmd::new(SSCAN).arg_bytes(key).arg_int(cursor),
            r#match,
            count,
            no_values,
        );
        self.exec_scan(cmd).await
    }
}
