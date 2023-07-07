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

    let length_delimited_write = FramedWrite::new(wr, LengthDelimitedCodec::new());
    // let serialized = Arc::new(Mutex::new(SymmetricallyFramed::new(
    //     length_delimited_write,
    //     SymmetricalJson::<Packet>::default(),
    // )));

    // let serialized_clone = serialized.clone();

    // tokio::spawn(async move {
    //     if let Ok(event) = rdev::listen(move |event| {
    //         let serialized_clone = serialized_clone.clone();
    //         tokio::spawn(async move {
    //             let serialized_clone = serialized_clone.clone();
    //             let mut s = serialized_clone.lock().await;
    //             s.send(Packet::Command(event)).await?;
    //
    //             Ok::<(), io::Error>(())
    //         });
    //     }) {
    //         println!("GOT {:?}", event);
    //     };
    //     Ok::<(), io::Error>(())
    // });

    let length_delimited_read = FramedRead::new(rd, LengthDelimitedCodec::new());

    let mut deserialized =
        SymmetricallyFramed::new(length_delimited_read, SymmetricalJson::<Packet>::default());

    while let Some(value) = deserialized.try_next().await? {
        println!("GOT {:?}", value);
        if let Packet::Command(event) = value {
            rdev::simulate(&event.event_type);
        }
    }

    Ok(())
}
