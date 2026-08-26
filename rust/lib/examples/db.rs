use wasm_bindgen::prelude::*;
use webc_lib::db::{env_conf, surreal};

#[wasm_bindgen]
pub async fn db_demo() -> Result<Option<u64>, JsValue> {
    let (conf, db_name) = env_conf();
    let ns = surreal(conf);
    let db = ns.db(db_name.unwrap_or_else(|| "test".to_string()));
    let db_res: Option<u64> = db
        .q1("RETURN 1", &())
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(db_res)
}

fn main() {}
