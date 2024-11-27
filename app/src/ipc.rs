use bytes::Bytes;
use http_body_util::{BodyExt, Empty};
use hyper::{client::conn, Method, Request};
use hyper_util::rt::TokioIo;
use serde_json::Value;

pub struct PluginRequest {
    pub method: Method,
    pub uri: String,
    pub value: Option<Value>,
}

pub async fn plugin_request(socket_path: &str, req: PluginRequest) -> Result<String, ()> {
    println!("Fetching data from {}", &req.uri);

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

    let request = Request::builder()
        .method(&req.method)
        .uri(&req.uri)
        .body(Empty::<Bytes>::new())
        .unwrap();

    let res = request_sender.send_request(request).await.unwrap();
    let body = res.collect().await.unwrap().to_bytes();
    let string = String::from_utf8_lossy(&body);

    return Ok(string.to_string());
}
