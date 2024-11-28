use axum::{http::Request, Router};
use hyper::body::Incoming;
use serde::Serialize;
use std::path::PathBuf;
use tokio::net::UnixListener;
use tower::Service;

use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server,
};

#[derive(Serialize)]
pub struct Datasource {
    pub kind: String,
}

#[derive(Serialize)]
pub struct Manifest {
    pub id: String,
    pub version: String,
    pub exported_datasources: Vec<Datasource>,
}

pub async fn bootstrap_plugin(socket_path: &str, app: Router) {
    let path = PathBuf::from(socket_path);
    let _ = tokio::fs::remove_file(&path).await;
    let uds = UnixListener::bind(path.clone()).unwrap();

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
