use kvrocks::{Config, Server, ServerConfig};

const KVROCKS_PASS: &str = "kvrocks_secret_pass";
const SENTINEL_PASS: &str = "sentinel_secret_pass";

#[tokio::test]
async fn test_real_kvrocks_auth_success() -> aok::Void {
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

    let client = kvrocks::client(conf).await?;

    // 1. PING
    let pong = client.ping(None).await?;
    assert_eq!(pong, "PONG");

    // 2. SET & GET
    client.set("auth_test_key", "auth_val", &[]).await?;

    let val: Option<String> = client.get("auth_test_key").await?;
    assert_eq!(val, Some("auth_val".into()));

    // 3. HSET & HGET
    let hset_res = client.hset("auth_test_hash", "f1", "v1").await?;
    assert!(hset_res >= 1 || hset_res == 0);

    let hval: Option<String> = client.hget("auth_test_hash", "f1").await?;
    assert_eq!(hval, Some("v1".into()));

    // 4. DEL
    let del_res = client.del("auth_test_key").await?;
    assert_eq!(del_res, 1);
    let _ = client.del("auth_test_hash").await;

    aok::OK
}

#[tokio::test]
async fn test_real_kvrocks_auth_failure() -> aok::Void {
    let conf = Config {
        server: Some(ServerConfig::Centralized {
            server: Server {
                host: "127.0.0.1".into(),
                port: 6667,
            },
        }),
        username: None,
        password: Some("wrong_password".into()),
        database: None,
    };

    let res = kvrocks::client(conf).await;
    assert!(res.is_err());

    aok::OK
}

#[tokio::test]
async fn test_real_sentinel_with_auth_query() -> aok::Void {
    let sent_conf = kvrocks::SentinelConfig::new("mymaster", vec!["127.0.0.1:26379".into()])
        .auth(None, Some(SENTINEL_PASS.into()));

    let master_addr = kvrocks::SentinelManager::resolve_master(&sent_conf).await?;
    assert!(!master_addr.is_empty());

    aok::OK
}

#[tokio::test]
async fn test_real_kvrocks_concurrency_pipeline() -> aok::Void {
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

    let client = kvrocks::client(conf).await?;

    let mut tasks = Vec::new();
    for i in 0..50 {
        let c = client.clone();
        tasks.push(tokio::spawn(async move {
            let key = format!("concurrent_auth_key_{}", i);
            let val = format!("val_{}", i);
            c.set(&key, &val, &[]).await.unwrap();
            let res: Option<String> = c.get(&key).await.unwrap();
            assert_eq!(res, Some(val));
            c.del(&key).await.unwrap();
        }));
    }

    for t in tasks {
        t.await.unwrap();
    }

    aok::OK
}

#[tokio::test]
async fn test_real_standalone_no_auth() -> aok::Void {
    let conf = Config {
        server: Some(ServerConfig::Centralized {
            server: Server {
                host: "127.0.0.1".into(),
                port: 6665,
            },
        }),
        username: None,
        password: None,
        database: None,
    };

    let client = kvrocks::client(conf).await?;

    let pong = client.ping(None).await?;
    assert_eq!(pong, "PONG");

    client.set("no_auth_k", "no_auth_v", &[]).await?;
    let val: Option<String> = client.get("no_auth_k").await?;
    assert_eq!(val, Some("no_auth_v".into()));
    client.del("no_auth_k").await?;

    aok::OK
}
