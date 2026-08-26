pub mod db;
pub mod kv;

pub use db::{DB, NS};
#[cfg(not(target_arch = "wasm32"))]
pub use kv::KV;
