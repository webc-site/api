pub mod adapter;
pub mod client;
pub mod cluster;
pub mod connection;
pub mod error;
pub mod resp3;
pub mod sentinel;

pub use client::{
    Client, Config, DEFAULT_REDIS_PORT, DEFAULT_SENTINEL_PORT, Server, ServerConfig, conf_from_env,
    server_li,
};
pub use cluster::{SlotMap, crc16, hash_tag, slot};
pub use error::{Error, Result};
pub use rapidhash::RapidHashMap;
pub use resp3::{Cmd, Decoder, FromValue, Value};
pub use sentinel::{SentinelConfig, SentinelManager};

pub fn client_lazy(conf: Config) -> Client {
    Client::new(conf)
}

pub fn lazy_from_env(prefix: impl AsRef<str>) -> Client {
    Client::from_env(prefix)
}

pub async fn client(conf: Config) -> Result<Client> {
    Client::connect(conf).await
}

pub async fn client_from_env(prefix: impl AsRef<str>) -> Result<Client> {
    let conf = conf_from_env(prefix);
    Client::connect(conf).await
}

pub async fn conn(prefix: impl AsRef<str>) -> Result<Client> {
    client_from_env(prefix).await
}

pub async fn connect(
    server: ServerConfig,
    username: Option<String>,
    password: Option<String>,
    database: Option<u8>,
) -> Result<Client> {
    let conf = Config {
        server: Some(server),
        username,
        password,
        database,
    };
    Client::connect(conf).await
}
