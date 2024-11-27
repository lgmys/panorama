use http_body_util::BodyExt;
use hyper::{client::conn, Request};
use hyper_util::rt::TokioIo;

pub async fn plugin_request(socket_path: &str, req: Request<String>) -> Result<String, ()> {
    let stream = tokio::net::UnixStream::connect(socket_path)
        .await
        .expect("Failed to connect to server");
    let io = TokioIo::new(stream);

    let (mut request_sender, connection) = conn::http1::handshake(io).await.unwrap();

    // spawn a task to poll the connection and drive the HTTP state
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Error in connection: {}", e);
        }
    });

    let res = request_sender.send_request(req).await.unwrap();
    let body = res.collect().await.unwrap().to_bytes();
    let string = String::from_utf8_lossy(&body);

    return Ok(string.to_string());
}
