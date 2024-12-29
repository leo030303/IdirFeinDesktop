use iced::futures;
use iced::stream;
use iced::widget::text;

use futures::channel::mpsc;
use futures::sink::SinkExt;
use futures::stream::{Stream, StreamExt};

use async_tungstenite::tungstenite;
use std::fmt;

// const LORO_SERVER: &str = "ws://127.0.0.1:8000/loro";

pub fn connect(server_url: String) -> impl Stream<Item = Event> {
    stream::channel(100, |mut output| async move {
        let mut state = ConnectionState::Disconnected;

        loop {
            match &mut state {
                ConnectionState::Disconnected => {
                    match async_tungstenite::tokio::connect_async(&server_url).await {
                        Ok((websocket, _)) => {
                            let (sender, receiver) = mpsc::channel(100);

                            let _ = output.send(Event::Connected(Connection(sender))).await;

                            state = ConnectionState::Connected(websocket, receiver);
                        }
                        Err(_) => {
                            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

                            let _ = output.send(Event::Disconnected).await;
                        }
                    }
                }
                ConnectionState::Connected(websocket, input) => {
                    let mut fused_websocket = websocket.by_ref().fuse();

                    futures::select! {
                        received = fused_websocket.select_next_some() => {
                            match received {
                                Ok(tungstenite::Message::Text(message)) => {
                                   let _ = output.send(Event::MessageReceived(ServerMessage::User(message))).await;
                                }
                                Err(_) => {
                                    let _ = output.send(Event::Disconnected).await;

                                    state = ConnectionState::Disconnected;
                                }
                                Ok(_) => continue,
                            }
                        }

                        message = input.select_next_some() => {
                            let result = websocket.send(tungstenite::Message::Text(message.to_string())).await;

                            if result.is_err() {
                                let _ = output.send(Event::Disconnected).await;

                                state = ConnectionState::Disconnected;
                            }
                        }
                    }
                }
            }
        }
    })
}

#[derive(Debug)]
enum ConnectionState {
    Disconnected,
    Connected(
        async_tungstenite::WebSocketStream<async_tungstenite::tokio::ConnectStream>,
        mpsc::Receiver<ServerMessage>,
    ),
}

#[derive(Debug, Clone)]
pub enum Event {
    Connected(Connection),
    Disconnected,
    MessageReceived(ServerMessage),
}

#[derive(Debug, Clone)]
pub struct Connection(mpsc::Sender<ServerMessage>);

impl Connection {
    pub fn send(&mut self, message: ServerMessage) {
        self.0.try_send(message).expect("Send message to server");
    }
}

#[derive(Debug, Clone)]
pub enum ServerMessage {
    Connected,
    Disconnected,
    User(String),
}

impl ServerMessage {
    pub fn new(message: &str) -> Option<Self> {
        if message.is_empty() {
            None
        } else {
            Some(Self::User(message.to_string()))
        }
    }

    pub fn connected() -> Self {
        ServerMessage::Connected
    }

    pub fn disconnected() -> Self {
        ServerMessage::Disconnected
    }

    pub fn as_str(&self) -> &str {
        match self {
            ServerMessage::Connected => "Connected successfully!",
            ServerMessage::Disconnected => "Connection lost... Retrying...",
            ServerMessage::User(message) => message.as_str(),
        }
    }
}

impl fmt::Display for ServerMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<'a> text::IntoFragment<'a> for &'a ServerMessage {
    fn into_fragment(self) -> text::Fragment<'a> {
        text::Fragment::Borrowed(self.as_str())
    }
}
