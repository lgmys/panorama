use axum::{extract::Path, routing::get, Router};
use hyper_util::rt::TokioIo;
use std::process::Command;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new().route("/api/plugin/:plugin_id", get(proxy_to_backend));

    let socket_path = "/tmp/backend.sock";
    let process_path = "target/debug/discover";

    start_backend_process(process_path, socket_path)
        .await
        .unwrap();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

// Proxy HTTP requests to the backend process
async fn proxy_to_backend(
    Path(plugin_id): Path<String>,
) -> Result<String, (hyper::StatusCode, String)> {
    let socket_path = "/tmp/backend.sock";

    let stream = tokio::net::UnixStream::connect(socket_path)
        .await
        .expect("Failed to connect to server");
    let io = TokioIo::new(stream);

    use bytes::Bytes;
    use http_body_util::Empty;
    use hyper::client::conn;
    use hyper::{Request, StatusCode};

    let (mut request_sender, connection) = conn::http1::handshake(io).await.unwrap();

    // spawn a task to poll the connection and drive the HTTP state
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Error in connection: {}", e);
        }
    });

    let request = Request::builder()
        .method("GET")
        .uri(&format!("/",))
        .body(Empty::<Bytes>::new())
        .unwrap();

    let res = request_sender.send_request(request).await.unwrap();
    assert!(res.status() == StatusCode::OK);

    println!("{}", res.status());

    Ok("".to_string())
}

// Start the backend process
async fn start_backend_process(
    process_path: &str,
    socket_path: &str,
) -> Result<(), (hyper::StatusCode, String)> {
    println!("Starting backend process on {}", socket_path);

    if std::path::Path::new(socket_path).exists() {
        match tokio::fs::remove_file(&socket_path).await {
            Ok(_) => {}
            Err(_) => {}
        };
    }

    match Command::new(process_path).arg(socket_path).spawn() {
        Ok(process) => {
            println!("Backend process started, pid: {}", process.id());
            Ok(())
        }
        Err(err) => Err((
            hyper::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to start backend: {:?}", err),
        )),
    }
}
