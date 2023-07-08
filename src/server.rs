use std::sync::{Arc, Mutex};

use anyhow::Result;
use futures::prelude::*;
use local_ip_address::local_ip;
use rdev::{Event, EventType};
use tokio::net::TcpListener;
use tokio::{io, net::TcpStream};
use tokio_serde::formats::SymmetricalJson;
use tokio_serde::SymmetricallyFramed;
use tokio_util::codec::{FramedWrite, LengthDelimitedCodec};

use crate::utils::{Packet, RdevError};

pub async fn server() -> Result<()> {
    let server = Server::new().await?;
    server.capture().await?;
    Ok(())
}

struct Server {
    last_server_mouse: Arc<Mutex<(f64, f64)>>,
    // TODO: migrate this to a hashmap of screens or something along those lines?
    client_mouse: Arc<Mutex<(f64, f64)>>,
    socket: TcpStream,
    screen: Arc<Mutex<Screen>>,
}

// TODO: eventually set up code to report screen size and other metadata, the screens will likely
// have to be defined in a hashmap or something along those lines instead of an enum
#[derive(Debug, Copy, Clone)]
enum Screen {
    Server,
    Client,
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
            client_mouse: Arc::new(std::sync::Mutex::new((1440.0, 0.0))),
            socket,
            screen: Arc::new(std::sync::Mutex::new(Screen::Server)),
        })
    }

    async fn capture(self) -> Result<()> {
        let (_, wr) = io::split(self.socket);

        let length_delimited_write = FramedWrite::new(wr, LengthDelimitedCodec::new());
        let serialized = Arc::new(tokio::sync::Mutex::new(SymmetricallyFramed::new(
            length_delimited_write,
            SymmetricalJson::<Packet>::default(),
        )));

        rdev::grab(move |event| {
            let screen = self.screen.clone();
            let mut screen = match screen.lock() {
                Ok(screen) => screen,
                Err(_) => return Some(event),
            };

            let (client, server) = match event.event_type {
                EventType::MouseMove { x, y } => {
                    let client_mouse = self.client_mouse.clone();
                    let mut client_mouse = match client_mouse.lock() {
                        Ok(client_mouse) => client_mouse,
                        Err(_) => return Some(event),
                    };

                    let server_mouse = self.last_server_mouse.clone();
                    let mut server_mouse = match server_mouse.lock() {
                        Ok(server_mouse) => server_mouse,
                        Err(_) => return Some(event),
                    };

                    let dx = x - server_mouse.0; // negative if mouse is moving left
                    let dy = y - server_mouse.1; // negative if mouse is moving up

                    // FIXME: basically from the tests that I (Om) have done, there is some issue with
                    // the treshold values and esstentially the the mouse will phase in and out of
                    // bound no matter what the treshold is. The code needs to be more nuanced than
                    // what it is right now

                    match screen.to_owned() {
                        Screen::Server if (x - 0.0).abs() <= 0.5 && dx < 0.0 => {
                            *screen = Screen::Client;
                            *client_mouse = (1440.0, y);
                            *server_mouse = (x, y);

                            (
                                Some(EventType::MouseMove {
                                    x: client_mouse.0,
                                    y: client_mouse.1,
                                }),
                                None,
                            )
                        }

                        Screen::Server => {
                            // the mouse is on the server screen

                            *server_mouse = (x, y);
                            (None, Some(event))
                        }

                        Screen::Client if (client_mouse.0 - 1440.0).abs() <= 0.5 && dx > 0.0 => {
                            // the mouse is on the client screen and is moving right
                            // switch to server screen

                            *screen = Screen::Server;
                            *client_mouse = (0.0, y);
                            *server_mouse = (x, y);

                            (
                                None,
                                Some(Event {
                                    time: event.time,
                                    name: event.name.clone(),
                                    event_type: EventType::MouseMove {
                                        x: 0.0,
                                        y: client_mouse.1,
                                    },
                                }),
                            )
                        }
                        Screen::Client => {
                            // the mouse is on the client screen

                            *client_mouse = (client_mouse.0 + dx, client_mouse.1 + dy);

                            (
                                Some(EventType::MouseMove {
                                    x: client_mouse.0,
                                    y: client_mouse.1,
                                }),
                                None,
                            )
                        }
                    }
                }
                _ => match screen.to_owned() {
                    Screen::Server => (None, Some(event)),
                    Screen::Client => (Some(event.event_type), None),
                },
            };

            // if the mouse is moving right away from 0, send the event
            let serialized = serialized.clone();

            if let Some(client) = client {
                tokio::spawn(async move {
                    let mut serialized = serialized.lock().await;
                    serialized.send(Packet::Command(client)).await?;

                    Result::<()>::Ok(())
                });
            }

            server
        })
        .map_err(|error| RdevError::Grab(error))?;

        Ok(())
    }
}
