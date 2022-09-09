use anyhow::{anyhow, Context};
use clap::Parser;
use iroh_p2p::config::{Config, CONFIG_FILE_NAME, ENV_PREFIX};
use iroh_p2p::{cli::Args, metrics, DiskStorage, Keychain, Node};
use iroh_util::{iroh_home_path, make_config};
use tokio::task;
use tracing::{debug, error};
// use tokio::runtime::{self, UnhandledPanic};
// use tokio::runtime::UnhandledPanic;


async fn serve() {
    {//serve(i, config.clone(), rpc_addr));
    let version = option_env!("IROH_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"));
    println!("Starting iroh-p2p, version {version}");

    let args = Args::parse();

    // TODO: configurable network
    let sources = vec![iroh_home_path(CONFIG_FILE_NAME), args.cfg.clone()];
    let network_config = make_config(
        // default
        Config::default_grpc(),
        // potential config files
        sources,
        // env var prefix for this config
        ENV_PREFIX,
        // map of present command line arguments
        args.make_overrides_map(),
    )
    .context("invalid config").unwrap();

    let metrics_config =
        metrics::metrics_config_with_compile_time_info(network_config.metrics.clone());

    let metrics_handle = iroh_metrics::MetricsHandle::new(metrics_config)
        .await
        .map_err(|e| anyhow!("metrics init failed: {:?}", e)).unwrap();

    #[cfg(unix)]
    {
        match iroh_util::increase_fd_limit() {
            Ok(soft) => debug!("NOFILE limit: soft = {}", soft),
            Err(err) => error!("Error increasing NOFILE limit: {}", err),
        }
    }

    let kc = Keychain::<DiskStorage>::new().await.unwrap();
    let rpc_addr = network_config
        .server_rpc_addr().unwrap()
        .ok_or_else(|| anyhow!("missing p2p rpc addr")).unwrap();
    let mut p2p = Node::new(network_config, rpc_addr, kc).await.unwrap();

    // Start services
    let p2p_task = task::unconstrained(async move {
        if let Err(err) = p2p.run().await {
            error!("{:?}", err);
        }
    }).await;

    iroh_util::block_until_sigint().await;
    

    // Cancel all async services
    // p2p_task.abort();

    metrics_handle.shutdown();
    }
}
/// Starts daemon process
// #[tokio::main(flavor = "multi_thread")]
fn main() -> anyhow::Result<()> {

    tokio::runtime::Builder::new_multi_thread()
                // .disable_lifo_slot()
                .max_blocking_threads(2048)
                .thread_stack_size(16 * 1024 * 1024)
                // .global_queue_interval(1024*1024)
                // .event_interval(2048)
                // .unhandled_panic(UnhandledPanic::ShutdownRuntime)
                .enable_all()
                .build()
                .unwrap()
                .block_on(serve());

    Ok(())
}
