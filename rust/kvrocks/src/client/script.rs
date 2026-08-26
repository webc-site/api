use crate::{
    client::Client,
    error::Result,
    resp3::{
        Cmd, FromValue, Value,
        constants::{
            DELETE, EVAL, EVAL_RO, EVALSHA, EVALSHA_RO, EXISTS, FCALL, FCALL_RO, FLUSH, FUNCNAME,
            FUNCTION, KILL, LIBRARYNAME, LIST, LISTFUNC, LISTLIB, LOAD, REPLACE, SCRIPT, WITHCODE,
        },
    },
};

fn eval_generic<K: AsRef<[u8]>, A: AsRef<[u8]>>(
    cmd_name: &'static str,
    target: &str,
    keys: &[K],
    args: &[A],
) -> Cmd {
    Cmd::new(cmd_name)
        .arg_bytes(target)
        .arg_int(keys.len())
        .args_slice(keys)
        .args_slice(args)
}

impl Client {
    pub async fn eval<R: FromValue, K: AsRef<[u8]>, A: AsRef<[u8]>>(
        &self,
        script: &str,
        keys: &[K],
        args: &[A],
    ) -> Result<R> {
        let cmd = eval_generic(EVAL, script, keys, args);
        self.execute_cmd(cmd).await
    }

    pub async fn evalsha<R: FromValue, K: AsRef<[u8]>, A: AsRef<[u8]>>(
        &self,
        sha: &str,
        keys: &[K],
        args: &[A],
    ) -> Result<R> {
        let cmd = eval_generic(EVALSHA, sha, keys, args);
        self.execute_cmd(cmd).await
    }

    pub async fn eval_ro<R: FromValue, K: AsRef<[u8]>, A: AsRef<[u8]>>(
        &self,
        script: &str,
        keys: &[K],
        args: &[A],
    ) -> Result<R> {
        let cmd = eval_generic(EVAL_RO, script, keys, args);
        self.execute_cmd(cmd).await
    }

    pub async fn evalsha_ro<R: FromValue, K: AsRef<[u8]>, A: AsRef<[u8]>>(
        &self,
        sha: &str,
        keys: &[K],
        args: &[A],
    ) -> Result<R> {
        let cmd = eval_generic(EVALSHA_RO, sha, keys, args);
        self.execute_cmd(cmd).await
    }

    pub async fn script_load(&self, script: &str) -> Result<String> {
        let cmd = Cmd::new(SCRIPT).arg(LOAD).arg_bytes(script);
        self.execute_cmd(cmd).await
    }

    pub async fn script_exists<S: AsRef<[u8]>>(&self, shas: &[S]) -> Result<Vec<bool>> {
        self.execute_cmd(Cmd::new(SCRIPT).arg(EXISTS).args_slice(shas))
            .await
    }

    pub async fn script_flush(&self) -> Result<()> {
        self.execute_cmd(Cmd::new(SCRIPT).arg(FLUSH)).await
    }

    pub async fn script_kill(&self) -> Result<()> {
        self.execute_cmd(Cmd::new(SCRIPT).arg(KILL)).await
    }

    pub async fn fcall<R: FromValue, K: AsRef<[u8]>, A: AsRef<[u8]>>(
        &self,
        func_name: &str,
        keys: &[K],
        args: &[A],
    ) -> Result<R> {
        let cmd = eval_generic(FCALL, func_name, keys, args);
        self.execute_cmd(cmd).await
    }

    pub async fn fcall_ro<R: FromValue, K: AsRef<[u8]>, A: AsRef<[u8]>>(
        &self,
        func_name: &str,
        keys: &[K],
        args: &[A],
    ) -> Result<R> {
        let cmd = eval_generic(FCALL_RO, func_name, keys, args);
        self.execute_cmd(cmd).await
    }

    pub async fn function_load(&self, code: &str, replace: bool) -> Result<String> {
        let cmd = Cmd::new(FUNCTION)
            .arg(LOAD)
            .arg_if(replace, REPLACE)
            .arg_bytes(code);
        self.execute_cmd(cmd).await
    }

    pub async fn function_delete(&self, lib_name: &str) -> Result<()> {
        self.execute_cmd(Cmd::new(FUNCTION).arg(DELETE).arg_bytes(lib_name))
            .await
    }

    pub async fn function_list(&self, lib_name: Option<&str>, with_code: bool) -> Result<Value> {
        let cmd = Cmd::new(FUNCTION)
            .arg(LIST)
            .arg_keyword_opt_bytes(LIBRARYNAME, lib_name)
            .arg_if(with_code, WITHCODE);
        self.execute(cmd).await
    }

    pub async fn function_listfunc(&self, func_name: Option<&str>) -> Result<Value> {
        let cmd = Cmd::new(FUNCTION)
            .arg(LISTFUNC)
            .arg_keyword_opt_bytes(FUNCNAME, func_name);
        self.execute(cmd).await
    }

    pub async fn function_listlib(&self, lib_name: &str) -> Result<Value> {
        self.execute(Cmd::new(FUNCTION).arg(LISTLIB).arg_bytes(lib_name))
            .await
    }

    pub async fn function_flush(&self) -> Result<()> {
        self.execute_cmd(Cmd::new(FUNCTION).arg(FLUSH)).await
    }
}
