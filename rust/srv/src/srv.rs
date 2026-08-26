use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use axum::extract::State;
use axum::http::{StatusCode, Uri};
use axum::response::IntoResponse;
use axum::routing::any;
use genv::{s, static_init};
use log::{error, info};
use tokio::net::TcpListener;

use crate::error::Result;
use crate::runtime::WasmEngine;

s!(
    LISTEN_ADDR: String | "0.0.0.0:8080".to_string();
    WASM_PATH: String | "dist/api.wasm".to_string();
);

pub async fn srv() -> Result<()> {
    let wasm_path = PathBuf::from(&*WASM_PATH);

    info!("Loading WASM module from {:?}", wasm_path);
    let engine = WasmEngine::new(&wasm_path)?;
    info!("WASM runtime initialized with Pooling Allocator");

    let app = Router::new()
        .fallback(any(handle_request))
        .with_state(engine);

    let listener = TcpListener::bind(&*LISTEN_ADDR).await?;
    info!("Server listening on http://{}", listener.local_addr()?);

    axum::serve(listener, app).await?;
    Ok(())
}

async fn handle_request(State(wasm): State<Arc<WasmEngine>>, uri: Uri) -> impl IntoResponse {
    let _permit = match wasm.semaphore.acquire().await {
        Ok(p) => p,
        Err(e) => {
            error!("Concurrency acquire error: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Concurrency acquire error: {e}"),
            );
        }
    };

    let mut store = wasm.new_store();
    match wasm
        .linker
        .instantiate_async(&mut store, &wasm.module)
        .await
    {
        Ok(instance) => {
            log::debug!("Instantiated WASM instance for request: {}", uri);
            let version = instance
                .get_typed_func::<(), i32>(&mut store, "api_version")
                .ok();

            if let Some(version_func) = version {
                match version_func.call_async(&mut store, ()).await {
                    Ok(v) => (
                        StatusCode::OK,
                        format!("WASM API (version: {v}) handled: {uri}"),
                    ),
                    Err(e) => {
                        error!("WASM call error: {e}");
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("WASM call error: {e}"),
                        )
                    }
                }
            } else {
                (StatusCode::OK, format!("Handled by WASM: {uri}"))
            }
        }
        Err(e) => {
            error!("WASM instantiation error: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Instantiation error: {e}"),
            )
        }
    }
}
