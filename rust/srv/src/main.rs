use mimalloc::MiMalloc;
use webc_srv::{Result, srv};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() -> Result<()> {
    log_init::init();
    srv().await
}
