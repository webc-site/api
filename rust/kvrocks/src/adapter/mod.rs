pub mod stream;

#[cfg(not(target_arch = "wasm32"))]
pub mod bin;
#[cfg(not(target_arch = "wasm32"))]
pub use bin::{connect_tcp, spawn_task};

#[cfg(target_arch = "wasm32")]
pub mod wasm;
#[cfg(target_arch = "wasm32")]
pub use wasm::{connect_tcp, spawn_task};

pub use stream::{AsyncStream, BoxedStream};
use tokio::sync::mpsc;

use crate::{
    connection::auto_pipeline::{AutoPipelineDriver, SenderHandle},
    error::Result,
};

pub fn spawn_driver<S: AsyncStream>(stream: S) -> SenderHandle {
    let (tx, rx) = mpsc::unbounded_channel();
    let handle = SenderHandle::new(tx);
    let driver = AutoPipelineDriver::new(stream, rx);

    spawn_task(driver.run());

    handle
}

pub async fn connect_adapter(addr: &str) -> Result<SenderHandle> {
    let stream = connect_tcp(addr).await?;
    Ok(spawn_driver(stream))
}
