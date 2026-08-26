use kvrocks::{
    Cmd,
    connection::{AutoPipelineDriver, SenderHandle},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, duplex},
    sync::mpsc,
};

#[tokio::test]
async fn test_auto_pipeline_mock() -> aok::Void {
    let (client_io, mut server_io) = duplex(4096);
    let (tx, rx) = mpsc::unbounded_channel();
    let handle = SenderHandle::new(tx);
    let driver = AutoPipelineDriver::new(client_io, rx);

    tokio::spawn(driver.run());

    // 模拟服务端响应任务
    tokio::spawn(async move {
        let mut buf = [0u8; 1024];
        let n = server_io.read(&mut buf).await.unwrap();
        assert!(n > 0);

        server_io.write_all(b"+PONG\r\n:100\r\n").await.unwrap();
        server_io.flush().await.unwrap();
    });

    let h1 = handle.clone();
    let h2 = handle.clone();

    let (r1, r2) = tokio::join!(
        async move { h1.execute(Cmd::new("PING")).await.unwrap() },
        async move { h2.execute(Cmd::new("INCR").arg("cnt")).await.unwrap() }
    );

    assert_eq!(r1.into_string().unwrap(), "PONG");
    assert_eq!(r2.as_i64().unwrap(), 100);

    aok::OK
}
