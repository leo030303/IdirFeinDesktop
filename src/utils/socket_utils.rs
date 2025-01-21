use async_tungstenite::tungstenite::client::IntoClientRequest;
use iced::futures;
use iced::stream;
use iced::widget::text;
use reqwest::header::HeaderValue;
use url::Url;

use futures::channel::mpsc;
use futures::sink::SinkExt;
use futures::stream::{Stream, StreamExt};

use async_tungstenite::tungstenite;
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use std::time::Duration;

use crate::utils::sync_utils::ServerFileRequest;

use super::auth_utils::get_auth_pair;
use super::sync_utils::SyncManager;

// const LORO_SERVER: &str = "ws://127.0.0.1:8000/loro";

pub fn connect(
    server_url: String,
    folders_to_sync: HashMap<String, PathBuf>,
    ignore_list: Vec<String>,
    default_data_storage_folder: PathBuf,
) -> impl Stream<Item = Event> {
    stream::channel(100, |mut output| async move {
        let mut state = ConnectionState::Disconnected;
        let (client_id, auth_token) = get_auth_pair();

        let mut post_server_url_with_auth = Url::parse(&(String::from("http://") + &server_url)) // TODO Change to https for prod
            .unwrap()
            .join("/sync/initialise")
            .unwrap();
        post_server_url_with_auth
            .query_pairs_mut()
            .append_pair("client_id", &client_id);

        let mut sync_manager =
            SyncManager::new(folders_to_sync, ignore_list, default_data_storage_folder);

        let res = loop {
            match reqwest::Client::new()
                .post(post_server_url_with_auth.as_ref())
                .body(sync_manager.get_initialiser_data())
                .bearer_auth(&auth_token)
                .send()
                .await
            {
                Ok(res) => {
                    break res;
                }
                Err(err) => {
                    println!("Sync post error, will retry: {err:?}");
                }
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        };

        let response_bytes = res.bytes().await.unwrap();

        let mut server_file_requests_vec: Vec<ServerFileRequest> =
            serde_json::from_slice(&response_bytes).unwrap();

        server_file_requests_vec.sort_unstable();

        println!("{server_file_requests_vec:?}");

        let mut server_file_requests_stream =
            futures::stream::iter(server_file_requests_vec).fuse();

        let mut socket_server_url_with_auth = Url::parse(&(String::from("ws://") + &server_url)) // TODO Change to wss for prod
            .unwrap()
            .join("sync/stream")
            .unwrap();
        socket_server_url_with_auth
            .query_pairs_mut()
            .append_pair("client_id", &client_id);

        loop {
            match &mut state {
                ConnectionState::Disconnected => {
                    println!("Disconnected");
                    let mut request = socket_server_url_with_auth
                        .as_str()
                        .into_client_request()
                        .unwrap();
                    let _ = request.headers_mut().insert(
                        reqwest::header::AUTHORIZATION,
                        HeaderValue::from_str(&format!("Bearer {}", auth_token)).unwrap(),
                    );
                    match async_tungstenite::tokio::connect_async(request).await {
                        Ok((websocket, _)) => {
                            let (sender, receiver) = mpsc::channel(100);

                            let _ = output.send(Event::Connected(Connection(sender))).await;

                            state = ConnectionState::Connected(websocket, receiver);
                        }
                        Err(err) => {
                            println!("Couldn't connect {err:?}");
                            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

                            let _ = output.send(Event::Disconnected).await;
                        }
                    }
                }
                ConnectionState::Connected(websocket, input) => {
                    println!("Connected");
                    let mut fused_websocket = websocket.by_ref().fuse();

                    futures::select! {

                        server_request = server_file_requests_stream.select_next_some() => {
                                if let Some(response) = sync_manager.handle_server_request(server_request) {
                                    if websocket.send(tungstenite::Message::Text(serde_json::to_string(&response).unwrap())).await.is_err() {
                                        let _ = output.send(Event::Disconnected).await;
                                        state = ConnectionState::Disconnected;
                                    }
                                }
                        }

                        received = fused_websocket.select_next_some() => {
                            match received {
                                Ok(tungstenite::Message::Text(message)) => {
                                   let deserialised_message: ServerFileRequest = serde_json::from_str(&message).unwrap();
                                   let message_to_send_option = sync_manager.handle_server_request(deserialised_message);
                                   if let Some(message_to_send) = message_to_send_option {
                                       let result = websocket.send(tungstenite::Message::Text(serde_json::to_string(&message_to_send).unwrap())).await;

                                       if result.is_err() {
                                           let _ = output.send(Event::Disconnected).await;

                                           state = ConnectionState::Disconnected;
                                       }

                                   }
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
