mod common;
use common::get_client;
use kvrocks::client::{XAddOption, XAutoClaim, XClaim, XReadGroup};

#[tokio::test]
async fn test_all_stream_commands() -> aok::Void {
    let client = get_client().await?;

    let key = "test_stream_all_k";
    let group = "test_stream_grp";
    let consumer = "c1";
    let _ = client.del(key).await;

    // 1. xadd, xadd_opt, xlen, xrange, xrevrange
    let id1 = client.xadd(key, "*", &[("f1", "v1")]).await?.unwrap();
    let id2 = client
        .xadd_opt(key, "*", &[("f2", "v2")], &[XAddOption::Nomkstream])
        .await?
        .unwrap();
    let id3 = client
        .xadd_opt(key, "*", &[("f3", "v3")], &[XAddOption::MaxLen(100, true)])
        .await?
        .unwrap();
    let _ = id3;
    assert_eq!(client.xlen(key).await?, 3);

    let range = client.xrange(key, "-", "+", None).await?;
    assert!(range.into_array().unwrap().len() >= 3);
    let revrange = client.xrevrange(key, "+", "-", Some(2)).await?;
    assert_eq!(revrange.into_array().unwrap().len(), 2);

    // 2. xread
    let read_res = client.xread(Some(2), None, &[(key, "0-0")]).await?;
    assert!(read_res.into_array().is_ok());

    // 3. xgroup_create, xreadgroup, xack, xpending, xclaim, xautoclaim, xinfo, xsetid
    if client.xgroup_create(key, group, "0", false).await.is_ok() {
        let _ = client
            .xreadgroup(group, consumer, &[(key, ">")], &[XReadGroup::Count(2)])
            .await;
        let acked = client.xack(key, group, &[&id1]).await?;
        assert_eq!(acked, 1);

        let _ = client.xpending(key, group, None, None).await;
        let _ = client
            .xclaim(
                key,
                group,
                consumer,
                0,
                &[&id2],
                &[XClaim::Idle(100), XClaim::Time(100)],
            )
            .await;
        let _ = client
            .xautoclaim(
                key,
                group,
                consumer,
                0,
                "0-0",
                &[XAutoClaim::Count(5), XAutoClaim::JustId],
            )
            .await;

        let _ = client.xinfo_stream(key).await;
        let _ = client.xinfo_groups(key).await;
        let _ = client.xinfo_consumers(key, group).await;

        let _ = client.xgroup_setid(key, group, "0").await;
        let _ = client.xgroup_destroy(key, group).await;
    }

    let _ = client.xsetid(key, "9999999999999-0").await;

    // 4. xtrim, xdel
    let _ = client.xtrim(key, 1, false).await;
    let _ = client.xdel(key, &[&id2]).await;

    let _ = client.del(key).await;

    aok::OK
}
