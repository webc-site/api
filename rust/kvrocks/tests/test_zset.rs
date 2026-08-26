mod common;
use common::get_client;
use kvrocks::client::{Aggregate, PopDirection, ZAddOption, ZRangeByScore, ZRangeStore, ZScan};

#[tokio::test]
async fn test_all_zset_commands() -> aok::Void {
    let client = get_client().await?;

    let key = "test_zset_comprehensive";
    let key2 = "test_zset_k2";
    let dst = "test_zset_dst";
    let _ = client.mdel(&[key, key2, dst]).await;

    // 1. zadd, zadd_opt, zcard, zscore, zmscore
    assert_eq!(
        client
            .zadd(key, &[(10.0, "alice"), (20.0, "bob"), (30.0, "charlie")])
            .await?,
        3
    );
    assert_eq!(
        client
            .zadd_opt(
                key,
                &[(15.0, "alice")],
                &[ZAddOption::Xx, ZAddOption::Gt, ZAddOption::Ch]
            )
            .await?,
        1
    );
    assert_eq!(client.zcard(key).await?, 3);
    assert_eq!(client.zscore(key, "alice").await?, Some(15.0));
    let mscores = client.zmscore(key, &["alice", "non_exist"]).await?;
    assert_eq!(mscores, vec![Some(15.0), None]);

    // 2. zincrby, zrank, zrevrank
    assert_eq!(client.zincrby(key, 5.0, "alice").await?, 20.0);
    assert_eq!(client.zrank(key, "alice").await?, Some(0)); // tie broken by lex
    assert_eq!(client.zrevrank(key, "charlie").await?, Some(0));

    // 3. zcount, zlexcount
    assert_eq!(client.zcount(key, "10", "+inf").await?, 3);
    assert_eq!(client.zlexcount(key, "[a", "[z").await?, 3);

    // 4. zrange, zrevrange, zrange_withscores
    let r1: Vec<String> = client.zrange(key, 0, -1).await?;
    assert_eq!(r1.len(), 3);
    let r2: Vec<String> = client.zrevrange(key, 0, -1).await?;
    assert_eq!(r2.len(), 3);
    let range_scores = client.zrange_withscores(key, 0, 1).await?;
    assert_eq!(range_scores.len(), 2);

    // 5. zrangebyscore, zrevrangebyscore, zrangebylex, zrevrangebylex
    let rbs: Vec<String> = client
        .zrangebyscore(key, "10", "20", &[ZRangeByScore::Limit(0, 2)])
        .await?;
    assert_eq!(rbs, vec!["alice", "bob"]);
    let rev_rbs: Vec<String> = client.zrevrangebyscore(key, "20", "10", &[]).await?;
    assert_eq!(rev_rbs, vec!["bob", "alice"]);

    let rbl: Vec<String> = client.zrangebylex(key, "-", "+", &[]).await?;
    assert_eq!(rbl.len(), 3);
    let rev_rbl: Vec<String> = client.zrevrangebylex(key, "+", "-", &[]).await?;
    assert_eq!(rev_rbl.len(), 3);

    // 6. zrangestore
    let _ = client
        .zrangestore(dst, key, "0", "1", &[ZRangeStore::ByScore])
        .await;

    // 7. zpopmin, zpopmax, bzpopmin, bzpopmax
    client
        .zadd(key2, &[(1.0, "m1"), (2.0, "m2"), (3.0, "m3"), (4.0, "m4")])
        .await?;
    let pop_min: Vec<(String, f64)> = client.zpopmin(key2, Some(1)).await?;
    assert_eq!(pop_min, vec![("m1".into(), 1.0)]);
    let pop_max: Vec<(String, f64)> = client.zpopmax(key2, Some(1)).await?;
    assert_eq!(pop_max, vec![("m4".into(), 4.0)]);
    let _ = client.bzpopmin::<String, _>(&[key2], 1.0).await;
    let _ = client.bzpopmax::<String, _>(&[key2], 1.0).await;

    // 8. zrandmember
    let rand_one: Vec<String> = client.zrandmember(key, None).await?;
    assert!(!rand_one.is_empty());
    let rand_cnt: Vec<String> = client.zrandmember(key, Some(2)).await?;
    assert_eq!(rand_cnt.len(), 2);

    // 9. zdiff, zdiffstore, zinter, zinterstore, zintercard, zunion, zunionstore (with Weights & Aggregate)
    client.zadd(key, &[(10.0, "a"), (20.0, "b")]).await?;
    client.zadd(key2, &[(20.0, "b"), (30.0, "c")]).await?;

    let diff: Vec<String> = client.zdiff(&[key, key2]).await?;
    assert_eq!(diff, vec!["a", "alice", "bob", "charlie"]);
    let _ = client.zdiffstore(dst, &[key, key2]).await;

    let inter: Vec<String> = client.zinter(&[key, key2], None, None).await?;
    assert_eq!(inter, vec!["b"]);
    assert_eq!(
        client
            .zinterstore(dst, &[key, key2], Some(&[1.0, 2.0]), Some(Aggregate::Max))
            .await?,
        1
    );
    assert_eq!(client.zintercard(&[key, key2], None).await?, 1);

    let union: Vec<String> = client.zunion(&[key, key2], None, None).await?;
    assert!(union.len() >= 4);
    let _ = client
        .zunionstore(dst, &[key, key2], Some(&[1.0, 1.0]), Some(Aggregate::Sum))
        .await;

    // 10. zremrangebyrank, zremrangebyscore, zremrangebylex, zrem, zscan, zmpop, bzmpop
    let _ = client.zremrangebyrank(key, 0, 0).await;
    let _ = client.zremrangebyscore(key, "10", "15").await;
    let _ = client.zremrangebylex(key, "[a", "[b").await;
    let _ = client.zrem(key, &["a", "b"]).await;

    let (cur, scanned) = client
        .zscan(key, 0, &[ZScan::Match("*"), ZScan::Count(10)])
        .await?;
    let _ = (cur, scanned);

    client
        .zadd(key, &[(100.0, "zpop_item1"), (101.0, "zpop_item2")])
        .await?;
    let _ = client.zmpop(&[key], PopDirection::Min, Some(1)).await;
    let _ = client.bzmpop(1.0, &[key], PopDirection::Max, Some(1)).await;

    let _ = client.mdel(&[key, key2, dst]).await;

    aok::OK
}
