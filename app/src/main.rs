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
use tower::{Service, ServiceExt};

#[derive(Debug)]
struct RequestMessage {
    text: String,
    response_tx: oneshot::Sender<String>,
}

#[derive(Clone)]
struct AppState {
    pub tx: HashMap<String, mpsc::Sender<RequestMessage>>,
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

    let mut plugin_listeners: Vec<UnixListener> = Vec::new();

    let plugins: Vec<Plugin> = vec![Plugin {
        socket_path: "/tmp/rust-unix-socket-example.sock".to_string(),
        id: "discover".to_string(),
        executable_path: "target/debug/discover".to_string(),
    }];

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

        plugin_listeners.push(plugin_listener);

        // Spawn the child process
        Command::new(&plugin.executable_path)
            .arg(&plugin.socket_path)
            .spawn()
            .expect("Failed to spawn child process");
    }

    let app = Router::new()
        .route("/", get(root))
        .with_state(AppState { tx: tx_map });

    let mut make_service = app.into_make_service_with_connect_info::<SocketAddr>();
    let external_listener = TcpListener::bind("0.0.0.0:3001").await.unwrap();

    let mut plugin_listener_stream = stream::select_all(
        plugin_listeners
            .into_iter()
            .map(|listener| tokio_stream::wrappers::UnixListenerStream::new(listener)),
    );

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
        Some(Ok(mut plugin_socket)) = plugin_listener_stream.next() => {
            // NOTE: this will be called only once per plugin
            let shared_rx = rx_map.get("discover").unwrap().clone();

            // NOTE: spawn plugin internal socket listener
            tokio::spawn(async move {
                while let Some(request) = shared_rx.lock().await.recv().await {
                    let message = request.text;
                    plugin_socket.write_all(message.as_bytes()).await.unwrap();
                    println!("Sent to child: {}", &message);

                    let mut buf = vec![0; 1024];
                    let n = plugin_socket.read(&mut buf).await.unwrap();
                    let response = String::from_utf8_lossy(&buf[..n]);

                    // Send the response back
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
    // TODO: send something to all plugins
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
