use crate::{
    client::Client,
    error::Result,
    resp3::{
        Cmd, Value,
        constants::{DISCARD, EXEC, MULTI, UNWATCH, WATCH},
    },
};

impl Client {
    pub async fn multi(&self) -> Result<()> {
        self.exec_into(Cmd::new(MULTI)).await
    }

    pub async fn discard(&self) -> Result<()> {
        self.exec_into(Cmd::new(DISCARD)).await
    }

    pub async fn exec(&self) -> Result<Option<Vec<Value>>> {
        self.execute_cmd(Cmd::new(EXEC)).await
    }

    pub async fn watch<K: AsRef<[u8]>>(&self, keys: &[K]) -> Result<()> {
        self.exec_into(Cmd::new(WATCH).args_slice(keys)).await
    }

    pub async fn unwatch(&self) -> Result<()> {
        self.exec_into(Cmd::new(UNWATCH)).await
    }
}
