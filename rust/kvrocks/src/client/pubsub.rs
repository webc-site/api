use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{
    adapter::{BoxedStream, connect_tcp},
    client::{Client, Topology, helper::build_auth_cmd},
    error::{Error, Result},
    resp3::{
        Cmd, Decoder, Value,
        constants::{
            CHANNELS, MPUBLISH, NUMPAT, NUMSUB, PSUBSCRIBE, PUBLISH, PUBSUB, PUNSUBSCRIBE,
            SHARDCHANNELS, SHARDNUMSUB, SSUBSCRIBE, SUBSCRIBE, SUNSUBSCRIBE, UNSUBSCRIBE,
        },
    },
};

pub struct PubSubStream {
    stream: BoxedStream,
    read_buf: BytesMut,
}

impl PubSubStream {
    pub async fn next_message(&mut self) -> Result<Option<Value>> {
        loop {
            if let Some(val) = Decoder::decode(&mut self.read_buf)? {
                return Ok(Some(val));
            }
            let n = self.stream.read_buf(&mut self.read_buf).await?;
            if n == 0 {
                return Ok(None);
            }
        }
    }
}

impl Client {
    pub async fn publish(
        &self,
        channel: impl AsRef<[u8]>,
        message: impl AsRef<[u8]>,
    ) -> Result<u64> {
        self.execute_cmd(Cmd::new(PUBLISH).arg_bytes(channel).arg_bytes(message))
            .await
    }

    /// Kvrocks specific: MPUBLISH <channel> <message> [message ...]
    pub async fn mpublish<M: AsRef<[u8]>>(
        &self,
        channel: impl AsRef<[u8]>,
        messages: &[M],
    ) -> Result<u64> {
        self.execute_cmd(Cmd::new(MPUBLISH).arg_bytes(channel).args_slice(messages))
            .await
    }

    pub async fn subscribe<C: AsRef<[u8]>>(&self, channels: &[C]) -> Result<Value> {
        self.execute(Cmd::new(SUBSCRIBE).args_slice(channels)).await
    }

    pub async fn subscribe_stream<C: AsRef<[u8]>>(&self, channels: &[C]) -> Result<PubSubStream> {
        let addr = match &self.inner.topology {
            Topology::Standalone { addr, .. } => addr.clone(),
            Topology::Sentinel {
                conf,
                current_master,
            } => {
                let _ = self.get_sentinel_conn(conf, current_master).await?;
                if let Some(m) = current_master.load_full() {
                    m.0.clone()
                } else {
                    return Err(Error::Config("no sentinel master available".into()));
                }
            }
            Topology::Cluster { nodes, .. } => nodes
                .first()
                .cloned()
                .unwrap_or_else(|| "127.0.0.1:6379".into()),
        };

        let mut stream = connect_tcp(&addr).await?;
        if let Some(pass) = &self.inner.password {
            let auth_cmd = build_auth_cmd(self.inner.username.as_deref(), pass);
            let mut buf = BytesMut::new();
            auth_cmd.encode(&mut buf);
            stream.write_all(&buf).await?;
            stream.flush().await?;
            let mut read_buf = BytesMut::new();
            loop {
                stream.read_buf(&mut read_buf).await?;
                if let Some(v) = Decoder::decode(&mut read_buf)? {
                    match v {
                        Value::Error(e) | Value::BlobError(e) => return Err(Error::Redis(e)),
                        _ => break,
                    }
                }
            }
        }

        let sub_cmd = Cmd::new(SUBSCRIBE).args_slice(channels);
        let mut buf = BytesMut::new();
        sub_cmd.encode(&mut buf);
        stream.write_all(&buf).await?;
        stream.flush().await?;

        Ok(PubSubStream {
            stream,
            read_buf: BytesMut::with_capacity(4096),
        })
    }

    pub async fn unsubscribe<C: AsRef<[u8]>>(&self, channels: &[C]) -> Result<Value> {
        self.execute(Cmd::new(UNSUBSCRIBE).args_slice(channels))
            .await
    }

    pub async fn psubscribe<P: AsRef<[u8]>>(&self, patterns: &[P]) -> Result<Value> {
        self.execute(Cmd::new(PSUBSCRIBE).args_slice(patterns))
            .await
    }

    pub async fn punsubscribe<P: AsRef<[u8]>>(&self, patterns: &[P]) -> Result<Value> {
        self.execute(Cmd::new(PUNSUBSCRIBE).args_slice(patterns))
            .await
    }

    pub async fn ssubscribe<C: AsRef<[u8]>>(&self, shardchannels: &[C]) -> Result<Value> {
        self.execute(Cmd::new(SSUBSCRIBE).args_slice(shardchannels))
            .await
    }

    pub async fn sunsubscribe<C: AsRef<[u8]>>(&self, shardchannels: &[C]) -> Result<Value> {
        self.execute(Cmd::new(SUNSUBSCRIBE).args_slice(shardchannels))
            .await
    }

    pub async fn pubsub_channels(&self, pattern: Option<&str>) -> Result<Vec<String>> {
        self.execute_cmd(Cmd::new(PUBSUB).arg(CHANNELS).arg_opt_bytes(pattern))
            .await
    }

    pub async fn pubsub_numsub<C: AsRef<[u8]>>(
        &self,
        channels: &[C],
    ) -> Result<Vec<(String, u64)>> {
        let cmd = Cmd::new(PUBSUB).arg(NUMSUB).args_slice(channels);
        self.exec_pair_array(cmd).await
    }

    pub async fn pubsub_numpat(&self) -> Result<u64> {
        self.execute_cmd(Cmd::new(PUBSUB).arg(NUMPAT)).await
    }

    pub async fn pubsub_shardchannels(&self, pattern: Option<&str>) -> Result<Vec<String>> {
        self.execute_cmd(Cmd::new(PUBSUB).arg(SHARDCHANNELS).arg_opt_bytes(pattern))
            .await
    }

    pub async fn pubsub_shardnumsub<C: AsRef<[u8]>>(
        &self,
        shardchannels: &[C],
    ) -> Result<Vec<(String, u64)>> {
        let cmd = Cmd::new(PUBSUB).arg(SHARDNUMSUB).args_slice(shardchannels);
        self.exec_pair_array(cmd).await
    }
}
