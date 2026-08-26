mod common;
use common::get_client;
use kvrocks::client::{DelEx, GetEx, LcsOption, Set};

#[tokio::test]
async fn test_all_string_commands() -> aok::Void {
    let client = get_client().await?;

    let _ = client
        .mdel(&[
            "test_str_k1",
            "test_str_k2",
            "test_str_k3",
            "test_str_num",
            "test_str_str",
            "test_str_cas",
            "test_str_lcs1",
            "test_str_lcs2",
        ])
        .await;

    // 1. set, get, set_typed, set_get
    assert_eq!(
        client.set("test_str_k1", "v1", &[]).await?,
        Some("OK".into())
    );
    assert_eq!(
        client.get::<String>("test_str_k1").await?,
        Some("v1".into())
    );
    assert_eq!(
        client
            .set_typed::<String>("test_str_k1", "v1_typed", &[])
            .await?,
        Some("OK".into())
    );
    assert_eq!(
        client
            .set_get::<String>("test_str_k1", "v1_new", &[])
            .await?,
        Some("v1_typed".into())
    );

    // 2. Set options: Nx, Xx, Ex, Px, Keepttl, Get, IfEq, IfNe
    assert_eq!(
        client.set("test_str_k1", "should_fail", &[Set::Nx]).await?,
        None
    );
    assert_eq!(
        client
            .set("test_str_k1", "v1_xx", &[Set::Xx, Set::Get])
            .await?,
        Some("v1_new".into())
    );
    let _ = client
        .set(
            "test_str_k_exp",
            "exp_val",
            &[Set::Ex(100), Set::Px(100000), Set::KeepTtl],
        )
        .await;
    let _ = client
        .set(
            "test_str_k_ifeq",
            "val",
            &[
                Set::IfEq("val"),
                Set::IfNe("diff"),
                Set::IfDeq("val"),
                Set::IfDne("diff"),
            ],
        )
        .await;
    let _ = client.del("test_str_k_exp").await;
    let _ = client.del("test_str_k_ifeq").await;

    // 3. setex, psetex, setnx
    client.setex("test_str_k2", 100, "v2").await?;
    client.psetex("test_str_k3", 100000, "v3").await?;
    assert!(!client.setnx("test_str_k1", "exist").await?);
    assert!(client.setnx("test_str_k_new_nx", "val_nx").await?);
    let _ = client.del("test_str_k_new_nx").await;

    // 4. mset, msetnx, msetex, mget
    client
        .mset(&[("test_str_m1", "v1"), ("test_str_m2", "v2")])
        .await?;
    let mvals: Vec<Option<String>> = client
        .mget(&["test_str_m1", "test_str_m2", "test_str_m_none"])
        .await?;
    assert_eq!(mvals, vec![Some("v1".into()), Some("v2".into()), None]);
    assert!(
        !client
            .msetnx(&[("test_str_m1", "val"), ("test_str_m3", "val")])
            .await?
    );
    let _ = client
        .msetex(&[("test_str_m1", "v1_ex")], &[Set::Ex(60), Set::Xx])
        .await;

    // 5. append, strlen, getrange, setrange
    client.set("test_str_str", "hello", &[]).await?;
    assert_eq!(client.append("test_str_str", " world").await?, 11);
    assert_eq!(client.strlen("test_str_str").await?, 11);
    assert_eq!(client.getrange("test_str_str", 0, 4).await?, "hello");
    assert_eq!(client.setrange("test_str_str", 6, "redis").await?, 11);

    // 6. incr, decr, incrby, decrby, incrbyfloat
    client.set("test_str_num", "10", &[]).await?;
    assert_eq!(client.incr("test_str_num").await?, 11);
    assert_eq!(client.decr("test_str_num").await?, 10);
    assert_eq!(client.incrby("test_str_num", 5).await?, 15);
    assert_eq!(client.decrby("test_str_num", 5).await?, 10);
    assert_eq!(client.incrbyfloat("test_str_num", 2.5).await?, 12.5);

    // 7. getex, getdel
    let _ = client
        .getex::<String>(
            "test_str_k1",
            &[GetEx::Persist, GetEx::Ex(100), GetEx::Px(100000)],
        )
        .await;
    let del_val: Option<String> = client.getdel("test_str_k2").await?;
    assert_eq!(del_val, Some("v2".into()));

    // 8. cas, cad, del_ex (Kvrocks specific)
    client.set("test_str_cas", "old_val", &[]).await?;
    assert_eq!(
        client
            .cas("test_str_cas", "wrong_val", "new_val", None)
            .await?,
        0
    );
    assert_eq!(
        client
            .cas("test_str_cas_not_found", "old_val", "new_val", None)
            .await?,
        -1
    );
    assert_eq!(
        client
            .cas("test_str_cas", "old_val", "new_val", Some(100))
            .await?,
        1
    );
    assert_eq!(client.cad("test_str_cas", "wrong_val").await?, 0);
    assert_eq!(client.cad("test_str_cas", "new_val").await?, 1);
    client.set("test_str_delex", "delex_val", &[]).await?;
    let _ = client
        .delex(
            "test_str_delex",
            &[DelEx::IfEq("delex_val"), DelEx::IfNe("other")],
        )
        .await;

    // 9. lcs, lcs_opt (LCS algorithm)
    client.set("test_str_lcs1", "ohmytext", &[]).await?;
    client.set("test_str_lcs2", "mynewtext", &[]).await?;
    assert_eq!(
        client.lcs("test_str_lcs1", "test_str_lcs2").await?,
        "mytext"
    );
    let lcs_len = client
        .lcs_opt("test_str_lcs1", "test_str_lcs2", &[LcsOption::Len])
        .await?;
    assert!(!lcs_len.is_null());
    let lcs_idx = client
        .lcs_opt(
            "test_str_lcs1",
            "test_str_lcs2",
            &[
                LcsOption::Idx,
                LcsOption::MinMatchLen(2),
                LcsOption::WithMatchLen,
            ],
        )
        .await?;
    assert!(!lcs_idx.is_null());

    // Clean up
    let _ = client
        .mdel(&[
            "test_str_k1",
            "test_str_k3",
            "test_str_m1",
            "test_str_m2",
            "test_str_str",
            "test_str_num",
            "test_str_lcs1",
            "test_str_lcs2",
            "test_str_delex",
        ])
        .await;

    aok::OK
}
