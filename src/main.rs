use clap::{Parser, Subcommand};
use rdev::Event;
use serde::{Deserialize, Serialize};
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
    Client {
        #[arg(short, long)]
        addr: String,
    },
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let cli = Cli::parse();

    match &cli.mode {
        Mode::Server => server::server().await,
        Mode::Client { addr } => client::listen(addr.clone()).await,
    }
}

#[derive(Serialize, Deserialize, Debug)]
enum Packet {
    Message(String),
    Command(Event),
}

#[derive(Serialize, Deserialize)]
enum Message {
    Ready,
}
