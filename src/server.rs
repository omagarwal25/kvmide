use std::sync::Arc;

use futures::prelude::*;
use tokio::io;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_serde::formats::SymmetricalJson;
use tokio_serde::SymmetricallyFramed;
use tokio_util::codec::{FramedWrite, LengthDelimitedCodec};
use local_ip_address::local_ip;

use crate::Packet;

pub async fn server() -> io::Result<()> {
    let hostname = local_ip().map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to get hostname"))?;
    let listener = TcpListener::bind(format!("{:?}:6142", hostname)).await?;

    println!("Listening on {:?}", listener.local_addr()?);

    let (socket, _) = listener.accept().await?;

    let (_, wr) = io::split(socket);

    let length_delimited_write = FramedWrite::new(wr, LengthDelimitedCodec::new());
    let serialized = Arc::new(Mutex::new(SymmetricallyFramed::new(
        length_delimited_write,
        SymmetricalJson::<Packet>::default(),
    )));

    // tokio::spawn(async move {
    //     loop {
    //         tokio::spawn(async move {
    //             let mut buf = vec![0; 128];
    //
    //             loop {
    //                 match rd.read(&mut buf).await {
    //                     // Return value of `Ok(0)` signifies that the remote has closed
    //                     // the connection. At this point, the task should close its
    //                     // handle and return.
    //                     Ok(0) => return,
    //                     Ok(n) => {
    //                         // Copy s is me typing from both laptops at once?the data back to socket
    //                     }
    //                     Err(_) => {
    //                         // Unexpected socket error. There isn't much we can do
    //                         // here so just stop processing.
    //                         return;
    //                     }
    //                 }
    //             }
    //         });
    //     }
    // });

    if let Ok(event) = rdev::grab(move |event| {
        // println!("d");
        let serialized = serialized.clone();
        tokio::spawn(async move {
            let mut serialized = serialized.lock().await;
            serialized.send(Packet::Command(event)).await.unwrap();
        });

        None
    }) {
        println!("GOT {:?}", event);
    };

    Ok(())
}
