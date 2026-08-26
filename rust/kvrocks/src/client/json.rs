use crate::{
    client::Client,
    error::Result,
    resp3::{
        Cmd, FromValue, Value,
        constants::{
            JSON_ARRAPPEND, JSON_ARRINDEX, JSON_ARRINSERT, JSON_ARRLEN, JSON_ARRPOP, JSON_ARRTRIM,
            JSON_CLEAR, JSON_DEBUG, JSON_DEL, JSON_FORGET, JSON_GET, JSON_INFO, JSON_MERGE,
            JSON_MGET, JSON_MSET, JSON_NUMINCRBY, JSON_NUMMULTBY, JSON_OBJKEYS, JSON_OBJLEN,
            JSON_RESP, JSON_SET, JSON_STRAPPEND, JSON_STRLEN, JSON_TOGGLE, JSON_TYPE, NX, OK, XX,
        },
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonSet {
    Nx,
    Xx,
}

fn json_key_path_cmd(
    name: &'static str,
    key: impl AsRef<[u8]>,
    path: Option<impl AsRef<[u8]>>,
) -> Cmd {
    Cmd::new(name).arg_bytes(key).arg_opt_bytes(path)
}

impl Client {
    pub async fn json_set(
        &self,
        key: impl AsRef<[u8]>,
        path: impl AsRef<[u8]>,
        json_value: impl AsRef<[u8]>,
        conf_li: impl AsRef<[JsonSet]>,
    ) -> Result<bool> {
        let mut cmd = Cmd::new(JSON_SET)
            .arg_bytes(key)
            .arg_bytes(path)
            .arg_bytes(json_value);
        for conf in conf_li.as_ref() {
            match conf {
                JsonSet::Nx => {
                    cmd = cmd.arg(NX);
                }
                JsonSet::Xx => {
                    cmd = cmd.arg(XX);
                }
            }
        }
        let res = self.execute(cmd).await?;
        match res {
            Value::SimpleString(s) if s == OK => Ok(true),
            Value::Null => Ok(false),
            v => bool::from_value(v),
        }
    }

    pub async fn json_get<T: FromValue, P: AsRef<[u8]>>(
        &self,
        key: impl AsRef<[u8]>,
        paths: &[P],
    ) -> Result<Option<T>> {
        self.execute_cmd(Cmd::new(JSON_GET).arg_bytes(key).args_slice(paths))
            .await
    }

    pub async fn json_info(&self, key: impl AsRef<[u8]>) -> Result<Value> {
        self.execute(Cmd::new(JSON_INFO).arg_bytes(key)).await
    }

    pub async fn json_del(
        &self,
        key: impl AsRef<[u8]>,
        path: Option<impl AsRef<[u8]>>,
    ) -> Result<u64> {
        self.execute_cmd(json_key_path_cmd(JSON_DEL, key, path))
            .await
    }

    pub async fn json_forget(
        &self,
        key: impl AsRef<[u8]>,
        path: Option<impl AsRef<[u8]>>,
    ) -> Result<u64> {
        self.execute_cmd(json_key_path_cmd(JSON_FORGET, key, path))
            .await
    }

    pub async fn json_type(
        &self,
        key: impl AsRef<[u8]>,
        path: Option<impl AsRef<[u8]>>,
    ) -> Result<Value> {
        self.execute(json_key_path_cmd(JSON_TYPE, key, path)).await
    }

    pub async fn json_arrlen(
        &self,
        key: impl AsRef<[u8]>,
        path: Option<impl AsRef<[u8]>>,
    ) -> Result<Value> {
        self.execute(json_key_path_cmd(JSON_ARRLEN, key, path))
            .await
    }

    pub async fn json_arrappend<V: AsRef<[u8]>>(
        &self,
        key: impl AsRef<[u8]>,
        path: impl AsRef<[u8]>,
        values: &[V],
    ) -> Result<Value> {
        let cmd = Cmd::new(JSON_ARRAPPEND)
            .arg_bytes(key)
            .arg_bytes(path)
            .args_slice(values);
        self.execute(cmd).await
    }

    pub async fn json_arrinsert<V: AsRef<[u8]>>(
        &self,
        key: impl AsRef<[u8]>,
        path: impl AsRef<[u8]>,
        index: i64,
        values: &[V],
    ) -> Result<Value> {
        let cmd = Cmd::new(JSON_ARRINSERT)
            .arg_bytes(key)
            .arg_bytes(path)
            .arg_int(index)
            .args_slice(values);
        self.execute(cmd).await
    }

    pub async fn json_arrpop(
        &self,
        key: impl AsRef<[u8]>,
        path: Option<impl AsRef<[u8]>>,
        index: Option<i64>,
    ) -> Result<Value> {
        let cmd = json_key_path_cmd(JSON_ARRPOP, key, path).arg_opt_int(index);
        self.execute(cmd).await
    }

    pub async fn json_arrtrim(
        &self,
        key: impl AsRef<[u8]>,
        path: impl AsRef<[u8]>,
        start: i64,
        stop: i64,
    ) -> Result<Value> {
        let cmd = Cmd::new(JSON_ARRTRIM)
            .arg_bytes(key)
            .arg_bytes(path)
            .arg_int(start)
            .arg_int(stop);
        self.execute(cmd).await
    }

    pub async fn json_arrindex(
        &self,
        key: impl AsRef<[u8]>,
        path: impl AsRef<[u8]>,
        value: impl AsRef<[u8]>,
        start: Option<i64>,
        stop: Option<i64>,
    ) -> Result<Value> {
        let cmd = Cmd::new(JSON_ARRINDEX)
            .arg_bytes(key)
            .arg_bytes(path)
            .arg_bytes(value)
            .arg_opt_int(start)
            .arg_opt_int(stop);
        self.execute(cmd).await
    }

    pub async fn json_toggle(
        &self,
        key: impl AsRef<[u8]>,
        path: impl AsRef<[u8]>,
    ) -> Result<Value> {
        self.execute(Cmd::new(JSON_TOGGLE).arg_bytes(key).arg_bytes(path))
            .await
    }

    pub async fn json_clear(
        &self,
        key: impl AsRef<[u8]>,
        path: Option<impl AsRef<[u8]>>,
    ) -> Result<u64> {
        self.execute_cmd(json_key_path_cmd(JSON_CLEAR, key, path))
            .await
    }

    pub async fn json_merge(
        &self,
        key: impl AsRef<[u8]>,
        path: impl AsRef<[u8]>,
        value: impl AsRef<[u8]>,
    ) -> Result<()> {
        self.execute_cmd(
            Cmd::new(JSON_MERGE)
                .arg_bytes(key)
                .arg_bytes(path)
                .arg_bytes(value),
        )
        .await
    }

    pub async fn json_numincrby(
        &self,
        key: impl AsRef<[u8]>,
        path: impl AsRef<[u8]>,
        number: f64,
    ) -> Result<String> {
        let cmd = Cmd::new(JSON_NUMINCRBY)
            .arg_bytes(key)
            .arg_bytes(path)
            .arg_float(number);
        self.execute_cmd(cmd).await
    }

    pub async fn json_nummultby(
        &self,
        key: impl AsRef<[u8]>,
        path: impl AsRef<[u8]>,
        number: f64,
    ) -> Result<String> {
        let cmd = Cmd::new(JSON_NUMMULTBY)
            .arg_bytes(key)
            .arg_bytes(path)
            .arg_float(number);
        self.execute_cmd(cmd).await
    }

    pub async fn json_objkeys(
        &self,
        key: impl AsRef<[u8]>,
        path: Option<impl AsRef<[u8]>>,
    ) -> Result<Value> {
        self.execute(json_key_path_cmd(JSON_OBJKEYS, key, path))
            .await
    }

    pub async fn json_objlen(
        &self,
        key: impl AsRef<[u8]>,
        path: Option<impl AsRef<[u8]>>,
    ) -> Result<Value> {
        self.execute(json_key_path_cmd(JSON_OBJLEN, key, path))
            .await
    }

    pub async fn json_strlen(
        &self,
        key: impl AsRef<[u8]>,
        path: Option<impl AsRef<[u8]>>,
    ) -> Result<Value> {
        self.execute(json_key_path_cmd(JSON_STRLEN, key, path))
            .await
    }

    pub async fn json_strappend(
        &self,
        key: impl AsRef<[u8]>,
        path: Option<impl AsRef<[u8]>>,
        value: impl AsRef<[u8]>,
    ) -> Result<Value> {
        let cmd = json_key_path_cmd(JSON_STRAPPEND, key, path).arg_bytes(value);
        self.execute(cmd).await
    }

    pub async fn json_mget<K: AsRef<[u8]>>(
        &self,
        keys: &[K],
        path: impl AsRef<[u8]>,
    ) -> Result<Vec<Option<String>>> {
        let cmd = Cmd::new(JSON_MGET).args_slice(keys).arg_bytes(path);
        self.execute_cmd(cmd).await
    }

    pub async fn json_mset<K: AsRef<[u8]>, P: AsRef<[u8]>, V: AsRef<[u8]>>(
        &self,
        items: &[(K, P, V)],
    ) -> Result<()> {
        let mut cmd = Cmd::new(JSON_MSET);
        for (k, p, v) in items {
            cmd = cmd.arg_bytes(k).arg_bytes(p).arg_bytes(v);
        }
        self.execute_cmd(cmd).await
    }

    pub async fn json_debug(
        &self,
        subcommand: &str,
        key: impl AsRef<[u8]>,
        path: Option<impl AsRef<[u8]>>,
    ) -> Result<Value> {
        let cmd = Cmd::new(JSON_DEBUG)
            .arg_bytes(subcommand)
            .arg_bytes(key)
            .arg_opt_bytes(path);
        self.execute(cmd).await
    }

    pub async fn json_resp(
        &self,
        key: impl AsRef<[u8]>,
        path: Option<impl AsRef<[u8]>>,
    ) -> Result<Value> {
        self.execute(json_key_path_cmd(JSON_RESP, key, path)).await
    }
}
