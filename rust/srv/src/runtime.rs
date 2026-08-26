use std::path::Path;
use std::sync::Arc;

use genv::s;
use tokio::sync::Semaphore;
use wasmtime::{
    Config, Engine, InstanceAllocationStrategy, Linker, Module, OptLevel, PoolingAllocationConfig,
    Store,
};
use wasmtime_wasi::WasiCtxBuilder;
use wasmtime_wasi::p1::{WasiP1Ctx, add_to_linker_async};

use crate::error::Result;

s!(
    POOL_TOTAL_INSTANCES: u32 | 1024;
    POOL_MAX_MEMORY_MB: usize | 64;
);

pub struct ServerCtx {
    pub wasi: WasiP1Ctx,
}

pub struct WasmEngine {
    pub engine: Engine,
    pub module: Module,
    pub linker: Linker<ServerCtx>,
    pub semaphore: Arc<Semaphore>,
}

impl WasmEngine {
    pub fn new(wasm_path: impl AsRef<Path>) -> Result<Arc<Self>> {
        let mut config = Config::new();
        config.cranelift_opt_level(OptLevel::Speed);
        config.parallel_compilation(true);
        config.memory_init_cow(true);
        config.async_stack_size(512 * 1024);

        let total_instances = *POOL_TOTAL_INSTANCES;
        let max_mem_mb = *POOL_MAX_MEMORY_MB;

        let mut pool_config = PoolingAllocationConfig::default();
        pool_config.total_core_instances(total_instances);
        pool_config.total_memories(total_instances);
        pool_config.total_tables(total_instances);
        pool_config.total_stacks(total_instances);
        pool_config.max_memories_per_module(1);
        pool_config.max_tables_per_module(1);
        pool_config.max_memory_size(max_mem_mb * 1024 * 1024);
        pool_config.decommit_batch_size(1024);

        config.allocation_strategy(InstanceAllocationStrategy::Pooling(pool_config));

        let engine = Engine::new(&config)?;
        let module = Module::from_file(&engine, wasm_path)?;

        let mut linker = Linker::new(&engine);
        add_to_linker_async(&mut linker, |s: &mut ServerCtx| &mut s.wasi)?;
        linker.define_unknown_imports_as_traps(&module)?;

        let semaphore = Arc::new(Semaphore::new(total_instances as usize));

        Ok(Arc::new(Self {
            engine,
            module,
            linker,
            semaphore,
        }))
    }

    pub fn new_store(&self) -> Store<ServerCtx> {
        let wasi = WasiCtxBuilder::new()
            .inherit_stdout()
            .inherit_stderr()
            .build_p1();

        Store::new(&self.engine, ServerCtx { wasi })
    }
}
