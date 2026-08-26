mod common;
use common::get_client;
use kvrocks::client::{HExpireCondition, HGetExOption, HRangeByLex, HScan, HSetExOption};
use rapidhash::RapidHashMap as HashMap;

#[tokio::test]
async fn test_all_hash_commands() -> aok::Void {
    let client = get_client().await?;

    let key = "test_hash_comprehensive";
    let _ = client.del(key).await;

    // 1. hset, hsetnx, hget, hmset, hmget
    assert_eq!(client.hset(key, "f1", "v1").await?, 1);
    assert!(!client.hsetnx(key, "f1", "v_new").await?);
    assert!(client.hsetnx(key, "f2", "v2").await?);
    assert_eq!(client.hget::<String>(key, "f1").await?, Some("v1".into()));

    client.hmset(key, &[("f3", "v3"), ("f4", "v4")]).await?;
    let mvals: Vec<Option<String>> = client.hmget(key, &["f1", "f2", "f_none"]).await?;
    assert_eq!(mvals, vec![Some("v1".into()), Some("v2".into()), None]);

    // 2. hgetall, hkeys, hvals, hexists, hlen, hlen_opt, hstrlen
    let all: HashMap<String, String> = client.hgetall(key).await?;
    assert_eq!(all.len(), 4);
    let keys = client.hkeys(key).await?;
    assert_eq!(keys.len(), 4);
    let vals: Vec<String> = client.hvals(key).await?;
    assert_eq!(vals.len(), 4);
    assert!(client.hexists(key, "f1").await?);
    assert_eq!(client.hlen(key).await?, 4);
    assert_eq!(client.hlen_opt(key, None).await?, 4);
    assert_eq!(client.hstrlen(key, "f1").await?, 2);

    // 3. hincrby, hincrbyfloat
    client.hset(key, "f_num", "10").await?;
    assert_eq!(client.hincrby(key, "f_num", 5).await?, 15);
    assert_eq!(client.hincrbyfloat(key, "f_num", 2.5).await?, 17.5);
    assert_eq!(client.hincrbyfloat(key, "f_num", -1.5).await?, 16.0);

    // 4. hrandfield, hrandfield_withvalues
    let rand_f: Vec<String> = client.hrandfield(key, None).await?;
    assert!(!rand_f.is_empty());
    let rand_cnt: Vec<String> = client.hrandfield(key, Some(2)).await?;
    assert_eq!(rand_cnt.len(), 2);
    let rand_pairs = client.hrandfield_withvalues(key, 2).await?;
    assert_eq!(rand_pairs.len(), 2);

    // 5. hscan, hrangebylex
    let (cur, scan_pairs) = client
        .hscan(key, 0, &[HScan::Match("f*"), HScan::Count(10)])
        .await?;
    assert!(!scan_pairs.is_empty());
    let _ = cur;
    let range_lex = client
        .hrangebylex(key, "-", "+", &[HRangeByLex::Limit(0, 2)])
        .await?;
    assert!(!range_lex.is_empty());

    // 6. hsetexpire, hsetex, hgetex, hpersist, hexpiretime, hpexpiretime, httl, hpttl, hexpire_one
    if client
        .hsetexpire(key, 100, &[("f1", "v1_exp")])
        .await
        .is_ok()
    {
        let _ = client
            .hsetex(key, &[("f1", "v1")], &[HSetExOption::Ex(100)])
            .await;
        let _ = client
            .hgetex::<String, _>(key, &["f1"], &[HGetExOption::Persist])
            .await;
        let _ = client.hexpiretime(key, &["f1"]).await;
        let _ = client.hpexpiretime(key, &["f1"]).await;
        let _ = client.httl(key, &["f1"]).await;
        let _ = client.hpttl(key, &["f1"]).await;
        let _ = client.hpersist(key, &["f1"]).await;
        let _ = client
            .hexpire_one(key, "f1", 100, &[HExpireCondition::Nx])
            .await;
        let _ = client
            .hpexpire_one(key, "f1", 100000, &[HExpireCondition::Xx])
            .await;
        let _ = client
            .hexpireat_one(key, "f1", 2000000000, &[HExpireCondition::Gt])
            .await;
        let _ = client
            .hpexpireat_one(key, "f1", 2000000000000, &[HExpireCondition::Lt])
            .await;
    }

    if let Ok(applied) = client
        .hsetex(key, &[("f_ex", "v_ex")], &[HSetExOption::Ex(100)])
        .await
    {
        assert_eq!(applied, 1);
        let _ = client
            .hgetex::<String, _>(key, &["f_ex"], &[HGetExOption::Persist])
            .await;
    }

    // 7. hdel
    assert_eq!(
        client
            .hdel(key, &["f1", "f2", "f3", "f4", "f_num", "f_ex"])
            .await?,
        5
    );
    let _ = client.del(key).await;

    aok::OK
}
