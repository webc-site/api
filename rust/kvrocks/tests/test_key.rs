mod common;
use common::get_client;
use kvrocks::client::{Scan, Sort};

#[tokio::test]
async fn test_all_key_commands() -> aok::Void {
    let client = get_client().await?;

    let k1 = "test_key_all_1";
    let k2 = "test_key_all_2";
    let _ = client.mdel(&[k1, k2]).await;

    // 1. set, exists, key_type, copy, rename, renamenx
    client.set(k1, "v1", &[]).await?;
    assert!(client.exists(k1).await?);
    assert_eq!(client.key_type(k1).await?, "string");

    assert!(client.copy(k1, k2, true).await?);
    assert!(!client.renamenx(k1, k2).await?);
    let _ = client.del(k2).await;
    client.rename(k1, k2).await?;

    // 2. keys, scan (Match, Count, Type), randomkey
    let keys_list = client.keys("test_key_all_*").await?;
    assert!(!keys_list.is_empty());
    let (cur, scanned) = client
        .scan(
            0,
            &[
                Scan::Match("test_key_all_*"),
                Scan::Count(10),
                Scan::Type("string"),
            ],
        )
        .await?;
    assert!(!scanned.is_empty());
    let _ = cur;
    let rk = client.randomkey().await?;
    assert!(rk.is_some());

    // 3. expire, pexpire, expireat, pexpireat, expiretime, pexpiretime, ttl, pttl, persist
    assert!(client.expire(k2, 100).await?);
    assert!(client.pexpire(k2, 100000).await?);
    assert!(client.expireat(k2, 2000000000).await?);
    assert!(client.pexpireat(k2, 2000000000000).await?);

    let _ = client.expiretime(k2).await;
    let _ = client.pexpiretime(k2).await;
    let ttl = client.ttl(k2).await?;
    assert!(ttl > 0);
    let pttl = client.pttl(k2).await?;
    assert!(pttl > 0);
    assert!(client.persist(k2).await?);

    // 4. dump, restore, kmetadata, move_to_db, movex
    if let Ok(Some(dumped)) = client.dump(k2).await {
        let k_restored = "test_key_all_restored";
        let _ = client.del(k_restored).await;
        let _ = client.restore(k_restored, 0, dumped, true).await;
        let _ = client.del(k_restored).await;
    }
    let _ = client.kmetadata(k2).await;
    let _ = client.move_to_db(k2, 1).await;
    let _ = client.movex(k2, "1").await;

    // 5. sort, sort_ro (Asc, Desc, Limit, Alpha, Store)
    let sort_k = "test_sort_all_k";
    let sort_dst = "test_sort_all_dst";
    let _ = client.mdel(&[sort_k, sort_dst]).await;
    client.rpush(sort_k, &["3", "1", "2"]).await?;
    let _ = client
        .sort(
            sort_k,
            &[
                Sort::Asc,
                Sort::Limit(0, 2),
                Sort::Alpha,
                Sort::Store(sort_dst),
            ],
        )
        .await;
    let _ = client
        .sort_ro(sort_k, &[Sort::Desc, Sort::Limit(0, 3)])
        .await;
    let _ = client.mdel(&[sort_k, sort_dst]).await;

    // 6. unlink, del
    let _ = client.unlink(k2).await;
    let _ = client.del(k2).await;

    aok::OK
}
