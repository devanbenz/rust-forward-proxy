use clap::Parser;
use rust_forward_proxy::cli::Args;
use rust_forward_proxy::proxy::run_proxy;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let target_addr = format!("{}:{}", args.target_addr, args.target_port);
    let listener_addr = format!("{}:{}", args.listener_addr, args.listener_port);

    run_proxy(target_addr, listener_addr).await;
}
