use crate::{
    client::Client,
    error::Result,
    resp3::{
        Cmd, Value,
        constants::{
            DD, FT_CREATE, FT_DROPINDEX, FT_EXPLAIN, FT_EXPLAINSQL, FT_INFO, FT_LIST, FT_SEARCH,
            FT_SEARCHSQL, FT_TAGVALS, LIMIT, NOCONTENT,
        },
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FtSearch {
    Limit(usize, usize),
    NoContent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FtDropIndex {
    Dd,
}

fn apply_ft_search(cmd: Cmd, conf: &FtSearch) -> Cmd {
    match conf {
        FtSearch::NoContent => cmd.arg(NOCONTENT),
        FtSearch::Limit(offset, num) => cmd.arg(LIMIT).arg_int(*offset).arg_int(*num),
    }
}

impl Client {
    pub async fn ft_create<A: AsRef<[u8]>>(
        &self,
        index: impl AsRef<[u8]>,
        args: &[A],
    ) -> Result<()> {
        self.execute_cmd(Cmd::new(FT_CREATE).arg_bytes(index).args_slice(args))
            .await
    }

    pub async fn ft_search(
        &self,
        index: &str,
        query: &str,
        conf_li: impl AsRef<[FtSearch]>,
    ) -> Result<Value> {
        let mut cmd = Cmd::new(FT_SEARCH).arg_bytes(index).arg_bytes(query);
        for conf in conf_li.as_ref() {
            cmd = apply_ft_search(cmd, conf);
        }
        self.execute(cmd).await
    }

    pub async fn ft_searchsql(&self, sql: &str) -> Result<Value> {
        self.execute(Cmd::new(FT_SEARCHSQL).arg_bytes(sql)).await
    }

    pub async fn ft_explain(&self, index: &str, query: &str) -> Result<String> {
        self.execute_cmd(Cmd::new(FT_EXPLAIN).arg_bytes(index).arg_bytes(query))
            .await
    }

    pub async fn ft_explainsql(&self, sql: &str) -> Result<String> {
        self.execute_cmd(Cmd::new(FT_EXPLAINSQL).arg_bytes(sql))
            .await
    }

    pub async fn ft_info(&self, index: &str) -> Result<Value> {
        self.execute(Cmd::new(FT_INFO).arg_bytes(index)).await
    }

    pub async fn ft_list(&self) -> Result<Vec<String>> {
        self.execute_cmd(Cmd::new(FT_LIST)).await
    }

    pub async fn ft_dropindex(
        &self,
        index: &str,
        conf_li: impl AsRef<[FtDropIndex]>,
    ) -> Result<()> {
        let mut cmd = Cmd::new(FT_DROPINDEX).arg_bytes(index);
        for conf in conf_li.as_ref() {
            if matches!(conf, FtDropIndex::Dd) {
                cmd = cmd.arg(DD);
            }
        }
        self.execute_cmd(cmd).await
    }

    pub async fn ft_tagvals(&self, index: &str, field_name: &str) -> Result<Vec<String>> {
        self.execute_cmd(Cmd::new(FT_TAGVALS).arg_bytes(index).arg_bytes(field_name))
            .await
    }
}
