mod common;
use common::get_client;
use kvrocks::client::{TsAlter, TsCreate, TsIncrBy, TsMGet, TsMRange, TsRange};

#[tokio::test]
async fn test_all_timeseries_commands() -> aok::Void {
    let client = get_client().await?;

    let key = "test_ts_all_1";
    let key2 = "test_ts_all_2";
    let _ = client.mdel(&[key, key2]).await;

    // 1. ts_create, ts_alter, ts_add, ts_madd, ts_get
    if client
        .ts_create(
            key,
            &[
                TsCreate::Retention(3600),
                TsCreate::ChunkSize(2048),
                TsCreate::DuplicatePolicy("last"),
                TsCreate::Labels(&[("sensor", "temp"), ("loc", "room1")]),
            ],
        )
        .await
        .is_ok()
    {
        let _ = client
            .ts_alter(
                key,
                &[
                    TsAlter::Retention(7200),
                    TsAlter::ChunkSize(4096),
                    TsAlter::DuplicatePolicy("max"),
                ],
            )
            .await;

        assert_eq!(client.ts_add(key, "1000", 25.0).await?, 1000);
        let madd_res = client
            .ts_madd(&[(key, "1001", 26.0), (key, "1002", 27.0)])
            .await?;
        assert_eq!(madd_res, vec![1001, 1002]);

        let _ = client.ts_get(key).await;

        // 2. ts_range, ts_revrange, ts_info, ts_queryindex
        let range = client
            .ts_range(key, "1000", "1002", &[TsRange::Count(10)])
            .await?;
        assert!(!range.is_empty());
        let revrange = client.ts_revrange(key, "1000", "1002", &[]).await?;
        assert!(!revrange.is_empty());

        let info = client.ts_info(key).await?;
        assert!(!info.is_null());

        let idx = client.ts_queryindex(&["sensor=temp"]).await?;
        assert!(idx.contains(&key.to_string()));

        // 3. ts_mget, ts_mrange, ts_mrevrange
        let _ = client
            .ts_mget(&["sensor=temp"], &[TsMGet::WithLabels])
            .await;
        let _ = client
            .ts_mrange(
                "1000",
                "1002",
                &["sensor=temp"],
                &[TsMRange::WithLabels, TsMRange::Count(10)],
            )
            .await;
        let _ = client
            .ts_mrevrange("1000", "1002", &["sensor=temp"], &[])
            .await;

        // 4. ts_createrule, ts_deleterule, ts_incrby, ts_decrby, ts_del
        if client.ts_create(key2, &[]).await.is_ok()
            && client.ts_createrule(key, key2, "avg", 10).await.is_ok()
        {
            let _ = client.ts_deleterule(key, key2).await;
        }

        let _ = client
            .ts_incrby(key, 1.0, &[TsIncrBy::Timestamp("2000")])
            .await;
        let _ = client
            .ts_decrby(key, 1.0, &[TsIncrBy::Timestamp("2001")])
            .await;
        let _ = client.ts_del(key, 1000, 1001).await;
    }

    let _ = client.mdel(&[key, key2]).await;

    aok::OK
}
