use crate::{
    client::Client,
    error::Result,
    resp3::{
        Cmd, Value,
        constants::{
            ASC, BYBOX, BYRADIUS, COUNT, DESC, FROMLONLAT, FROMMEMBER, GEOADD, GEODIST, GEOHASH,
            GEOPOS, GEORADIUS, GEORADIUS_RO, GEORADIUSBYMEMBER, GEORADIUSBYMEMBER_RO, GEOSEARCH,
            GEOSEARCHSTORE, STORE, STOREDIST, WITHCOORD, WITHDIST, WITHHASH,
        },
    },
};

#[derive(Debug, Clone, PartialEq)]
pub enum GeoSearch<'a> {
    FromMember(&'a str),
    FromLonLat(f64, f64),
    ByRadius(f64, &'a str),
    ByBox(f64, f64, &'a str),
    Asc,
    Desc,
    Count(usize),
    WithCoord,
    WithDist,
    WithHash,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GeoRadius<'a> {
    WithCoord,
    WithDist,
    WithHash,
    Count(usize),
    Asc,
    Desc,
    Store(&'a str),
    StoreDist(&'a str),
}

#[derive(Debug, Clone, PartialEq)]
pub enum GeoSearchStore<'a> {
    FromMember(&'a str),
    FromLonLat(f64, f64),
    ByRadius(f64, &'a str),
    ByBox(f64, f64, &'a str),
    Asc,
    Desc,
    Count(usize),
    StoreDist,
}

fn apply_geosearch_opt(cmd: Cmd, conf: &GeoSearch<'_>) -> Cmd {
    match conf {
        GeoSearch::FromMember(m) => cmd.arg(FROMMEMBER).arg_bytes(m),
        GeoSearch::FromLonLat(lon, lat) => cmd.arg(FROMLONLAT).arg_float(*lon).arg_float(*lat),
        GeoSearch::ByRadius(radius, unit) => cmd.arg(BYRADIUS).arg_float(*radius).arg_bytes(unit),
        GeoSearch::ByBox(w, h, unit) => cmd.arg(BYBOX).arg_float(*w).arg_float(*h).arg_bytes(unit),
        GeoSearch::Asc => cmd.arg(ASC),
        GeoSearch::Desc => cmd.arg(DESC),
        GeoSearch::Count(c) => cmd.arg(COUNT).arg_int(*c),
        GeoSearch::WithCoord => cmd.arg(WITHCOORD),
        GeoSearch::WithDist => cmd.arg(WITHDIST),
        GeoSearch::WithHash => cmd.arg(WITHHASH),
    }
}

fn apply_geosearchstore_opt(cmd: Cmd, conf: &GeoSearchStore<'_>) -> Cmd {
    match conf {
        GeoSearchStore::FromMember(m) => cmd.arg(FROMMEMBER).arg_bytes(m),
        GeoSearchStore::FromLonLat(lon, lat) => cmd.arg(FROMLONLAT).arg_float(*lon).arg_float(*lat),
        GeoSearchStore::ByRadius(radius, unit) => {
            cmd.arg(BYRADIUS).arg_float(*radius).arg_bytes(unit)
        }
        GeoSearchStore::ByBox(w, h, unit) => {
            cmd.arg(BYBOX).arg_float(*w).arg_float(*h).arg_bytes(unit)
        }
        GeoSearchStore::Asc => cmd.arg(ASC),
        GeoSearchStore::Desc => cmd.arg(DESC),
        GeoSearchStore::Count(c) => cmd.arg(COUNT).arg_int(*c),
        GeoSearchStore::StoreDist => cmd.arg(STOREDIST),
    }
}

fn apply_georadius_options<'a>(mut cmd: Cmd, conf_li: &[GeoRadius<'a>], readonly: bool) -> Cmd {
    for conf in conf_li {
        match conf {
            GeoRadius::WithCoord => {
                cmd = cmd.arg(WITHCOORD);
            }
            GeoRadius::WithDist => {
                cmd = cmd.arg(WITHDIST);
            }
            GeoRadius::WithHash => {
                cmd = cmd.arg(WITHHASH);
            }
            GeoRadius::Count(c) => {
                cmd = cmd.arg(COUNT).arg_int(*c);
            }
            GeoRadius::Asc => {
                cmd = cmd.arg(ASC);
            }
            GeoRadius::Desc => {
                cmd = cmd.arg(DESC);
            }
            GeoRadius::Store(k) => {
                if !readonly {
                    cmd = cmd.arg(STORE).arg_bytes(k);
                }
            }
            GeoRadius::StoreDist(k) => {
                if !readonly {
                    cmd = cmd.arg(STOREDIST).arg_bytes(k);
                }
            }
        }
    }
    cmd
}

impl Client {
    pub async fn geoadd<M: AsRef<[u8]>>(
        &self,
        key: impl AsRef<[u8]>,
        items: &[(f64, f64, M)],
    ) -> Result<u64> {
        let mut cmd = Cmd::new(GEOADD).arg_bytes(key);
        for (longitude, latitude, member) in items {
            cmd = cmd
                .arg_float(*longitude)
                .arg_float(*latitude)
                .arg_bytes(member);
        }
        self.execute_cmd(cmd).await
    }

    pub async fn geodist(
        &self,
        key: impl AsRef<[u8]>,
        member1: impl AsRef<[u8]>,
        member2: impl AsRef<[u8]>,
        unit: Option<&str>,
    ) -> Result<Option<f64>> {
        let mut cmd = Cmd::new(GEODIST)
            .arg_bytes(key)
            .arg_bytes(member1)
            .arg_bytes(member2);
        if let Some(u) = unit {
            cmd = cmd.arg_bytes(u);
        }
        self.execute_cmd(cmd).await
    }

    pub async fn geohash<M: AsRef<[u8]>>(
        &self,
        key: impl AsRef<[u8]>,
        members: &[M],
    ) -> Result<Vec<Option<String>>> {
        self.execute_cmd(Cmd::new(GEOHASH).arg_bytes(key).args_slice(members))
            .await
    }

    pub async fn geopos<M: AsRef<[u8]>>(
        &self,
        key: impl AsRef<[u8]>,
        members: &[M],
    ) -> Result<Vec<Option<(f64, f64)>>> {
        let cmd = Cmd::new(GEOPOS).arg_bytes(key).args_slice(members);
        self.execute_cmd(cmd).await
    }

    pub async fn geosearch(
        &self,
        key: impl AsRef<[u8]>,
        conf_li: impl AsRef<[GeoSearch<'_>]>,
    ) -> Result<Value> {
        let mut cmd = Cmd::new(GEOSEARCH).arg_bytes(key);
        for conf in conf_li.as_ref() {
            cmd = apply_geosearch_opt(cmd, conf);
        }
        self.execute(cmd).await
    }

    pub async fn georadius(
        &self,
        key: impl AsRef<[u8]>,
        longitude: f64,
        latitude: f64,
        radius: f64,
        unit: &str,
        conf_li: impl AsRef<[GeoRadius<'_>]>,
    ) -> Result<Value> {
        let cmd = Cmd::new(GEORADIUS)
            .arg_bytes(key)
            .arg_float(longitude)
            .arg_float(latitude)
            .arg_float(radius)
            .arg_bytes(unit);
        let cmd = apply_georadius_options(cmd, conf_li.as_ref(), false);
        self.execute(cmd).await
    }

    pub async fn georadiusbymember(
        &self,
        key: impl AsRef<[u8]>,
        member: impl AsRef<[u8]>,
        radius: f64,
        unit: &str,
        conf_li: impl AsRef<[GeoRadius<'_>]>,
    ) -> Result<Value> {
        let cmd = Cmd::new(GEORADIUSBYMEMBER)
            .arg_bytes(key)
            .arg_bytes(member)
            .arg_float(radius)
            .arg_bytes(unit);
        let cmd = apply_georadius_options(cmd, conf_li.as_ref(), false);
        self.execute(cmd).await
    }

    pub async fn georadius_ro(
        &self,
        key: impl AsRef<[u8]>,
        longitude: f64,
        latitude: f64,
        radius: f64,
        unit: &str,
        conf_li: impl AsRef<[GeoRadius<'_>]>,
    ) -> Result<Value> {
        let cmd = Cmd::new(GEORADIUS_RO)
            .arg_bytes(key)
            .arg_float(longitude)
            .arg_float(latitude)
            .arg_float(radius)
            .arg_bytes(unit);
        let cmd = apply_georadius_options(cmd, conf_li.as_ref(), true);
        self.execute(cmd).await
    }

    pub async fn georadiusbymember_ro(
        &self,
        key: impl AsRef<[u8]>,
        member: impl AsRef<[u8]>,
        radius: f64,
        unit: &str,
        conf_li: impl AsRef<[GeoRadius<'_>]>,
    ) -> Result<Value> {
        let cmd = Cmd::new(GEORADIUSBYMEMBER_RO)
            .arg_bytes(key)
            .arg_bytes(member)
            .arg_float(radius)
            .arg_bytes(unit);
        let cmd = apply_georadius_options(cmd, conf_li.as_ref(), true);
        self.execute(cmd).await
    }

    pub async fn geosearchstore(
        &self,
        destination: impl AsRef<[u8]>,
        source: impl AsRef<[u8]>,
        conf_li: impl AsRef<[GeoSearchStore<'_>]>,
    ) -> Result<u64> {
        let mut cmd = Cmd::new(GEOSEARCHSTORE)
            .arg_bytes(destination)
            .arg_bytes(source);
        for conf in conf_li.as_ref() {
            cmd = apply_geosearchstore_opt(cmd, conf);
        }
        self.execute_cmd(cmd).await
    }
}
