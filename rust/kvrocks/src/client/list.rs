use crate::{
    client::Client,
    error::Result,
    resp3::{
        Cmd, FromValue, Value,
        constants::{
            AFTER, BEFORE, BLMOVE, BLMPOP, BLPOP, BRPOP, COUNT, LEFT, LINDEX, LINSERT, LLEN, LMOVE,
            LMPOP, LPOP, LPOS, LPUSH, LPUSHX, LRANGE, LREM, LSET, LTRIM, MAXLEN, RANK, RIGHT, RPOP,
            RPOPLPUSH, RPUSH, RPUSHX,
        },
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LPos {
    Rank(i64),
    Count(usize),
    MaxLen(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertDirection {
    Before,
    After,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListDirection {
    Left,
    Right,
}

fn build_lmpop_cmd<K: AsRef<[u8]>>(
    name: &'static str,
    timeout: Option<f64>,
    keys: &[K],
    dir: ListDirection,
    count: Option<usize>,
) -> Cmd {
    Cmd::new(name)
        .arg_opt_float(timeout)
        .arg_int(keys.len())
        .args_slice(keys)
        .arg(match dir {
            ListDirection::Left => LEFT,
            ListDirection::Right => RIGHT,
        })
        .arg_keyword_opt_int(COUNT, count)
}

impl Client {
    pub async fn lpush<V: AsRef<[u8]>>(&self, key: impl AsRef<[u8]>, values: &[V]) -> Result<u64> {
        self.execute_cmd(Cmd::new(LPUSH).arg_bytes(key).args_slice(values))
            .await
    }

    pub async fn rpush<V: AsRef<[u8]>>(&self, key: impl AsRef<[u8]>, values: &[V]) -> Result<u64> {
        self.execute_cmd(Cmd::new(RPUSH).arg_bytes(key).args_slice(values))
            .await
    }

    pub async fn lpushx<V: AsRef<[u8]>>(&self, key: impl AsRef<[u8]>, values: &[V]) -> Result<u64> {
        self.execute_cmd(Cmd::new(LPUSHX).arg_bytes(key).args_slice(values))
            .await
    }

    pub async fn rpushx<V: AsRef<[u8]>>(&self, key: impl AsRef<[u8]>, values: &[V]) -> Result<u64> {
        self.execute_cmd(Cmd::new(RPUSHX).arg_bytes(key).args_slice(values))
            .await
    }

    pub async fn lpop<T: FromValue>(&self, key: impl AsRef<[u8]>) -> Result<Option<T>> {
        self.execute_cmd(Cmd::new(LPOP).arg_bytes(key)).await
    }

    pub async fn lpop_count<T: FromValue>(
        &self,
        key: impl AsRef<[u8]>,
        count: usize,
    ) -> Result<Vec<T>> {
        self.execute_cmd(Cmd::new(LPOP).arg_bytes(key).arg_int(count))
            .await
    }

    pub async fn rpop<T: FromValue>(&self, key: impl AsRef<[u8]>) -> Result<Option<T>> {
        self.execute_cmd(Cmd::new(RPOP).arg_bytes(key)).await
    }

    pub async fn rpop_count<T: FromValue>(
        &self,
        key: impl AsRef<[u8]>,
        count: usize,
    ) -> Result<Vec<T>> {
        self.execute_cmd(Cmd::new(RPOP).arg_bytes(key).arg_int(count))
            .await
    }

    pub async fn lrange<T: FromValue>(
        &self,
        key: impl AsRef<[u8]>,
        start: i64,
        stop: i64,
    ) -> Result<Vec<T>> {
        let cmd = Cmd::new(LRANGE).arg_bytes(key).arg_int(start).arg_int(stop);
        self.execute_cmd(cmd).await
    }

    pub async fn llen(&self, key: impl AsRef<[u8]>) -> Result<u64> {
        self.execute_cmd(Cmd::new(LLEN).arg_bytes(key)).await
    }

    pub async fn lindex<T: FromValue>(
        &self,
        key: impl AsRef<[u8]>,
        index: i64,
    ) -> Result<Option<T>> {
        self.execute_cmd(Cmd::new(LINDEX).arg_bytes(key).arg_int(index))
            .await
    }

    pub async fn lset(
        &self,
        key: impl AsRef<[u8]>,
        index: i64,
        val: impl AsRef<[u8]>,
    ) -> Result<()> {
        let cmd = Cmd::new(LSET).arg_bytes(key).arg_int(index).arg_bytes(val);
        self.execute_cmd(cmd).await
    }

    pub async fn lrem(
        &self,
        key: impl AsRef<[u8]>,
        count: i64,
        val: impl AsRef<[u8]>,
    ) -> Result<u64> {
        let cmd = Cmd::new(LREM).arg_bytes(key).arg_int(count).arg_bytes(val);
        self.execute_cmd(cmd).await
    }

    pub async fn ltrim(&self, key: impl AsRef<[u8]>, start: i64, stop: i64) -> Result<()> {
        let cmd = Cmd::new(LTRIM).arg_bytes(key).arg_int(start).arg_int(stop);
        self.execute_cmd(cmd).await
    }

    pub async fn lpos(
        &self,
        key: impl AsRef<[u8]>,
        val: impl AsRef<[u8]>,
        conf_li: impl AsRef<[LPos]>,
    ) -> Result<Option<i64>> {
        let mut cmd = Cmd::new(LPOS).arg_bytes(key).arg_bytes(val);
        for conf in conf_li.as_ref() {
            match conf {
                LPos::Rank(r) => {
                    cmd = cmd.arg(RANK).arg_int(*r);
                }
                LPos::Count(c) => {
                    cmd = cmd.arg(COUNT).arg_int(*c);
                }
                LPos::MaxLen(m) => {
                    cmd = cmd.arg(MAXLEN).arg_int(*m);
                }
            }
        }
        self.execute_cmd(cmd).await
    }

    pub async fn lpos_count(
        &self,
        key: impl AsRef<[u8]>,
        val: impl AsRef<[u8]>,
        count: usize,
        conf_li: impl AsRef<[LPos]>,
    ) -> Result<Vec<i64>> {
        let mut cmd = Cmd::new(LPOS)
            .arg_bytes(key)
            .arg_bytes(val)
            .arg(COUNT)
            .arg_int(count);
        for conf in conf_li.as_ref() {
            match conf {
                LPos::Rank(r) => {
                    cmd = cmd.arg(RANK).arg_int(*r);
                }
                LPos::MaxLen(m) => {
                    cmd = cmd.arg(MAXLEN).arg_int(*m);
                }
                LPos::Count(_) => {}
            }
        }
        self.execute_cmd(cmd).await
    }

    pub async fn lmove<T: FromValue>(
        &self,
        src: impl AsRef<[u8]>,
        dst: impl AsRef<[u8]>,
        from_left: bool,
        to_left: bool,
    ) -> Result<Option<T>> {
        let cmd = Cmd::new(LMOVE)
            .arg_bytes(src)
            .arg_bytes(dst)
            .arg(if from_left { LEFT } else { RIGHT })
            .arg(if to_left { LEFT } else { RIGHT });
        self.execute_cmd(cmd).await
    }

    pub async fn rpoplpush<T: FromValue>(
        &self,
        src: impl AsRef<[u8]>,
        dst: impl AsRef<[u8]>,
    ) -> Result<Option<T>> {
        self.execute_cmd(Cmd::new(RPOPLPUSH).arg_bytes(src).arg_bytes(dst))
            .await
    }

    pub async fn blpop<T: FromValue, K: AsRef<[u8]>>(
        &self,
        keys: &[K],
        timeout_sec: f64,
    ) -> Result<Option<(String, T)>> {
        self.execute_cmd(Cmd::new(BLPOP).args_slice(keys).arg_float(timeout_sec))
            .await
    }

    pub async fn brpop<T: FromValue, K: AsRef<[u8]>>(
        &self,
        keys: &[K],
        timeout_sec: f64,
    ) -> Result<Option<(String, T)>> {
        self.execute_cmd(Cmd::new(BRPOP).args_slice(keys).arg_float(timeout_sec))
            .await
    }

    pub async fn linsert(
        &self,
        key: impl AsRef<[u8]>,
        dir: InsertDirection,
        pivot: impl AsRef<[u8]>,
        val: impl AsRef<[u8]>,
    ) -> Result<i64> {
        let cmd = Cmd::new(LINSERT)
            .arg_bytes(key)
            .arg(match dir {
                InsertDirection::Before => BEFORE,
                InsertDirection::After => AFTER,
            })
            .arg_bytes(pivot)
            .arg_bytes(val);
        self.execute_cmd(cmd).await
    }

    pub async fn blmove<T: FromValue>(
        &self,
        src: impl AsRef<[u8]>,
        dst: impl AsRef<[u8]>,
        from: ListDirection,
        to: ListDirection,
        timeout_sec: f64,
    ) -> Result<Option<T>> {
        let cmd = Cmd::new(BLMOVE)
            .arg_bytes(src)
            .arg_bytes(dst)
            .arg(match from {
                ListDirection::Left => LEFT,
                ListDirection::Right => RIGHT,
            })
            .arg(match to {
                ListDirection::Left => LEFT,
                ListDirection::Right => RIGHT,
            })
            .arg_float(timeout_sec);
        self.execute_cmd(cmd).await
    }

    pub async fn lmpop<K: AsRef<[u8]>>(
        &self,
        keys: &[K],
        dir: ListDirection,
        count: Option<usize>,
    ) -> Result<Value> {
        let cmd = build_lmpop_cmd(LMPOP, None, keys, dir, count);
        self.execute(cmd).await
    }

    pub async fn blmpop<K: AsRef<[u8]>>(
        &self,
        timeout_sec: f64,
        keys: &[K],
        dir: ListDirection,
        count: Option<usize>,
    ) -> Result<Value> {
        let cmd = build_lmpop_cmd(BLMPOP, Some(timeout_sec), keys, dir, count);
        self.execute(cmd).await
    }
}
