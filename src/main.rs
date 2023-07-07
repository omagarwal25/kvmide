use anyhow::Result;
use clap::{Parser, Subcommand};
use rdev::Event;
use serde::{Deserialize, Serialize};

mod client;
mod server;
mod utils;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    mode: Mode,
}

#[derive(Subcommand)]
enum Mode {
    Server,
    Client {
        #[arg(short, long)]
        addr: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.mode {
        Mode::Server => server::server().await,
        Mode::Client { addr } => client::listen(addr.clone()).await,
    }
}
