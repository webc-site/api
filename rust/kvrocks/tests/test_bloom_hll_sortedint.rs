mod common;
use common::get_client;
use kvrocks::client::{BfInsert, BfReserve, CfReserve, SiRange, SiRangeByValue};

#[tokio::test]
async fn test_all_bloom_hll_sortedint_commands() -> aok::Void {
    let client = get_client().await?;

    // 1. Bloom Filter (8 commands)
    let bf = "test_bf_all_k";
    let _ = client.del(bf).await;
    if client
        .bf_reserve(
            bf,
            0.01,
            1000,
            &[BfReserve::NonScaling, BfReserve::Expansion(2)],
        )
        .await
        .is_ok()
    {
        assert!(client.bf_add(bf, "item1").await?);
        let madd = client.bf_madd(bf, &["item2", "item3"]).await?;
        assert_eq!(madd, vec![true, true]);

        assert!(client.bf_exists(bf, "item1").await?);
        let mexists = client.bf_mexists(bf, &["item1", "item_none"]).await?;
        assert_eq!(mexists, vec![true, false]);

        let _ = client.bf_info(bf).await;
        let _ = client.bf_card(bf).await;
        let _ = client
            .bf_insert(
                bf,
                &["item4", "item5"],
                &[
                    BfInsert::Capacity(1000),
                    BfInsert::Error(0.01),
                    BfInsert::NonScaling,
                ],
            )
            .await;
    }
    let _ = client.del(bf).await;

    // 2. Cuckoo Filter (2 commands)
    let cf = "test_cf_all_k";
    let _ = client.del(cf).await;
    if client
        .cf_reserve(cf, 1000, &[CfReserve::Expansion(1)])
        .await
        .is_ok()
    {
        assert!(client.cf_add(cf, "cf_val1").await?);
    }
    let _ = client.del(cf).await;

    // 3. HyperLogLog (3 commands)
    let hll1 = "test_hll_all_1";
    let hll2 = "test_hll_all_2";
    let hll_dst = "test_hll_all_dst";
    let _ = client.mdel(&[hll1, hll2, hll_dst]).await;
    client.pfadd(hll1, &["a", "b", "c"]).await?;
    client.pfadd(hll2, &["c", "d", "e"]).await?;
    assert!(client.pfcount(&[hll1]).await? >= 3);
    assert!(client.pfcount(&[hll1, hll2]).await? >= 5);
    client.pfmerge(hll_dst, &[hll1, hll2]).await?;
    let _ = client.mdel(&[hll1, hll2, hll_dst]).await;

    // 4. SortedInt (8 commands)
    let si = "test_si_all_k";
    let _ = client.del(si).await;
    if let Ok(cnt) = client.siadd(si, &[10, 20, 30, 40, 50]).await {
        assert_eq!(cnt, 5);
        assert_eq!(client.sicard(si).await?, 5);
        assert!(client.siexists(si, 30).await?);
        assert!(!client.siexists(si, 99).await?);

        let range = client.sirange(si, 0, 2, &[SiRange::Cursor(0)]).await?;
        assert!(!range.is_empty());
        let revrange = client.sirevrange(si, 0, 2, &[]).await?;
        assert!(!revrange.is_empty());

        let _ = client
            .sirangebyvalue(si, 10, 30, &[SiRangeByValue::Limit(0, 2)])
            .await;
        let _ = client.sirevrangebyvalue(si, 30, 10, &[]).await;

        assert_eq!(client.sirem(si, &[10, 20]).await?, 2);
    }
    let _ = client.del(si).await;

    aok::OK
}
