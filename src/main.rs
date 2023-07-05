use clap::{Parser, Subcommand};
use tokio::io;
mod client;
mod server;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    mode: Mode,
}

#[derive(Subcommand)]
enum Mode {
    Server,
    Client,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let cli = Cli::parse();

    match &cli.mode {
        Mode::Server => server::server().await,
        Mode::Client => client::listen().await,
    }
}
