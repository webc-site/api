mod common;
use common::get_client;
use kvrocks::client::JsonSet;

#[tokio::test]
async fn test_all_json_commands() -> aok::Void {
    let client = get_client().await?;

    let key = "test_json_all_k";
    let key2 = "test_json_all_k2";
    let _ = client.mdel(&[key, key2]).await;

    // 1. json_set, json_get, json_mget, json_type
    if client
        .json_set(
            key,
            "$",
            "{\"name\":\"kvrocks\",\"count\":1,\"arr\":[1,2],\"flag\":false}",
            &[JsonSet::Nx],
        )
        .await
        .is_ok()
    {
        let val = client.json_get::<String, _>(key, &["$.name"]).await?;
        assert!(val.is_some());

        let _ = client.json_set(key2, "$", "{\"age\":20}", &[]).await;
        let mget_res = client.json_mget(&[key, key2], "$").await?;
        assert_eq!(mget_res.len(), 2);

        let types = client.json_type(key, Some("$.name")).await?;
        assert!(!types.is_null());

        // 2. json_numincrby, json_nummultby
        let _ = client.json_numincrby(key, "$.count", 5.0).await;
        let _ = client.json_nummultby(key, "$.count", 2.0).await;

        // 3. json_strappend, json_strlen
        let _ = client
            .json_strappend(key, Some("$.name"), "\"rocks\"")
            .await;
        let _ = client.json_strlen(key, Some("$.name")).await;

        // 4. json_arrlen, json_arrappend, json_arrinsert, json_arrpop, json_arrtrim, json_arrindex
        let _ = client.json_arrlen(key, Some("$.arr")).await;
        let _ = client.json_arrappend(key, "$.arr", &["3"]).await;
        let _ = client.json_arrinsert(key, "$.arr", 0, &["0"]).await;
        let _ = client.json_arrindex(key, "$.arr", "1", None, None).await;
        let _ = client.json_arrpop(key, Some("$.arr"), None).await;
        let _ = client.json_arrtrim(key, "$.arr", 0, 1).await;

        // 5. json_objkeys, json_objlen
        let _ = client.json_objkeys(key, Some("$")).await;
        let _ = client.json_objlen(key, Some("$")).await;

        // 6. json_toggle, json_clear, json_forget, json_merge, json_mset, json_resp, json_debug, json_info
        let _ = client.json_toggle(key, "$.flag").await;
        let _ = client.json_merge(key, "$", "{\"merged\":true}").await;
        let _ = client.json_mset(&[(key, "$.extra", "\"extra_val\"")]).await;
        let _ = client.json_resp(key, Some("$")).await;
        let _ = client.json_debug("MEMORY", key, Some("$")).await;
        let _ = client.json_info(key).await;
        let _ = client.json_clear(key, Some("$.arr")).await;
        let _ = client.json_forget(key, Some("$.merged")).await;
        let _ = client.json_del(key, Some("$")).await;
    }

    let _ = client.mdel(&[key, key2]).await;

    aok::OK
}
