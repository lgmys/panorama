use std::env;

use axum::{http::Request, routing::get, Router};
use hyper::body::Incoming;
use std::path::PathBuf;
use tokio::net::UnixListener;
use tower::Service;

use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server,
};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let socket_path = &args[1];

    println!("starting plugin on {}", &socket_path);
    let path = PathBuf::from(socket_path);

    let _ = tokio::fs::remove_file(&path).await;

    let uds = UnixListener::bind(path.clone()).unwrap();
    let app = Router::new().route("/", get(handler));

    loop {
        let (socket, _remote_addr) = uds.accept().await.unwrap();

        let tower_service = app.clone();

        tokio::spawn(async move {
            let socket = TokioIo::new(socket);

            let hyper_service = hyper::service::service_fn(move |request: Request<Incoming>| {
                tower_service.clone().call(request)
            });

            if let Err(err) = server::conn::auto::Builder::new(TokioExecutor::new())
                .serve_connection_with_upgrades(socket, hyper_service)
                .await
            {
                eprintln!("failed to serve connection: {err:#}");
            }
        });
    }
}

async fn handler() -> &'static str {
    "Hello, World!"
}
