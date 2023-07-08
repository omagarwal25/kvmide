use anyhow::Result;
use futures::prelude::*;
use tokio::io;
use tokio::net::TcpStream;
use tokio_serde::formats::SymmetricalJson;
use tokio_serde::SymmetricallyFramed;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

use crate::utils::Packet;
pub async fn listen(host: String) -> Result<()> {
    let socket = TcpStream::connect(host).await?;
    let (rd, wr) = io::split(socket);

    // let length_delimited_write = FramedWrite::new(wr, LengthDelimitedCodec::new());
    let length_delimited_read = FramedRead::new(rd, LengthDelimitedCodec::new());

    // TODO: eventually set up code to report screen size and other metadata

    let mut deserialized =
        SymmetricallyFramed::new(length_delimited_read, SymmetricalJson::<Packet>::default());

    while let Some(value) = deserialized.try_next().await? {
        println!("GOT {:?}", value);
        if let Packet::Command(event) = value {
            rdev::simulate(&event)?;
        }
    }

    Ok(())
}
