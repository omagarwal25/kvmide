use std::sync::Arc;

use anyhow::Result;
use futures::prelude::*;
use local_ip_address::local_ip;
use rdev::{Event, EventType};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::{io, net::TcpStream};
use tokio_serde::formats::SymmetricalJson;
use tokio_serde::SymmetricallyFramed;
use tokio_util::codec::{FramedWrite, LengthDelimitedCodec};

use crate::utils::RdevError;
use crate::Packet;

pub async fn server() -> Result<()> {
    let server = Server::new().await?;
    server.capture().await?;

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

    Ok(())
}

struct Server {
    last_mouse: (f64, f64),
    socket: TcpStream,
}

impl Server {
    async fn new() -> Result<Self> {
        let hostname = local_ip()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to get hostname"))?;
        let listener = TcpListener::bind(format!("{:?}:6142", hostname)).await?;

        println!("Listening on {:?}", listener.local_addr()?);

        let (socket, _) = listener.accept().await?;

        Ok(Self {
            last_mouse: (0.0, 0.0),
            socket,
        })
    }

    async fn capture(self) -> Result<()> {
        let (_, wr) = io::split(self.socket);

        let length_delimited_write = FramedWrite::new(wr, LengthDelimitedCodec::new());
        let serialized = Arc::new(Mutex::new(SymmetricallyFramed::new(
            length_delimited_write,
            SymmetricalJson::<Packet>::default(),
        )));

        rdev::grab(move |event| {
            let away = false;

            if let EventType::MouseMove { x, y } = event.event_type {
                // check if the mouse has moved to the left and the last mouse was within 0.5 of 0

                if (x - 0.0).abs() < 0.5 && (x - self.last_mouse.0).abs() > 0.0 {
                    return Some(event);
                }

                self.last_mouse = (x, y);
            }

            if (self.last_mouse.0 - 0.0).abs() > 0.5 {
                return Some(event);
            }

            // if the mouse is moving right away from 0, send the event

            let serialized = serialized.clone();
            tokio::spawn(async move {
                let mut serialized = serialized.lock().await;
                serialized.send(Packet::Command(event)).await;
            });

            None
        })
        .map_err(|error| RdevError::Grab(error))?;

        Ok(())
    }
}
