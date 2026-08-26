use crate::{
    client::Client,
    error::Result,
    resp3::{
        Cmd, Value,
        constants::{DB_NAME, FETCH_FILE, FETCH_META, PSYNC, REPLCONF, WAIT},
    },
};

impl Client {
    pub async fn replconf<A: AsRef<[u8]>>(&self, args: &[A]) -> Result<Value> {
        self.execute(Cmd::new(REPLCONF).args_slice(args)).await
    }

    pub async fn psync(&self, replication_id: &str, offset: i64) -> Result<Value> {
        let cmd = Cmd::new(PSYNC).arg_bytes(replication_id).arg_int(offset);
        self.execute(cmd).await
    }

    /// Kvrocks internal replication: _FETCH_META
    pub async fn _fetch_meta(&self) -> Result<Value> {
        self.execute(Cmd::new(FETCH_META)).await
    }

    /// Kvrocks internal replication: _FETCH_FILE
    pub async fn _fetch_file(&self, file_name: &str) -> Result<Value> {
        self.execute(Cmd::new(FETCH_FILE).arg_bytes(file_name))
            .await
    }

    /// Kvrocks internal replication: _DB_NAME
    pub async fn _db_name(&self) -> Result<String> {
        self.execute_cmd(Cmd::new(DB_NAME)).await
    }

    pub async fn wait(&self, num_replicas: usize, timeout_ms: u64) -> Result<u64> {
        self.execute_cmd(Cmd::new(WAIT).arg_int(num_replicas).arg_int(timeout_ms))
            .await
    }
}
