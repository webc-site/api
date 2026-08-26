use std::env;

use serde::Deserialize;

pub const DEFAULT_REDIS_PORT: u16 = 6379;
pub const DEFAULT_SENTINEL_PORT: u16 = 26379;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Server {
    pub host: String,
    pub port: u16,
}

impl Server {
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
        }
    }

    pub fn to_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

pub fn server_li(host_port: impl AsRef<str>, default_port: u16) -> Vec<Server> {
    host_port
        .as_ref()
        .split_whitespace()
        .map(|s| {
            if let Some((host, port)) = s.split_once(':') {
                Server {
                    host: host.into(),
                    port: port.parse().unwrap_or(default_port),
                }
            } else {
                Server {
                    host: s.into(),
                    port: default_port,
                }
            }
        })
        .collect()
}

#[derive(Debug, Clone)]
pub enum ServerConfig {
    Centralized {
        server: Server,
    },
    Sentinel {
        service_name: String,
        hosts: Vec<Server>,
        username: Option<String>,
        password: Option<String>,
    },
    Cluster {
        nodes: Vec<Server>,
    },
}

impl ServerConfig {
    pub fn centralized(server: Server) -> Self {
        Self::Centralized { server }
    }

    pub fn sentinel(
        service_name: impl Into<String>,
        hosts: Vec<Server>,
        username: Option<String>,
        password: Option<String>,
    ) -> Self {
        Self::Sentinel {
            service_name: service_name.into(),
            hosts,
            username,
            password,
        }
    }

    pub fn cluster(nodes: Vec<Server>) -> Self {
        Self::Cluster { nodes }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub server: Option<ServerConfig>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub database: Option<u8>,
}

#[derive(Deserialize, Debug)]
struct SentinelConf {
    pub name: String,
    #[serde(default)]
    pub host: Option<String>,
    #[serde(default, alias = "user")]
    pub username: Option<String>,
    #[serde(default, alias = "pass")]
    pub password: Option<String>,
}

#[derive(Deserialize, Debug)]
struct ClusterConf {
    pub nodes: Vec<String>,
}

pub fn conf_from_env(prefix: impl AsRef<str>) -> Config {
    let p = prefix.as_ref();
    let redis_env = format!("{p}_REDIS");
    let sentinel_env = format!("{p}_SENTINEL");
    let cluster_env = format!("{p}_CLUSTER");
    let user_env = format!("{p}_USER");
    let pass_env = format!("{p}_PASS");
    let db_env = format!("{p}_DB");

    let username = env::var(&user_env).ok();
    let password = env::var(&pass_env).ok();
    let database: Option<u8> = env::var(&db_env).ok().and_then(|v| v.parse().ok());

    let server = if let Ok(sentinel_str) = env::var(&sentinel_env) {
        let conf: SentinelConf =
            sonic_rs::from_str(&sentinel_str).expect("failed to parse SENTINEL json");
        let hosts = conf
            .host
            .map(|h| server_li(h, DEFAULT_SENTINEL_PORT))
            .unwrap_or_default();
        Some(ServerConfig::sentinel(
            conf.name,
            hosts,
            conf.username,
            conf.password,
        ))
    } else if let Ok(cluster_str) = env::var(&cluster_env) {
        let nodes = if cluster_str.trim().starts_with('{') || cluster_str.trim().starts_with('[') {
            if let Ok(c) = sonic_rs::from_str::<ClusterConf>(&cluster_str) {
                c.nodes
                    .into_iter()
                    .flat_map(|n| server_li(n, DEFAULT_REDIS_PORT))
                    .collect()
            } else if let Ok(nodes_li) = sonic_rs::from_str::<Vec<String>>(&cluster_str) {
                nodes_li
                    .into_iter()
                    .flat_map(|n| server_li(n, DEFAULT_REDIS_PORT))
                    .collect()
            } else {
                server_li(&cluster_str, DEFAULT_REDIS_PORT)
            }
        } else {
            server_li(&cluster_str, DEFAULT_REDIS_PORT)
        };
        Some(ServerConfig::cluster(nodes))
    } else if let Ok(redis_str) = env::var(&redis_env) {
        let server = server_li(redis_str, DEFAULT_REDIS_PORT)
            .into_iter()
            .next()
            .expect("empty redis hosts");
        Some(ServerConfig::centralized(server))
    } else {
        None
    };

    Config {
        server,
        username,
        password,
        database,
    }
}
