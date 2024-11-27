use std::env;

use axum::{http::Request, routing::get, Json, Router};
use hyper::body::Incoming;
use plugin_shared::{Datasource, Manifest};
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

    println!("starting Discover plugin on unix socket: {}", &socket_path);

    let path = PathBuf::from(socket_path);
    let _ = tokio::fs::remove_file(&path).await;

    let uds = UnixListener::bind(path.clone()).unwrap();

    let app = Router::new().route("/info", get(handler)).route(
        "/manifest",
        get(|| async move {
            let manifest = Manifest {
                id: "discover".to_string(),
                version: "0.0.1".to_string(),
                exported_datasources: vec![Datasource {
                    id: "elasticsearch".to_string(),
                }],
            };

            Json(manifest)
        }),
    );

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
                if err.to_string().contains("error shutting down") {
                    return;
                }

                eprintln!("failed to serve connection: {err:#}");
            }
        });
    }
}

async fn handler() -> Json<String> {
    Json(format!("Hello, World!"))
}
