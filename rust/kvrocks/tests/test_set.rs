mod common;
use common::get_client;
use kvrocks::client::SScan;

#[tokio::test]
async fn test_all_set_commands() -> aok::Void {
    let client = get_client().await?;

    let k1 = "test_set_uniq_1";
    let k2 = "test_set_uniq_2";
    let dst = "test_set_uniq_dst";
    let _ = client.mdel(&[k1, k2, dst]).await;

    // 1. sadd, scard, sismember, smismember, smembers
    assert_eq!(client.sadd(k1, &["a", "b", "c", "d"]).await?, 4);
    assert_eq!(client.scard(k1).await?, 4);
    assert!(client.sismember(k1, "a").await?);
    assert!(!client.sismember(k1, "z").await?);
    assert_eq!(
        client.smismember(k1, &["a", "z", "b"]).await?,
        vec![true, false, true]
    );
    let members: Vec<String> = client.smembers(k1).await?;
    assert_eq!(members.len(), 4);

    // 2. srandmember (positive & negative count), spop, smove
    let rand_one: Vec<String> = client.srandmember(k1, None).await?;
    assert!(!rand_one.is_empty());
    let rand_multi: Vec<String> = client.srandmember(k1, Some(2)).await?;
    assert_eq!(rand_multi.len(), 2);
    let rand_allow_dup: Vec<String> = client.srandmember(k1, Some(-4)).await?;
    assert_eq!(rand_allow_dup.len(), 4);

    assert!(client.smove(k1, k2, "d").await?);
    assert_eq!(client.scard(k2).await?, 1);
    assert!(!client.smove(k1, k2, "non_existing_member").await?);

    let pop_res: Vec<String> = client.spop(k1, Some(1)).await?;
    assert_eq!(pop_res.len(), 1);

    // reset sets
    let _ = client.mdel(&[k1, k2, dst]).await;
    client.sadd(k1, &["1", "2", "3"]).await?;
    client.sadd(k2, &["2", "3", "4"]).await?;

    // 3. sinter, sinterstore, sintercard (with limit)
    let inter: Vec<String> = client.sinter(&[k1, k2]).await?;
    assert_eq!(inter.len(), 2);
    assert_eq!(client.sinterstore(dst, &[k1, k2]).await?, 2);
    assert_eq!(client.sintercard(&[k1, k2], None).await?, 2);
    assert_eq!(client.sintercard(&[k1, k2], Some(1)).await?, 1);

    // 4. sunion, sunionstore
    let union: Vec<String> = client.sunion(&[k1, k2]).await?;
    assert_eq!(union.len(), 4);
    assert_eq!(client.sunionstore(dst, &[k1, k2]).await?, 4);

    // 5. sdiff, sdiffstore
    let diff: Vec<String> = client.sdiff(&[k1, k2]).await?;
    assert_eq!(diff.len(), 1);
    assert_eq!(diff[0], "1");
    assert_eq!(client.sdiffstore(dst, &[k1, k2]).await?, 1);

    // 6. sscan, srem
    let (cur, scanned) = client
        .sscan(k1, 0, &[SScan::Match("*"), SScan::Count(10)])
        .await?;
    assert!(!scanned.is_empty());
    let _ = cur;

    assert_eq!(client.srem(k1, &["1", "2"]).await?, 2);

    let _ = client.mdel(&[k1, k2, dst]).await;

    aok::OK
}
