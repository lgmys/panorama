use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{ConnectInfo, State},
    routing::get,
    Router,
};
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
    pub tx: mpsc::Sender<RequestMessage>,
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);

    let app = Router::new()
        .route("/", get(root))
        .with_state(AppState { tx });

    let shared_rx = Arc::new(Mutex::new(rx));

    let mut make_service = app.into_make_service_with_connect_info::<SocketAddr>();
    let external_listener = TcpListener::bind("0.0.0.0:3001").await.unwrap();

    let socket_path = "/tmp/rust-unix-socket-example.sock";
    let child_process_executable = "target/debug/discover";

    // Clean up any existing socket file
    if fs::metadata(socket_path).await.is_ok() {
        fs::remove_file(socket_path).await.unwrap();
    }

    // Create a Unix listener for the plugin
    let plugin_listener = UnixListener::bind(socket_path).unwrap();
    println!("Socket created and listening at {}", socket_path);

    // Spawn the child process
    Command::new(child_process_executable)
        .arg(socket_path)
        .spawn()
        .expect("Failed to spawn child process");

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
        Ok((mut plugin_socket, _)) = plugin_listener.accept() => {
            let shared_rx = shared_rx.clone();
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

    if let Err(_) = state.tx.send(request).await {
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
