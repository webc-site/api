use std::future::Future;

use crate::{
    adapter::stream::BoxedStream,
    error::{Error, Result},
};

pub async fn connect_tcp(_addr: &str) -> Result<BoxedStream> {
    // Wasm 目标下若无 direct TCP，可通过 JS websocket/worker socket 适配
    Err(Error::Config(
        "Direct TCP in wasm requires worker socket transport adapter".into(),
    ))
}

pub fn spawn_task<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    wasm_bindgen_futures::spawn_local(future);
}
