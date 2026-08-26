use std::env::var;

use serde::Deserialize;
pub use sur::Conf;

pub type DbConf = Conf;

pub const DEFAULT_URI: Option<&str> = option_env!("SURREAL_URI");
pub const DEFAULT_CONF: Option<&str> = option_env!("SURREAL_CONF");
pub const DEFAULT_DB: Option<&str> = option_env!("SURREAL_DB");

#[derive(Deserialize)]
struct AuthConf {
    #[serde(alias = "user")]
    pub username: String,
    #[serde(alias = "pass")]
    pub password: String,
    #[serde(default, alias = "ns")]
    pub namespace: Option<String>,
}

pub fn env_conf() -> (Conf, Option<String>) {
    let uri = var("SURREAL_URI")
        .ok()
        .or_else(|| DEFAULT_URI.map(ToString::to_string))
        .expect("miss env SURREAL_URI");

    let conf_str = var("SURREAL_CONF")
        .ok()
        .or_else(|| DEFAULT_CONF.map(ToString::to_string))
        .expect("miss env SURREAL_CONF");

    let db = var("SURREAL_DB")
        .ok()
        .or_else(|| DEFAULT_DB.map(ToString::to_string));

    let auth: AuthConf = sonic_rs::from_str(&conf_str).expect("failed to parse SURREAL_CONF json");

    (
        Conf {
            uri,
            username: auth.username,
            password: auth.password,
            namespace: auth.namespace,
        },
        db,
    )
}
