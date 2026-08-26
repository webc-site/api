use crate::{
    client::Client,
    error::Result,
    resp3::{
        Cmd, Value,
        constants::{ASKING, CLUSTER, CLUSTERX, READONLY, READWRITE},
    },
};

impl Client {
    pub async fn cluster<A: AsRef<[u8]>>(&self, subcommand: &str, args: &[A]) -> Result<Value> {
        self.execute(Cmd::new(CLUSTER).arg_bytes(subcommand).args_slice(args))
            .await
    }

    /// Kvrocks specific: CLUSTERX
    pub async fn clusterx<A: AsRef<[u8]>>(&self, subcommand: &str, args: &[A]) -> Result<Value> {
        self.execute(Cmd::new(CLUSTERX).arg_bytes(subcommand).args_slice(args))
            .await
    }

    pub async fn readonly(&self) -> Result<()> {
        self.execute_cmd(Cmd::new(READONLY)).await
    }

    pub async fn readwrite(&self) -> Result<()> {
        self.execute_cmd(Cmd::new(READWRITE)).await
    }

    pub async fn asking(&self) -> Result<()> {
        self.execute_cmd(Cmd::new(ASKING)).await
    }
}
