#[cfg(not(target_arch = "wasm32"))]
use tokio::net::TcpListener;

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> webc_api::Result<()> {
    let app = webc_api::router();
    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    println!("listening on http://{}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {}
