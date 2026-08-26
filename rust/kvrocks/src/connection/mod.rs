pub mod auto_pipeline;
pub mod conn;

pub use auto_pipeline::{AutoPipelineDriver, Request, SenderHandle};
pub use conn::Connection;
