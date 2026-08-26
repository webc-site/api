pub use kvrocks::*;

#[cfg(not(target_arch = "wasm32"))]
use std::sync::LazyLock;

#[cfg(not(target_arch = "wasm32"))]
pub static KV: LazyLock<Client> = LazyLock::new(|| {
    let conf = conf_from_env("KV");
    Client::new(conf)
});
