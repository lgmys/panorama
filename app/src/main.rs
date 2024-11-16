use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use axum::{
    extract::{ConnectInfo, State},
    routing::get,
    Router,
};
use futures::stream::{self, StreamExt};
use hyper::{body::Incoming, Request};
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server,
};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, UnixListener},
    process::Command,
    sync::{mpsc, oneshot, Mutex},
};
use tokio_stream::wrappers::UnixListenerStream;
use tower::{Service, ServiceExt};

#[derive(Debug)]
struct RequestMessage {
    text: String,
    response_tx: oneshot::Sender<String>,
}

#[derive(Clone)]
struct AppState {
    pub tx: Arc<HashMap<String, mpsc::Sender<RequestMessage>>>,
}

struct Plugin {
    pub socket_path: String,
    pub executable_path: String,
    pub id: String,
}

#[tokio::main]
async fn main() {
    let mut tx_map = HashMap::<String, mpsc::Sender<RequestMessage>>::new();
    let mut rx_map = HashMap::<String, Arc<Mutex<mpsc::Receiver<RequestMessage>>>>::new();

    let mut plugin_listeners: Vec<(String, UnixListener)> = Vec::new();

    let plugins: Vec<Plugin> = vec![
        Plugin {
            socket_path: "/tmp/panorama-discover.sock".to_string(),
            id: "discover".to_string(),
            executable_path: "target/debug/discover".to_string(),
        },
        Plugin {
            socket_path: "/tmp/panorama-discover2.sock".to_string(),
            id: "discover2".to_string(),
            executable_path: "target/debug/discover".to_string(),
        },
    ];

    for plugin in &plugins {
        let (tx, rx) = mpsc::channel(32);
        tx_map.insert(plugin.id.clone(), tx);
        rx_map.insert(plugin.id.clone(), Arc::new(Mutex::new(rx)));

        if fs::metadata(&plugin.socket_path).await.is_ok() {
            fs::remove_file(&plugin.socket_path).await.unwrap();
        }

        // Create a Unix listener for the plugin
        let plugin_listener = UnixListener::bind(&plugin.socket_path).unwrap();
        println!("Socket created and listening at {}", &plugin.socket_path);

        plugin_listeners.push((plugin.id.clone(), plugin_listener));

        // Spawn the child process
        Command::new(&plugin.executable_path)
            .arg(&plugin.socket_path)
            .spawn()
            .expect("Failed to spawn child process");
    }

    let tx_map = Arc::new(tx_map);

    let app = Router::new()
        .route("/", get(root))
        .with_state(AppState { tx: tx_map.clone() });

    let mut make_service = app.into_make_service_with_connect_info::<SocketAddr>();
    let external_listener = TcpListener::bind("0.0.0.0:3001").await.unwrap();

    let mut plugin_listener_stream =
        stream::select_all(plugin_listeners.into_iter().map(|(plugin_id, listener)| {
            UnixListenerStream::new(listener).map(move |connection| (plugin_id.clone(), connection))
        }));

    loop {
        tokio::select! {
            Ok((socket, remote_addr)) = external_listener.accept() => {
                // NOTE: spawn external http listener
                let tower_service = make_service.call(remote_addr).await.unwrap();

                tokio::spawn(async move {
                    let socket = TokioIo::new(socket);

                    let hyper_service = hyper::service::service_fn(move |request: Request<Incoming>| {
                        tower_service.clone().oneshot(request)
                    });

                    if let Err(err) = server::conn::auto::Builder::new(TokioExecutor::new())
                        .serve_connection_with_upgrades(socket, hyper_service)
                        .await
                    {
                        eprintln!("failed to serve connection: {err:#}");
                    }
                });
        }
        Some((plugin_id, Ok(mut plugin_socket))) = plugin_listener_stream.next() => {
            let plugin_id = plugin_id.clone();
            // NOTE: this will be called only once per plugin
            let plugin_rx = rx_map.get(&plugin_id).unwrap().clone();

            let tx_map = tx_map.clone();

            // NOTE: spawn plugin internal socket listener
            tokio::spawn(async move {
                while let Some(request) = plugin_rx.lock().await.recv().await {
                    let message = request.text;
                    plugin_socket.write_all(message.as_bytes()).await.unwrap();
                    println!("Sent to {}: {}", &plugin_id, &message);

                    // TODO: make it a function
                    let (response_tx, response_rx) = oneshot::channel();
                    // Send the request message
                    let request2 = RequestMessage {
                        text: "ha 2".to_string(),
                        response_tx,
                    };

                    // NOTE: its important to check if the plugin is not trying to call a method on
                    // itself
                    if plugin_id != "discover2" {
                        if let Err(_) = tx_map.get("discover2").unwrap().send(request2).await {
                            dbg!("Failed to send message");
                        } else {
                            // Wait for the response
                            match response_rx.await {
                                Ok(response) => dbg!(response),
                                Err(_) => dbg!("Failed to receive response".to_string()),
                            };
                        }
                    }
                    // TODO: end make it a function

                    let mut buf = vec![0; 1024];
                    let n = plugin_socket.read(&mut buf).await.unwrap();
                    let response = String::from_utf8_lossy(&buf[..n]);

                    println!("Relaying response from {}: {}", &plugin_id, &response);

                    if let Err(e) = request.response_tx.send(response.to_string()) {
                        eprintln!("Failed to send response: {}", e);
                    }
                }
            });
        }}
    }
}

async fn root(
    ConnectInfo(info): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
) -> &'static str {
    dbg!(info);

    // Create a oneshot channel for the response
    let (response_tx, response_rx) = oneshot::channel();

    // Send the request message
    let request = RequestMessage {
        text: "ha!".to_string(),
        response_tx,
    };

    if let Err(_) = state.tx.get("discover").unwrap().send(request).await {
        dbg!("Failed to send message");
    } else {
        // Wait for the response
        match response_rx.await {
            Ok(response) => dbg!(response),
            Err(_) => dbg!("Failed to receive response".to_string()),
        };
    }

    return "hello";
}
