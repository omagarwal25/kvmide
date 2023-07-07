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

use crate::utils::{Packet, RdevError};

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
    last_server_mouse: Arc<std::sync::Mutex<(f64, f64)>>,
    // TODO: migrate this to a hashmap of screens or something along those lines?
    client_mouse: Arc<std::sync::Mutex<(f64, f64)>>,
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
            last_server_mouse: Arc::new(std::sync::Mutex::new((0.0, 0.0))),
            client_mouse: Arc::new(std::sync::Mutex::new((0.0, 0.0))),
            socket,
        })
    }

    async fn capture(mut self) -> Result<()> {
        let (_, wr) = io::split(self.socket);

        let length_delimited_write = FramedWrite::new(wr, LengthDelimitedCodec::new());
        let serialized = Arc::new(Mutex::new(SymmetricallyFramed::new(
            length_delimited_write,
            SymmetricalJson::<Packet>::default(),
        )));

        rdev::grab(move |event| {
            let mut server_mouse = self.last_server_mouse.clone().lock();

            let mut server_mouse = match server_mouse {
                Ok(server_mouse) => server_mouse,
                Err(_) => return Some(event),
            };

            if let EventType::MouseMove { x, y } = event.event_type {
                let mut client_mouse = self.client_mouse.clone().lock();
                let mut client_mouse = match client_mouse {
                    Ok(client_mouse) => client_mouse,
                    Err(_) => return Some(event),
                };

                let dx = x - server_mouse.0;
                let dy = y - server_mouse.1;

                if (x - 0.0).abs() < 0.5 && dx > 0.0 && (client_mouse.0 - 1440.0).abs() > 0.5 {
                    *server_mouse = (x, y);
                    return Some(event);
                }

                if (x - 0.0).abs() > 0.5 {
                    *server_mouse = (x, y);
                    return Some(event);
                }

                if (x - 0.0).abs() < 0.5 && dx < 0.0 {
                    *client_mouse = (0.0, y);
                }

                // basically what we need to do is calculate the dx and dy of the mouse to the last
                // mouse position and send that to the other computer

                let event = EventType::MouseMove {
                    x: client_mouse.0 + dx,
                    y: client_mouse.1 + dy,
                };

                *client_mouse = (client_mouse.0 + dx, client_mouse.1 + dy);

                let serialized = serialized.clone();

                tokio::spawn(async move {
                    let mut serialized = serialized.lock().await;
                    serialized.send(Packet::Command(event)).await;
                });

                return None;
            }

            if (server_mouse.0 - 0.0).abs() > 0.5 {
                return Some(event);
            }

            // if the mouse is moving right away from 0, send the event
            let serialized = serialized.clone();

            tokio::spawn(async move {
                let mut serialized = serialized.lock().await;
                serialized.send(Packet::Command(event.event_type)).await;
            });

            None
        })
        .map_err(|error| RdevError::Grab(error))?;

        Ok(())
    }
}
