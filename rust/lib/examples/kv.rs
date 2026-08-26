use wasm_bindgen::prelude::*;
use webc_lib::kv::client_from_env;

#[wasm_bindgen]
pub async fn kv_demo(key: String, val: String) -> Result<Option<String>, JsValue> {
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
    Ok(kv_res)
}

fn main() {}
