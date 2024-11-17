use axum::{extract::Path, routing::get, Router};
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper_util::client::legacy::Client;
use hyperlocal::{UnixClientExt, UnixConnector, Uri};
use std::process::Command;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new().route("/proxy/:endpoint", get(proxy_to_backend));

    let socket_path = "/tmp/backend.sock";

    tokio::fs::remove_file(&socket_path).await.unwrap();
    // Ensure the backend process is running
    if !std::path::Path::new(socket_path).exists() {
        start_backend_process(socket_path).unwrap();
    }

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

// Proxy HTTP requests to the backend process
async fn proxy_to_backend(
    Path(endpoint): Path<String>,
) -> Result<String, (hyper::StatusCode, String)> {
    let socket_path = "/tmp/backend.sock";

    let url = Uri::new(socket_path, "/").into();

    let client: Client<UnixConnector, Full<Bytes>> = Client::unix();

    let mut response = client.get(url).await.unwrap();

    while let Some(frame_result) = response.frame().await {
        let frame = frame_result.unwrap();

        if let Some(segment) = frame.data_ref() {
            dbg!(segment);
        }
    }

    Ok("".to_string())
}

// Start the backend process
fn start_backend_process(socket_path: &str) -> Result<(), (hyper::StatusCode, String)> {
    println!("Starting backend process on {}", socket_path);

    match Command::new("target/debug/discover")
        .arg(socket_path)
        .spawn()
    {
        Ok(_) => {
            println!("Backend process started.");
            Ok(())
        }
        Err(err) => Err((
            hyper::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to start backend: {:?}", err),
        )),
    }
}
