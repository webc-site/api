use std::env::var;

use webc_lib::KV;

#[tokio::test]
async fn test_kv_client() {
    if var("KV_REDIS").is_err() && var("KV_SENTINEL").is_err() {
        println!("Skip test_kv_client: no KV_REDIS or KV_SENTINEL configured");
        return;
    }

    let pong = KV.ping(None).await.expect("ping failed");
    assert_eq!(pong, "PONG");

    let key = format!("test_key_{}", rand::random::<u64>());
    let val = rand::random::<u64>().to_string();

    KV.set(&key, &val, &[]).await.expect("set failed");

    let res: Option<String> = KV.get(&key).await.expect("get failed");
    assert_eq!(res, Some(val));

    let _: u64 = KV.del(&key).await.expect("del failed");
}
