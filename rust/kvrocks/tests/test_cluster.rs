use kvrocks::{SlotMap, crc16, hash_tag, slot};

#[test]
fn test_crc16_and_slot() -> aok::Void {
    assert_eq!(crc16(b"123456789"), 0x31c3);

    // Hash tag
    assert_eq!(hash_tag(b"user1000"), b"user1000");
    assert_eq!(hash_tag(b"{user1000}.account"), b"user1000");
    assert_eq!(hash_tag(b"foo{}{bar}"), b"foo{}{bar}");

    let s1 = slot(b"{user1000}.account");
    let s2 = slot(b"{user1000}.profile");
    assert_eq!(s1, s2);
    aok::OK
}

#[test]
fn test_slot_map() -> aok::Void {
    let map = SlotMap::new();
    map.update_range(0, 5000, "127.0.0.1:7000");
    map.update_range(5001, 10000, "127.0.0.1:7001");
    map.update_range(10001, 16383, "127.0.0.1:7002");

    assert_eq!(map.get_node(100).as_deref(), Some("127.0.0.1:7000"));
    assert_eq!(map.get_node(6000).as_deref(), Some("127.0.0.1:7001"));
    assert_eq!(map.get_node(12000).as_deref(), Some("127.0.0.1:7002"));
    aok::OK
}

#[tokio::test]
async fn test_real_cluster_commands() -> aok::Void {
    let conf = kvrocks::Config {
        server: Some(kvrocks::ServerConfig::Cluster {
            nodes: vec![
                kvrocks::Server {
                    host: "127.0.0.1".into(),
                    port: 7000,
                },
                kvrocks::Server {
                    host: "127.0.0.1".into(),
                    port: 7001,
                },
                kvrocks::Server {
                    host: "127.0.0.1".into(),
                    port: 7002,
                },
            ],
        }),
        username: None,
        password: None,
        database: None,
    };

    let client = kvrocks::client(conf).await?;

    let ping_res = client.ping(None).await?;
    assert_eq!(ping_res, "PONG");

    // 跨槽位读写测试
    client.set("cluster_test_k1", "val1", &[]).await?;
    let v1: Option<String> = client.get("cluster_test_k1").await?;
    assert_eq!(v1, Some("val1".into()));

    client.set("cluster_test_k2", "val2", &[]).await?;
    let v2: Option<String> = client.get("cluster_test_k2").await?;
    assert_eq!(v2, Some("val2".into()));

    let _ = client.del("cluster_test_k1").await;
    let _ = client.del("cluster_test_k2").await;

    aok::OK
}
