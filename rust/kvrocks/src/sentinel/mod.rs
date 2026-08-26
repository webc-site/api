use crate::{
    connection::Connection,
    error::{Error, Result},
    resp3::{
        Cmd, Value,
        constants::{GET_MASTER_ADDR_BY_NAME, SENTINEL},
    },
};

#[derive(Debug, Clone)]
pub struct SentinelConfig {
    pub service_name: String,
    pub hosts: Vec<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl SentinelConfig {
    pub fn new(service_name: impl Into<String>, hosts: Vec<String>) -> Self {
        Self {
            service_name: service_name.into(),
            hosts,
            username: None,
            password: None,
        }
    }

    pub fn auth(mut self, username: Option<String>, password: Option<String>) -> Self {
        self.username = username;
        self.password = password;
        self
    }
}

pub struct SentinelManager;

impl SentinelManager {
    pub async fn resolve_master(conf: &SentinelConfig) -> Result<String> {
        if conf.hosts.is_empty() {
            return Err(Error::Sentinel("empty sentinel hosts list".into()));
        }

        let mut last_err = None;

        for host in &conf.hosts {
            match Connection::create_and_handshake(
                host,
                conf.username.as_deref(),
                conf.password.as_deref(),
                None,
            )
            .await
            {
                Ok(handle) => {
                    let cmd = Cmd::new(SENTINEL)
                        .arg(GET_MASTER_ADDR_BY_NAME)
                        .arg_bytes(&conf.service_name);

                    match handle.execute(cmd).await {
                        Ok(Value::Array(items)) if items.len() >= 2 => {
                            let ip = items[0]
                                .as_str()
                                .ok_or_else(|| Error::Sentinel("invalid master ip".into()))?;
                            let port = items[1]
                                .as_i64()
                                .map_err(|_| Error::Sentinel("invalid master port".into()))?;
                            return Ok(format!("{ip}:{port}"));
                        }
                        Ok(val) => {
                            last_err = Some(Error::Sentinel(format!(
                                "unexpected sentinel response: {val:?}"
                            )));
                        }
                        Err(e) => {
                            last_err = Some(e);
                        }
                    }
                }
                Err(e) => {
                    last_err = Some(e);
                }
            }
        }

        Err(last_err
            .unwrap_or_else(|| Error::Sentinel("failed to resolve master from sentinels".into())))
    }
}
