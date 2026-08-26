pub mod conf;

pub use conf::{Conf, DEFAULT_CONF, DEFAULT_DB, DEFAULT_URI, DbConf, env_conf};
use std::sync::LazyLock;
pub use sur::{Db, Sur, surreal};

pub static NS: LazyLock<Sur> = LazyLock::new(|| {
    let (conf, _) = env_conf();
    surreal(conf)
});

pub static DB: LazyLock<Db> = LazyLock::new(|| {
    let (_, name) = env_conf();
    let name = name.expect("miss env SURREAL_DB");
    NS.db(name)
});
