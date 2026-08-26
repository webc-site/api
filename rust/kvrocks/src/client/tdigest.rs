use crate::{
    client::Client,
    error::Result,
    resp3::{
        Cmd, Value,
        constants::{
            COMPRESSION, OVERRIDE, TDIGEST_ADD, TDIGEST_BYRANK, TDIGEST_BYREVRANK, TDIGEST_CDF,
            TDIGEST_CREATE, TDIGEST_INFO, TDIGEST_MAX, TDIGEST_MERGE, TDIGEST_MIN,
            TDIGEST_QUANTILE, TDIGEST_RANK, TDIGEST_RESET, TDIGEST_REVRANK, TDIGEST_TRIMMED_MEAN,
        },
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TDigestMerge {
    Override,
}

impl Client {
    pub async fn tdigest_create(
        &self,
        key: impl AsRef<[u8]>,
        compression: Option<u32>,
    ) -> Result<()> {
        let cmd = Cmd::new(TDIGEST_CREATE)
            .arg_bytes(key)
            .arg_keyword_opt_int(COMPRESSION, compression);
        self.execute_cmd(cmd).await
    }

    pub async fn tdigest_add(&self, key: impl AsRef<[u8]>, values: &[f64]) -> Result<()> {
        self.execute_cmd(Cmd::new(TDIGEST_ADD).arg_bytes(key).args_floats(values))
            .await
    }

    pub async fn tdigest_max(&self, key: impl AsRef<[u8]>) -> Result<Option<f64>> {
        self.execute_cmd(Cmd::new(TDIGEST_MAX).arg_bytes(key)).await
    }

    pub async fn tdigest_min(&self, key: impl AsRef<[u8]>) -> Result<Option<f64>> {
        self.execute_cmd(Cmd::new(TDIGEST_MIN).arg_bytes(key)).await
    }

    pub async fn tdigest_rank(&self, key: impl AsRef<[u8]>, values: &[f64]) -> Result<Vec<i64>> {
        self.execute_cmd(Cmd::new(TDIGEST_RANK).arg_bytes(key).args_floats(values))
            .await
    }

    pub async fn tdigest_revrank(&self, key: impl AsRef<[u8]>, values: &[f64]) -> Result<Vec<i64>> {
        self.execute_cmd(Cmd::new(TDIGEST_REVRANK).arg_bytes(key).args_floats(values))
            .await
    }

    pub async fn tdigest_byrank(&self, key: impl AsRef<[u8]>, ranks: &[u64]) -> Result<Vec<f64>> {
        self.execute_cmd(Cmd::new(TDIGEST_BYRANK).arg_bytes(key).args_ints(ranks))
            .await
    }

    pub async fn tdigest_byrevrank(
        &self,
        key: impl AsRef<[u8]>,
        ranks: &[u64],
    ) -> Result<Vec<f64>> {
        self.execute_cmd(Cmd::new(TDIGEST_BYREVRANK).arg_bytes(key).args_ints(ranks))
            .await
    }

    pub async fn tdigest_quantile(
        &self,
        key: impl AsRef<[u8]>,
        quantiles: &[f64],
    ) -> Result<Vec<f64>> {
        self.execute_cmd(
            Cmd::new(TDIGEST_QUANTILE)
                .arg_bytes(key)
                .args_floats(quantiles),
        )
        .await
    }

    pub async fn tdigest_cdf(&self, key: impl AsRef<[u8]>, values: &[f64]) -> Result<Vec<f64>> {
        self.execute_cmd(Cmd::new(TDIGEST_CDF).arg_bytes(key).args_floats(values))
            .await
    }

    pub async fn tdigest_trimmed_mean(
        &self,
        key: impl AsRef<[u8]>,
        low_cut_quantile: f64,
        high_cut_quantile: f64,
    ) -> Result<f64> {
        self.execute_cmd(
            Cmd::new(TDIGEST_TRIMMED_MEAN)
                .arg_bytes(key)
                .arg_float(low_cut_quantile)
                .arg_float(high_cut_quantile),
        )
        .await
    }

    pub async fn tdigest_merge<K: AsRef<[u8]>>(
        &self,
        destination: impl AsRef<[u8]>,
        source_keys: &[K],
        compression: Option<u32>,
        conf_li: impl AsRef<[TDigestMerge]>,
    ) -> Result<()> {
        let mut cmd = Cmd::new(TDIGEST_MERGE)
            .arg_bytes(destination)
            .arg_int(source_keys.len())
            .args_slice(source_keys)
            .arg_keyword_opt_int(COMPRESSION, compression);
        for conf in conf_li.as_ref() {
            if matches!(conf, TDigestMerge::Override) {
                cmd = cmd.arg(OVERRIDE);
            }
        }
        self.execute_cmd(cmd).await
    }

    pub async fn tdigest_info(&self, key: impl AsRef<[u8]>) -> Result<Value> {
        self.execute(Cmd::new(TDIGEST_INFO).arg_bytes(key)).await
    }

    pub async fn tdigest_reset(&self, key: impl AsRef<[u8]>) -> Result<()> {
        self.execute_cmd(Cmd::new(TDIGEST_RESET).arg_bytes(key))
            .await
    }
}
