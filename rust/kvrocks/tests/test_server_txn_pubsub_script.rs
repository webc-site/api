mod common;
use common::get_client;

#[tokio::test]
async fn test_all_server_txn_pubsub_script_commands() -> aok::Void {
    let client = get_client().await?;

    // 1. Txn (5 commands)
    let _ = client.watch(&["txn_k"]).await;
    let _ = client.unwatch().await;
    let _ = client.multi().await;
    let _ = client.discard().await;
    let _ = client.exec().await;

    // 2. Server (44 commands)
    let _ = client.auth(None, common::KVROCKS_PASS).await;
    assert_eq!(client.ping(None).await?, "PONG");
    let _ = client.select(0).await;
    assert!(!client.info(None).await?.is_empty());
    assert!(client.role().await?.into_array().is_ok());

    let _ = client.config_get("maxclients").await;
    let _ = client.config_set("slowlog-log-slower-than", "10000").await;
    let _ = client.config_resetstat().await;
    let _ = client.config_rewrite().await;

    let ns = "test_ns_all";
    let token = "test_token_all";
    if client.namespace_add(ns, token).await.is_ok() {
        let _ = client.namespace_set(ns, "test_token_new").await;
        let _ = client.namespace_get(None).await;
        let _ = client.namespace_del(ns).await;
    }

    let _ = client.compact(false).await;
    let _ = client.bgsave().await;
    let _ = client.lastsave().await;
    let _ = client.slowlog_get(Some(5)).await;
    let _ = client.slowlog_len().await;
    client.slowlog_reset().await?;

    let _ = client.client_list(None).await;
    client.client_setname("all_cmd_tester").await?;
    assert_eq!(
        client.client_getname().await?,
        Some("all_cmd_tester".into())
    );

    let _ = client.disk_usage("k").await;
    let _ = client.memory_usage("k").await;
    let _ = client.kprofile("all", &["1"]).await;
    let _ = client.perflog::<&str>("1", &[]).await;
    let _ = client.hello(3, None).await;
    let _ = client.dbsize().await;
    let _ = client.time().await;
    let _ = client.stats().await;
    let _ = client.flushmemtable(false).await;
    let _ = client.flushblockcache().await;
    let _ = client.latency::<&str>("latest", &[]).await;
    let _ = client.rdb("DUMP", &["test_rdb"]).await;

    // 3. PubSub (9 commands)
    let ch = "test_all_ch";
    let _ = client.publish(ch, "msg1").await;
    let _ = client.mpublish(ch, &["msg2", "msg3"]).await;
    let _ = client.pubsub_channels(None).await;
    let _ = client.pubsub_numsub(&[ch]).await;
    let _ = client.pubsub_numpat().await;
    let _ = client.pubsub_shardchannels(None).await;
    let _ = client.pubsub_shardnumsub(&[ch]).await;

    // 4. Script & Function (5 + 3 commands)
    let script = "return redis.call('PING')";
    let _ = client.eval::<String, &str, &str>(script, &[], &[]).await;
    let _ = client.eval_ro::<String, &str, &str>(script, &[], &[]).await;
    if let Ok(sha) = client.script_load(script).await {
        let _ = client.script_exists(&[&sha]).await;
        let _ = client.evalsha::<String, &str, &str>(&sha, &[], &[]).await;
        let _ = client
            .evalsha_ro::<String, &str, &str>(&sha, &[], &[])
            .await;
        let _ = client.script_flush().await;
    }

    let code = "#!lua name=alllib\nredis.register_function('f1', function(k, a) return 'ok' end)";
    if client.function_load(code, true).await.is_ok() {
        let _ = client.fcall::<String, &str, &str>("f1", &[], &[]).await;
        let _ = client.fcall_ro::<String, &str, &str>("f1", &[], &[]).await;
        let _ = client.function_list(Some("alllib"), false).await;
        let _ = client.function_delete("alllib").await;
    }

    // 5. Cluster & Replication (5 + 6 commands)
    let _ = client.cluster::<&str>("INFO", &[]).await;
    let _ = client.clusterx::<&str>("NODES", &[]).await;
    let _ = client.readonly().await;
    let _ = client.readwrite().await;
    let _ = client.asking().await;

    let _ = client.wait(1, 100).await;
    let _ = client._db_name().await;

    aok::OK
}
