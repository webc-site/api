use std::future::Future;

use tokio::net::TcpStream;

use crate::{adapter::stream::BoxedStream, error::Result};

pub async fn connect_tcp(addr: &str) -> Result<BoxedStream> {
    let stream = TcpStream::connect(addr).await?;
    let _ = stream.set_nodelay(true);
    Ok(BoxedStream(Box::pin(stream)))
}

pub fn spawn_task<F>(future: F)
where
    F: Future<Output = ()> + Send + 'static,
{
    tokio::spawn(future);
}
