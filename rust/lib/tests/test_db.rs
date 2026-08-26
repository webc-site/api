use webc_lib::{DB, NS};

#[tokio::test]
async fn test_db_ping() {
    let auth_res = NS.auth(false).await;
    assert!(auth_res.is_ok());

    let sur_db = NS.db("i");
    let res1: Vec<u64> = sur_db.q("RETURN 1;", &()).await.expect("query failed");
    assert_eq!(res1, vec![1]);

    let res2: Vec<u64> = DB.q("RETURN 1;", &()).await.expect("query failed");
    assert_eq!(res2, vec![1]);
}
