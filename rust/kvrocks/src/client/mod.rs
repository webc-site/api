pub mod bit;
pub mod bloom;
pub mod cluster;
pub mod conf;
pub mod geo;
pub mod hash;
pub(crate) mod helper;
pub mod hll;
pub mod json;
pub mod key;
pub mod list;
pub mod pubsub;
pub mod replication;
pub mod script;
pub mod search;
pub mod server;
pub mod set;
pub mod sortedint;
pub mod stream;
pub mod string;
pub mod tdigest;
pub mod timeseries;
pub mod txn;
pub mod zset;

use std::{borrow::Cow, sync::Arc};

use arc_swap::{ArcSwap, ArcSwapOption};
use rapidhash::RapidHashMap as HashMap;

pub use bit::{BitIndexUnit, Bitfield};
pub use bloom::{BfInsert, BfReserve, CfReserve};
pub use conf::{
    Config, DEFAULT_REDIS_PORT, DEFAULT_SENTINEL_PORT, Server, ServerConfig, conf_from_env,
    server_li,
};
pub use geo::{GeoRadius, GeoSearch, GeoSearchStore};
pub use hash::{HExpireCondition, HGetExOption, HRangeByLex, HScan, HSetExOption};
pub use json::JsonSet;
pub use key::{Scan, Sort};
pub use list::{InsertDirection, LPos, ListDirection};
pub use pubsub::PubSubStream;
pub use search::{FtDropIndex, FtSearch};
pub use server::PollUpdates;
pub use set::SScan;
pub use sortedint::{SiRange, SiRangeByValue};
pub use stream::{XAddOption, XAutoClaim, XClaim, XReadGroup};
pub use string::{DelEx, GetEx, LcsOption, Set};
pub use tdigest::TDigestMerge;
pub use timeseries::{TsAdd, TsAlter, TsCreate, TsIncrBy, TsMGet, TsMRange, TsRange};
pub use zset::{Aggregate, PopDirection, ZAddOption, ZRangeByScore, ZRangeStore, ZScan};

use crate::{
    cluster::{SlotMap, slot},
    connection::{Connection, SenderHandle},
    error::{Error, Result},
    resp3::{
        Cmd, FromValue, Value,
        constants::{ASKING, CLUSTER, SHARDS, SLOTS},
    },
    sentinel::{SentinelConfig, SentinelManager},
};

#[derive(Debug)]
pub(crate) enum Topology {
    Standalone {
        addr: String,
        handle: ArcSwapOption<SenderHandle>,
    },
    Sentinel {
        conf: SentinelConfig,
        current_master: ArcSwapOption<(String, SenderHandle)>,
    },
    Cluster {
        nodes: Vec<String>,
        slots: SlotMap,
        connections: ArcSwap<HashMap<String, SenderHandle>>,
    },
}

#[derive(Debug)]
pub(crate) struct Inner {
    pub(crate) topology: Topology,
    pub(crate) username: Option<String>,
    pub(crate) password: Option<String>,
    pub(crate) database: Option<u8>,
}

#[derive(Clone, Debug)]
pub struct Client {
    inner: Arc<Inner>,
}

impl Client {
    pub fn new(conf: Config) -> Self {
        let server_conf = conf.server.unwrap_or_else(|| ServerConfig::Centralized {
            server: Server::new("127.0.0.1", DEFAULT_REDIS_PORT),
        });

        let topology = match server_conf {
            ServerConfig::Centralized { server } => Topology::Standalone {
                addr: server.to_addr(),
                handle: ArcSwapOption::empty(),
            },
            ServerConfig::Sentinel {
                service_name,
                hosts,
                username: sent_user,
                password: sent_pass,
            } => {
                let sent_conf = SentinelConfig::new(
                    service_name,
                    hosts.into_iter().map(|s| s.to_addr()).collect(),
                )
                .auth(sent_user, sent_pass);

                Topology::Sentinel {
                    conf: sent_conf,
                    current_master: ArcSwapOption::empty(),
                }
            }
            ServerConfig::Cluster { nodes } => {
                let seed_addrs: Vec<String> = nodes.into_iter().map(|s| s.to_addr()).collect();
                Topology::Cluster {
                    nodes: seed_addrs,
                    slots: SlotMap::new(),
                    connections: ArcSwap::from_pointee(HashMap::default()),
                }
            }
        };

        Self {
            inner: Arc::new(Inner {
                topology,
                username: conf.username,
                password: conf.password,
                database: conf.database,
            }),
        }
    }

    pub fn from_env(prefix: impl AsRef<str>) -> Self {
        Self::new(conf_from_env(prefix))
    }

    pub async fn exec_into<T: FromValue>(&self, cmd: Cmd) -> Result<T> {
        let val = self.execute(cmd).await?;
        FromValue::from_value(val)
    }

    pub async fn execute_cmd<T: FromValue>(&self, cmd: Cmd) -> Result<T> {
        self.exec_into(cmd).await
    }

    pub async fn exec_val(&self, cmd: Cmd) -> Result<Value> {
        self.execute(cmd).await
    }

    pub async fn exec_single_or_array<T: FromValue>(&self, cmd: Cmd) -> Result<Vec<T>> {
        let val = self.execute(cmd).await?;
        helper::parse_single_or_array(val)
    }

    pub async fn exec_pair_array<T1: FromValue, T2: FromValue>(
        &self,
        cmd: Cmd,
    ) -> Result<Vec<(T1, T2)>> {
        let val = self.execute(cmd).await?;
        helper::parse_pair_array(val)
    }

    pub async fn exec_scan(&self, cmd: Cmd) -> Result<(u64, Vec<String>)> {
        let val = self.execute(cmd).await?;
        helper::parse_scan_result(val)
    }

    pub async fn exec_scan_pair<T1: FromValue, T2: FromValue>(
        &self,
        cmd: Cmd,
    ) -> Result<(u64, Vec<(T1, T2)>)> {
        let val = self.execute(cmd).await?;
        helper::parse_scan_pair_result(val)
    }

    pub async fn connect(conf: Config) -> Result<Self> {
        let client = Self::new(conf);
        client.init().await?;
        Ok(client)
    }

    pub async fn init(&self) -> Result<()> {
        match &self.inner.topology {
            Topology::Standalone { addr, handle } => {
                let _ = self.get_standalone_conn(addr, handle).await?;
            }
            Topology::Sentinel {
                conf,
                current_master,
            } => {
                let _ = self.get_sentinel_conn(conf, current_master).await?;
            }
            Topology::Cluster {
                nodes,
                slots,
                connections,
            } => {
                self.ensure_cluster_slots(nodes, slots, connections).await?;
            }
        }
        Ok(())
    }

    pub async fn execute(&self, cmd: Cmd) -> Result<Value> {
        match &self.inner.topology {
            Topology::Standalone { addr, handle } => {
                let current_handle = self.get_standalone_conn(addr, handle).await?;
                match current_handle.execute(cmd.clone()).await {
                    Ok(val) => Ok(val),
                    Err(Error::ConnectionClosed) => {
                        let new_handle = self
                            .reconnect_standalone(addr, handle, Some(&current_handle))
                            .await?;
                        new_handle.execute(cmd).await
                    }
                    Err(e) => Err(e),
                }
            }
            Topology::Sentinel {
                conf,
                current_master,
            } => {
                let current_handle = self.get_sentinel_conn(conf, current_master).await?;
                match current_handle.execute(cmd.clone()).await {
                    Ok(val) => Ok(val),
                    Err(Error::ConnectionClosed) => {
                        let new_handle = self
                            .refresh_sentinel_master(conf, current_master, Some(&current_handle))
                            .await?;
                        new_handle.execute(cmd).await
                    }
                    Err(Error::Redis(ref s))
                        if s.starts_with("READONLY")
                            || s.starts_with("LOADING")
                            || s.starts_with("MASTERDOWN") =>
                    {
                        let new_handle = self
                            .refresh_sentinel_master(conf, current_master, Some(&current_handle))
                            .await?;
                        new_handle.execute(cmd).await
                    }
                    Err(e) => Err(e),
                }
            }
            Topology::Cluster {
                nodes,
                slots,
                connections,
            } => {
                self.ensure_cluster_slots(nodes, slots, connections).await?;

                let key_opt = cmd.first_key();
                let slot_num = key_opt.map(slot).unwrap_or(0);
                let node_addr = slots.get_node(slot_num);

                let handle = if let Some(addr) = node_addr {
                    self.get_or_create_cluster_conn(&addr).await?
                } else {
                    let conns = connections.load();
                    conns
                        .values()
                        .next()
                        .cloned()
                        .ok_or(Error::ClusterSlotUncovered(slot_num))?
                };

                match handle.execute(cmd.clone()).await {
                    Ok(val) => Ok(val),
                    Err(Error::ConnectionClosed) => {
                        if let Some(addr) = slots.get_node(slot_num) {
                            let new_handle =
                                self.reconnect_cluster_conn(&addr, Some(&handle)).await?;
                            new_handle.execute(cmd).await
                        } else {
                            Err(Error::ConnectionClosed)
                        }
                    }
                    Err(Error::Moved {
                        slot: moved_slot,
                        addr,
                    }) => {
                        slots.update_slot(moved_slot, addr.clone());
                        let target_handle = self.get_or_create_cluster_conn(&addr).await?;
                        target_handle.execute(cmd).await
                    }
                    Err(Error::Ask { addr, .. }) => {
                        let target_handle = self.get_or_create_cluster_conn(&addr).await?;
                        let _ = target_handle.execute(Cmd::new(ASKING)).await;
                        target_handle.execute(cmd).await
                    }
                    Err(e) => Err(e),
                }
            }
        }
    }

    async fn get_standalone_conn(
        &self,
        addr: &str,
        handle: &ArcSwapOption<SenderHandle>,
    ) -> Result<SenderHandle> {
        if let Some(h) = handle.load_full() {
            return Ok((*h).clone());
        }
        self.reconnect_standalone(addr, handle, None).await
    }

    async fn reconnect_standalone(
        &self,
        addr: &str,
        handle: &ArcSwapOption<SenderHandle>,
        failed_handle: Option<&SenderHandle>,
    ) -> Result<SenderHandle> {
        if let Some(current) = handle.load_full()
            && failed_handle.is_none_or(|f| !current.is_same(f))
        {
            return Ok((*current).clone());
        }
        let new_handle = Connection::create_and_handshake(
            addr,
            self.inner.username.as_deref(),
            self.inner.password.as_deref(),
            self.inner.database,
        )
        .await?;
        handle.store(Some(Arc::new(new_handle.clone())));
        Ok(new_handle)
    }

    pub(crate) async fn get_sentinel_conn(
        &self,
        conf: &SentinelConfig,
        current_master: &ArcSwapOption<(String, SenderHandle)>,
    ) -> Result<SenderHandle> {
        if let Some(m) = current_master.load_full() {
            return Ok(m.1.clone());
        }
        self.refresh_sentinel_master(conf, current_master, None)
            .await
    }

    async fn refresh_sentinel_master(
        &self,
        conf: &SentinelConfig,
        current_master: &ArcSwapOption<(String, SenderHandle)>,
        failed_handle: Option<&SenderHandle>,
    ) -> Result<SenderHandle> {
        if let Some(current) = current_master.load_full()
            && failed_handle.is_none_or(|f| !current.1.is_same(f))
        {
            return Ok(current.1.clone());
        }
        let new_master = SentinelManager::resolve_master(conf).await?;
        let new_handle = Connection::create_and_handshake(
            &new_master,
            self.inner.username.as_deref(),
            self.inner.password.as_deref(),
            self.inner.database,
        )
        .await?;
        current_master.store(Some(Arc::new((new_master, new_handle.clone()))));
        Ok(new_handle)
    }

    async fn ensure_cluster_slots(
        &self,
        nodes: &[String],
        slots: &SlotMap,
        connections: &ArcSwap<HashMap<String, SenderHandle>>,
    ) -> Result<()> {
        if !connections.load().is_empty() {
            return Ok(());
        }
        for addr in nodes {
            if let Ok(handle) = Connection::create_and_handshake(
                addr,
                self.inner.username.as_deref(),
                self.inner.password.as_deref(),
                None,
            )
            .await
            {
                let mut parsed = false;
                if let Ok(val) = handle.execute(Cmd::new(CLUSTER).arg(SHARDS)).await {
                    parsed = slots.parse_cluster_shards(&val).is_ok();
                }
                if !parsed && let Ok(val) = handle.execute(Cmd::new(CLUSTER).arg(SLOTS)).await {
                    parsed = slots.parse_cluster_slots(&val).is_ok();
                }

                if parsed {
                    connections.rcu(|old| {
                        let mut m = (**old).clone();
                        m.insert(addr.clone(), handle.clone());
                        m
                    });
                    return Ok(());
                }
            }
        }
        Err(Error::Config(
            "failed to initialize cluster slots topology".into(),
        ))
    }

    async fn get_or_create_cluster_conn(&self, addr: &str) -> Result<SenderHandle> {
        if let Topology::Cluster { connections, .. } = &self.inner.topology {
            if let Some(h) = connections.load().get(addr).cloned() {
                return Ok(h);
            }
            self.reconnect_cluster_conn(addr, None).await
        } else {
            Err(Error::Config("not in cluster mode".into()))
        }
    }

    async fn reconnect_cluster_conn(
        &self,
        addr: &str,
        failed_handle: Option<&SenderHandle>,
    ) -> Result<SenderHandle> {
        if let Topology::Cluster { connections, .. } = &self.inner.topology {
            let conns = connections.load();
            if let Some(current) = conns.get(addr)
                && failed_handle.is_none_or(|f| !current.is_same(f))
            {
                return Ok(current.clone());
            }
            let new_handle = Connection::create_and_handshake(
                addr,
                self.inner.username.as_deref(),
                self.inner.password.as_deref(),
                None,
            )
            .await?;
            connections.rcu(|old| {
                let mut m = (**old).clone();
                m.insert(addr.to_string(), new_handle.clone());
                m
            });
            Ok(new_handle)
        } else {
            Err(Error::Config("not in cluster mode".into()))
        }
    }

    pub async fn cmd(&self, name: impl Into<Cow<'static, str>>, args: &[&[u8]]) -> Result<Value> {
        let cmd = Cmd::new(name).args(args.iter().copied());
        self.execute(cmd).await
    }
}
