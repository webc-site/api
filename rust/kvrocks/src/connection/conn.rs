use crate::{
    adapter::connect_adapter,
    client::helper::build_auth_cmd,
    connection::SenderHandle,
    error::Result,
    resp3::{
        Cmd,
        constants::{AUTH, HELLO, SELECT},
    },
};

fn build_hello_cmd(username: Option<&str>, password: Option<&str>) -> Cmd {
    let mut cmd = Cmd::new(HELLO).arg("3");
    if let Some(pass) = password {
        cmd = cmd
            .arg(AUTH)
            .arg(username.unwrap_or("default").as_bytes())
            .arg(pass.as_bytes());
    }
    cmd
}

pub struct Connection;

impl Connection {
    pub async fn create_and_handshake(
        addr: &str,
        username: Option<&str>,
        password: Option<&str>,
        database: Option<u8>,
    ) -> Result<SenderHandle> {
        let handle = connect_adapter(addr).await?;

        // 1. 发送 HELLO 3 握手并可附带 AUTH
        let hello_cmd = build_hello_cmd(username, password);
        if let Err(e) = handle.execute(hello_cmd).await {
            // 某些兼容 Redis/Kvrocks 旧版可能需要先 AUTH 再 HELLO 3
            if let Some(pass) = password {
                handle.execute(build_auth_cmd(username, pass)).await?;
                handle.execute(build_hello_cmd(None, None)).await?;
            } else {
                return Err(e);
            }
        }

        // 2. 选择数据库（若非 0）
        if let Some(db) = database
            && db > 0
        {
            handle.execute(Cmd::new(SELECT).arg_int(db)).await?;
        }

        Ok(handle)
    }
}
