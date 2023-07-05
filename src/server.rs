use std::sync::Arc;

use futures::prelude::*;
use serde_json::{json, Value};
use tokio::net::{TcpListener, TcpStream};
use tokio::{io, sync::Mutex};
use tokio_serde::formats::SymmetricalJson;
use tokio_serde::SymmetricallyFramed;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

use crate::Packet;

pub async fn server() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6142").await?;

    let (mut socket, _) = listener.accept().await?;

    let (mut rd, wr) = io::split(socket);

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
    //                         // Copy the data back to socket
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

    let serialized_clone = serialized.clone();

    tokio::spawn(async move {
        if let Ok(event) = rdev::listen(move |event| {
            let serialized_clone = serialized_clone.clone();
            tokio::spawn(async move {
                let serialized_clone = serialized_clone.clone();
                let mut s = serialized_clone.lock().await;
                s.send(Packet::Command(event)).await?;

                Ok::<(), io::Error>(())
            });
        }) {
            println!("GOT {:?}", event);
        };
        Ok::<(), io::Error>(())
    });

    Ok(())
}

