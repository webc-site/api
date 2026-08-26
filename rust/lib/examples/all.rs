use wasm_bindgen::prelude::*;
use webc_lib::{
    db::{env_conf, surreal},
    kv::client_from_env,
};

#[wasm_bindgen]
pub async fn db_kv_demo(key: String, val: String) -> Result<String, JsValue> {
    // 1. 操作 KV
    let kv = client_from_env("KV")
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    kv.set(&key, &val, &[])
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let kv_res: Option<String> = kv
        .get(&key)
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    // 2. 操作 DB
    let (conf, db_name) = env_conf();
    let ns = surreal(conf);
    let db = ns.db(db_name.unwrap_or_else(|| "test".to_string()));
    let db_res: Option<u64> = db
        .q1("RETURN 1", &())
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    Ok(format!("kv={:?}, db={:?}", kv_res, db_res))
}

fn main() {}
