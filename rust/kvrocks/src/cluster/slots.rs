use std::sync::Arc;

use arc_swap::ArcSwap;

use crate::{
    cluster::crc16::CLUSTER_SLOTS,
    error::{Error, Result},
    resp3::types::Value,
};

#[derive(Debug)]
pub struct SlotMap {
    slots: ArcSwap<Box<[Option<Arc<str>>]>>,
}

impl Default for SlotMap {
    fn default() -> Self {
        Self::new()
    }
}

#[inline]
fn format_addr(ip: &str, port: i64) -> Arc<str> {
    format!("{ip}:{port}").into()
}

#[inline]
fn get_field<'a>(val: &'a Value, field: &str) -> Option<&'a Value> {
    match val {
        Value::Map(pairs) => pairs
            .iter()
            .find_map(|(k, v)| (k.as_str() == Some(field)).then_some(v)),
        Value::Array(arr) if arr.len() % 2 == 0 => arr
            .as_chunks::<2>()
            .0
            .iter()
            .find_map(|pair| (pair[0].as_str() == Some(field)).then_some(&pair[1])),
        _ => None,
    }
}

impl SlotMap {
    pub fn new() -> Self {
        Self {
            slots: ArcSwap::from_pointee(vec![None; CLUSTER_SLOTS].into_boxed_slice()),
        }
    }

    #[inline]
    pub fn get_node(&self, slot: u16) -> Option<Arc<str>> {
        self.slots.load().get(slot as usize).cloned().flatten()
    }

    pub fn update_slot(&self, slot: u16, addr: impl Into<Arc<str>>) {
        self.update_ranges(&[(slot, slot, addr.into())]);
    }

    pub fn update_range(&self, start: u16, end: u16, addr: impl Into<Arc<str>>) {
        self.update_ranges(&[(start, end, addr.into())]);
    }

    pub fn update_ranges(&self, ranges: &[(u16, u16, Arc<str>)]) {
        if ranges.is_empty() {
            return;
        }
        self.slots.rcu(|old| {
            let mut s = (**old).clone();
            for &(start, end, ref addr) in ranges {
                if start > end {
                    continue;
                }
                let start_idx = (start as usize).min(CLUSTER_SLOTS);
                let end_idx = (end as usize).min(CLUSTER_SLOTS.saturating_sub(1));
                if start_idx <= end_idx {
                    s[start_idx..=end_idx].fill(Some(addr.clone()));
                }
            }
            s
        });
    }

    pub fn parse_cluster_slots(&self, val: &Value) -> Result<()> {
        // 兼容 CLUSTER SLOTS 格式: Array of [start_slot, end_slot, [ip, port, id], ...]
        if let Value::Array(items) = val {
            let mut ranges = Vec::with_capacity(items.len());
            for item in items {
                if let Value::Array(parts) = item
                    && parts.len() >= 3
                {
                    let start = parts[0].as_i64()? as u16;
                    let end = parts[1].as_i64()? as u16;
                    if let Value::Array(master_node) = &parts[2]
                        && master_node.len() >= 2
                    {
                        let ip = master_node[0].as_str().unwrap_or("127.0.0.1");
                        let port = master_node[1].as_i64()?;
                        let addr = format_addr(ip, port);
                        ranges.push((start, end, addr));
                    }
                }
            }
            self.update_ranges(&ranges);
            return Ok(());
        }
        Err(Error::Protocol("invalid CLUSTER SLOTS response".into()))
    }

    pub fn parse_cluster_shards(&self, val: &Value) -> Result<()> {
        // 兼容 RESP3 CLUSTER SHARDS 格式 (零拷贝遍历)
        if let Value::Array(shards) = val {
            let mut ranges = Vec::with_capacity(shards.len());
            for shard in shards {
                let mut master_addr: Option<Arc<str>> = None;
                if let Some(Value::Array(nodes)) = get_field(shard, "nodes") {
                    for node in nodes {
                        let role = get_field(node, "role")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default();
                        if role == "master" {
                            let ip = get_field(node, "ip")
                                .or_else(|| get_field(node, "endpoint"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("127.0.0.1");
                            let port = get_field(node, "port")
                                .and_then(|v| v.as_i64().ok())
                                .unwrap_or(6379);
                            master_addr = Some(format_addr(ip, port));
                            break;
                        }
                    }
                }

                if let (Some(addr), Some(Value::Array(slots))) =
                    (master_addr, get_field(shard, "slots"))
                {
                    for pair in slots.as_chunks::<2>().0 {
                        if let (Ok(start), Ok(end)) = (pair[0].as_i64(), pair[1].as_i64()) {
                            ranges.push((start as u16, end as u16, addr.clone()));
                        }
                    }
                }
            }
            self.update_ranges(&ranges);
            return Ok(());
        }
        Err(Error::Protocol("invalid CLUSTER SHARDS response".into()))
    }
}
