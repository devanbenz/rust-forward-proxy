use clap::Parser;

#[derive(Parser)]
pub struct Args {
    #[arg(long)]
    pub target_addr: String,
    #[arg(long)]
    pub target_port: String,
    #[arg(long)]
    pub listener_addr: String,
    #[arg(long)]
    pub listener_port: String,
}

