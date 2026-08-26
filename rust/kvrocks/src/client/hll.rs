use crate::{
    client::Client,
    error::Result,
    resp3::{
        Cmd,
        constants::{PFADD, PFCOUNT, PFMERGE},
    },
};

impl Client {
    pub async fn pfadd<E: AsRef<[u8]>>(
        &self,
        key: impl AsRef<[u8]>,
        elements: &[E],
    ) -> Result<bool> {
        self.execute_cmd(Cmd::new(PFADD).arg_bytes(key).args_slice(elements))
            .await
    }

    pub async fn pfcount<K: AsRef<[u8]>>(&self, keys: &[K]) -> Result<u64> {
        self.execute_cmd(Cmd::new(PFCOUNT).args_slice(keys)).await
    }

    pub async fn pfmerge<K: AsRef<[u8]>>(
        &self,
        destkey: impl AsRef<[u8]>,
        sourcekeys: &[K],
    ) -> Result<()> {
        self.execute_cmd(Cmd::new(PFMERGE).arg_bytes(destkey).args_slice(sourcekeys))
            .await
    }
}
