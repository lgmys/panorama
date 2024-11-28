use std::error::Error as StdError;

use http_body_util::BodyExt;
use hyper::{body::Body, client::conn, Request, StatusCode};
use hyper_util::rt::TokioIo;

pub async fn plugin_request<B>(
    socket_path: &str,
    req: Request<B>,
) -> Result<(StatusCode, String), ()>
where
    B: Body + Send + 'static,
    B::Data: Send,
    B::Error: Into<Box<dyn StdError + Send + Sync>>,
{
    let stream = tokio::net::UnixStream::connect(socket_path)
        .await
        .expect("Failed to connect to server");
    let io = TokioIo::new(stream);

    let (mut request_sender, connection) = conn::http1::handshake(io).await.unwrap();

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Error in connection: {}", e);
        }
    });

    let res = request_sender.send_request(req).await.unwrap();
    let status = res.status();

    let body = res.collect().await.unwrap().to_bytes();
    let string = String::from_utf8_lossy(&body);

    return Ok((status, string.to_string()));
}
