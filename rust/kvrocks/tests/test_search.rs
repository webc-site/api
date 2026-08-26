mod common;
use common::get_client;
use kvrocks::client::{FtDropIndex, FtSearch};

#[tokio::test]
async fn test_all_search_commands() -> aok::Void {
    let client = get_client().await?;

    let idx = "test_ft_all_idx";
    let _ = client.ft_dropindex(idx, &[]).await;

    if client
        .ft_create(
            idx,
            &[
                "ON", "JSON", "PREFIX", "1", "item:", "SCHEMA", "title", "TEXT", "price", "NUMERIC",
            ],
        )
        .await
        .is_ok()
    {
        let l1 = client.ft_list().await?;
        assert!(l1.contains(&idx.to_string()));

        let _ = client
            .ft_search(idx, "*", &[FtSearch::Limit(0, 10), FtSearch::NoContent])
            .await;
        let _ = client.ft_searchsql("SELECT * FROM test_ft_all_idx").await;
        let _ = client.ft_explain(idx, "*").await;
        let _ = client.ft_explainsql("SELECT * FROM test_ft_all_idx").await;
        let _ = client.ft_info(idx).await;
        let _ = client.ft_tagvals(idx, "title").await;

        let _ = client.ft_dropindex(idx, &[FtDropIndex::Dd]).await;
    }

    aok::OK
}
