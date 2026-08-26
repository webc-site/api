use std::collections::VecDeque;

use bytes::BytesMut;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::{mpsc, oneshot},
};

use crate::{
    error::{Error, Result},
    resp3::{Cmd, Decoder, Value},
};

pub struct Request {
    pub cmd: Cmd,
    pub responder: oneshot::Sender<Result<Value>>,
}

#[derive(Clone, Debug)]
pub struct SenderHandle {
    tx: mpsc::UnboundedSender<Request>,
}

impl SenderHandle {
    pub fn new(tx: mpsc::UnboundedSender<Request>) -> Self {
        Self { tx }
    }

    pub async fn execute(&self, cmd: Cmd) -> Result<Value> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(Request { cmd, responder: tx })
            .map_err(|_| Error::ConnectionClosed)?;

        rx.await.map_err(|_| Error::ConnectionClosed)?
    }

    pub fn is_same(&self, other: &Self) -> bool {
        self.tx.same_channel(&other.tx)
    }
}

pub struct AutoPipelineDriver<S> {
    stream: S,
    rx: mpsc::UnboundedReceiver<Request>,
    pending: VecDeque<oneshot::Sender<Result<Value>>>,
    read_buf: BytesMut,
    write_buf: BytesMut,
}

impl<S> AutoPipelineDriver<S>
where
    S: AsyncReadExt + AsyncWriteExt + Unpin,
{
    pub fn new(stream: S, rx: mpsc::UnboundedReceiver<Request>) -> Self {
        Self {
            stream,
            rx,
            pending: VecDeque::with_capacity(64),
            read_buf: BytesMut::with_capacity(8192),
            write_buf: BytesMut::with_capacity(8192),
        }
    }

    pub async fn run(mut self) {
        loop {
            tokio::select! {
              biased;

              // 1. 处理待发送请求（微批处理 auto-pipelining）
              req_opt = self.rx.recv() => {
                match req_opt {
                  Some(req) => {
                    self.write_buf.clear();
                    req.cmd.encode(&mut self.write_buf);
                    self.pending.push_back(req.responder);

                    // 尝试排空当前积压的所有请求，一次性批量写入
                    while let Ok(next_req) = self.rx.try_recv() {
                      next_req.cmd.encode(&mut self.write_buf);
                      self.pending.push_back(next_req.responder);
                    }

                    if let Err(e) = self.stream.write_all(&self.write_buf).await {
                      self.fail_all(Error::Io(e));
                      break;
                    }
                    if let Err(e) = self.stream.flush().await {
                      self.fail_all(Error::Io(e));
                      break;
                    }
                  }
                  None => {
                    // 发送端全部关闭
                    break;
                  }
                }
              }

              // 2. 从连接中读取响应数据并解码 (零拷贝 read_buf 直接读取)
              read_res = self.stream.read_buf(&mut self.read_buf) => {
                match read_res {
                  Ok(0) => {
                    // 对端关闭连接
                    self.fail_all(Error::ConnectionClosed);
                    break;
                  }
                  Ok(_) => {
                    // 循环解码所有已接收完整的 RESP3 帧
                    loop {
                      match Decoder::decode(&mut self.read_buf) {
                        Ok(Some(val)) => {
                          if let Some(responder) = self.pending.pop_front() {
                            let res = Self::convert_resp_error(val);
                            let _ = responder.send(res);
                          }
                        }
                        Ok(None) => break,
                        Err(e) => {
                          self.fail_all(e);
                          return;
                        }
                      }
                    }
                  }
                  Err(e) => {
                    self.fail_all(Error::Io(e));
                    break;
                  }
                }
              }
            }
        }
    }

    fn convert_resp_error(val: Value) -> Result<Value> {
        match val {
            Value::Error(s) | Value::BlobError(s) => {
                if let Some(err) =
                    parse_redirect(&s, "MOVED ", |slot, addr| Error::Moved { slot, addr }).or_else(
                        || parse_redirect(&s, "ASK ", |slot, addr| Error::Ask { slot, addr }),
                    )
                {
                    return Err(err);
                }
                Err(Error::Redis(s))
            }
            _ => Ok(val),
        }
    }

    fn fail_all(&mut self, err: Error) {
        while let Some(responder) = self.pending.pop_front() {
            let _ = responder.send(Err(err.clone()));
        }
    }
}

fn parse_redirect(s: &str, prefix: &str, ctor: impl FnOnce(u16, String) -> Error) -> Option<Error> {
    let rest = s.strip_prefix(prefix)?;
    let mut parts = rest.split_ascii_whitespace();
    let slot = parts.next()?.parse::<u16>().ok()?;
    let addr = parts.next()?.to_string();
    Some(ctor(slot, addr))
}
