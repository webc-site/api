mod common;
use common::get_client;
use kvrocks::client::TDigestMerge;

#[tokio::test]
async fn test_all_tdigest_commands() -> aok::Void {
    let client = get_client().await?;

    let key = "test_tdigest_all_1";
    let key2 = "test_tdigest_all_2";
    let _ = client.mdel(&[key, key2]).await;

    if client.tdigest_create(key, Some(100)).await.is_ok() {
        // 1. tdigest_add, min, max
        client
            .tdigest_add(key, &[1.0, 2.0, 3.0, 4.0, 5.0, 100.0])
            .await?;
        assert_eq!(client.tdigest_min(key).await?, Some(1.0));
        assert_eq!(client.tdigest_max(key).await?, Some(100.0));

        // 2. quantile, rank, revrank, byrank, byrevrank, trimmed_mean, cdf
        let q = client.tdigest_quantile(key, &[0.1, 0.5, 0.9, 0.99]).await?;
        assert_eq!(q.len(), 4);
        let r = client.tdigest_rank(key, &[1.0, 5.0]).await?;
        assert_eq!(r.len(), 2);
        let rr = client.tdigest_revrank(key, &[1.0, 5.0]).await?;
        assert_eq!(rr.len(), 2);
        let br = client.tdigest_byrank(key, &[0, 2]).await?;
        assert_eq!(br.len(), 2);
        let brr = client.tdigest_byrevrank(key, &[0, 2]).await?;
        assert_eq!(brr.len(), 2);
        let mean = client.tdigest_trimmed_mean(key, 0.1, 0.9).await?;
        assert!(mean > 0.0);
        let _ = client.tdigest_cdf(key, &[1.0, 50.0, 100.0]).await;

        // 3. info, merge, reset
        let info = client.tdigest_info(key).await?;
        assert!(!info.is_null());

        if client.tdigest_create(key2, Some(100)).await.is_ok() {
            client.tdigest_add(key2, &[10.0, 20.0]).await?;
            let _ = client
                .tdigest_merge(key, &[key2], Some(100), &[TDigestMerge::Override])
                .await;
        }

        client.tdigest_reset(key).await?;
    }

    let _ = client.mdel(&[key, key2]).await;

    aok::OK
}
