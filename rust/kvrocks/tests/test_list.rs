mod common;
use common::get_client;
use kvrocks::client::{InsertDirection, LPos, ListDirection};

#[tokio::test]
async fn test_all_list_commands() -> aok::Void {
    let client = get_client().await?;

    let key = "test_list_comprehensive";
    let key2 = "test_list_dest";
    let _ = client.mdel(&[key, key2]).await;

    // 1. rpush, lpush, rpushx, lpushx, llen, lrange
    assert_eq!(client.rpush(key, &["a", "b", "c"]).await?, 3);
    assert_eq!(client.lpush(key, &["first"]).await?, 4);
    assert_eq!(client.rpushx(key, &["d"]).await?, 5);
    assert_eq!(client.lpushx(key, &["zero"]).await?, 6);
    assert_eq!(client.llen(key).await?, 6);

    let range: Vec<String> = client.lrange(key, 0, -1).await?;
    assert_eq!(range, vec!["zero", "first", "a", "b", "c", "d"]);

    // 2. lpop, rpop, lpop_count, rpop_count
    assert_eq!(client.lpop::<String>(key).await?, Some("zero".into()));
    assert_eq!(client.rpop::<String>(key).await?, Some("d".into()));
    let popped_l: Vec<String> = client.lpop_count(key, 2).await?;
    assert_eq!(popped_l, vec!["first", "a"]);
    let popped_r: Vec<String> = client.rpop_count(key, 1).await?;
    assert_eq!(popped_r, vec!["c"]);

    // 3. lindex, lset, linsert (Before & After), lpos (Rank, MaxLen, Count)
    client.rpush(key, &["x", "y", "x"]).await?;
    assert_eq!(client.lindex::<String>(key, 0).await?, Some("b".into()));
    client.lset(key, 0, "b_mod").await?;
    assert_eq!(client.lindex::<String>(key, 0).await?, Some("b_mod".into()));
    assert_eq!(
        client
            .linsert(key, InsertDirection::Before, "x", "inserted_b")
            .await?,
        5
    );
    assert_eq!(
        client
            .linsert(key, InsertDirection::After, "y", "inserted_a")
            .await?,
        6
    );

    let pos_with_opts = client
        .lpos(key, "x", &[LPos::Rank(1), LPos::MaxLen(10)])
        .await?;
    assert!(pos_with_opts.is_some());
    let pos_multi = client.lpos_count(key, "x", 2, &[]).await?;
    assert!(!pos_multi.is_empty());

    // 4. lrem, ltrim
    assert_eq!(client.lrem(key, 1, "inserted_b").await?, 1);
    assert_eq!(client.lrem(key, -1, "inserted_a").await?, 1);
    client.ltrim(key, 0, 2).await?;

    // 5. rpoplpush, lmove (All 4 combinations: L/R to L/R)
    let _ = client.mdel(&[key, key2]).await;
    client.rpush(key, &["move_me"]).await?;
    assert_eq!(
        client.rpoplpush::<String>(key, key2).await?,
        Some("move_me".into())
    );
    let _ = client.mdel(&[key, key2]).await;
    client.rpush(key, &["m1", "m2", "m3", "m4"]).await?;
    assert_eq!(
        client.lmove::<String>(key, key2, true, true).await?,
        Some("m1".into())
    );
    assert_eq!(
        client.lmove::<String>(key, key2, true, false).await?,
        Some("m2".into())
    );
    assert_eq!(
        client.lmove::<String>(key, key2, false, true).await?,
        Some("m4".into())
    );
    assert_eq!(
        client.lmove::<String>(key, key2, false, false).await?,
        Some("m3".into())
    );

    // 6. blpop, brpop, lmpop, blmpop
    let _ = client.mdel(&[key, key2]).await;
    client.rpush(key, &["block_me"]).await?;
    let blp = client.blpop::<String, _>(&[key], 1.0).await?;
    assert_eq!(blp, Some((key.to_string(), "block_me".into())));

    client.rpush(key, &["block_me2"]).await?;
    let brp = client.brpop::<String, _>(&[key], 1.0).await?;
    assert_eq!(brp, Some((key.to_string(), "block_me2".into())));

    client.rpush(key, &["p1", "p2"]).await?;
    let _ = client.lmpop(&[key], ListDirection::Left, Some(2)).await;
    let _ = client
        .blmpop(1.0, &[key], ListDirection::Right, Some(2))
        .await;

    let _ = client.mdel(&[key, key2]).await;

    aok::OK
}
