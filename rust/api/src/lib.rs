pub mod error;
pub use error::{Error, Result};

use axum::{Router, routing::get};
#[cfg(target_arch = "wasm32")]
use {
    axum::body::Body, http::Request, http_body_util::BodyExt, tower_service::Service,
    wasm_bindgen::prelude::*,
};

pub fn router() -> Router {
    Router::new().route("/", get(root))
}

async fn root() -> &'static str {
    "webc-api"
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn handle(uri: String) -> core::result::Result<String, JsValue> {
    let mut app = router();
    let req = Request::builder()
        .uri(uri)
        .body(Body::empty())
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let res = app
        .call(req)
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let bytes = res
        .into_body()
        .collect()
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?
        .to_bytes();
    String::from_utf8(bytes.to_vec()).map_err(|e| JsValue::from_str(&e.to_string()))
}
