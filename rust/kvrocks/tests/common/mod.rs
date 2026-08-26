use kvrocks::{Client, Config, Server, ServerConfig, client};

pub const KVROCKS_PASS: &str = "kvrocks_secret_pass";

pub async fn get_client() -> aok::Result<Client> {
    let conf = Config {
        server: Some(ServerConfig::Centralized {
            server: Server {
                host: "127.0.0.1".into(),
                port: 6667,
            },
        }),
        username: None,
        password: Some(KVROCKS_PASS.into()),
        database: None,
    };

    Ok(client(conf).await?)
}
