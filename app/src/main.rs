use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use bytes::Bytes;
use futures::{future, StreamExt};
use http_body_util::{BodyExt, Empty};
use hyper::client::conn;
use hyper::Request;
use hyper_util::rt::TokioIo;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio::{
    fs,
    process::{Child, Command},
    sync::Mutex,
    task,
    time::sleep,
};

type LoadedPluginsRegistry = Arc<Mutex<HashMap<String, Manifest>>>;

#[derive(Deserialize, Clone, Debug)]
struct Plugin {
    pub binary_path: String,
    pub socket_path: String,
}

#[derive(Deserialize, Clone, Debug)]
struct PanoramaConfig {
    pub plugins: HashMap<String, Plugin>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Manifest {
    pub id: String,
}

#[derive(Clone)]
struct AppState {
    pub config: Arc<PanoramaConfig>,
    pub loaded_plugins: Arc<Mutex<HashMap<String, Manifest>>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = "./panorama.toml";
    let file = fs::read_to_string(&config_path).await.unwrap();
    let config: PanoramaConfig = toml::from_str(&file).unwrap();
    let loaded_plugins = Arc::new(Mutex::new(HashMap::<String, Manifest>::new()));

    let mut tasks: Vec<task::JoinHandle<_>> = vec![];

    let values = config.plugins.clone().into_values();

    for plugin_config in values {
        let loaded_plugins = loaded_plugins.clone();

        let task = task::spawn(async move {
            monitor_process(
                loaded_plugins,
                plugin_config.binary_path.clone(),
                plugin_config.socket_path.clone(),
            )
            .await;
        });
        tasks.push(task);
    }

    let server_task = run_axum_server(Arc::new(config.clone()), loaded_plugins.clone());

    tokio::select! {
        _ = future::select_all(tasks) => {
            eprintln!("File monitoring task exited.");
        }
        _ = server_task => {
            eprintln!("Axum server exited.");
        }
    }

    Ok(())
}

async fn run_axum_server(config: Arc<PanoramaConfig>, loaded_plugins: LoadedPluginsRegistry) {
    let app = Router::new()
        .route("/api/plugins", get(get_plugin_status))
        .route("/api/plugin/:plugin_id/*rest", get(proxy_to_backend))
        .with_state(AppState {
            config,
            loaded_plugins: loaded_plugins.clone(),
        });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn monitor_process(
    loaded_plugins: LoadedPluginsRegistry,
    process_path: String,
    socket_path: String,
) {
    println!("Starting backend process monitoring for {}", process_path);

    let mut last_modified = get_last_modified(&process_path).await;

    let mut current_process: Option<Child> = None;

    let mut first_restart = true;

    loop {
        // Watch for file changes
        let current_modified = get_last_modified(&process_path).await;

        if current_modified != last_modified || first_restart {
            if first_restart {
                first_restart = false;
            }

            println!(
                "Detected change in {}, restarting backend process...",
                process_path
            );
            last_modified = current_modified;

            // Kill the existing process if it's running
            if let Some(mut process) = current_process.take() {
                if let Err(err) = process.kill().await {
                    eprintln!("Failed to kill existing process: {:?}", err);
                } else {
                    println!("Killed existing backend process",);
                }
            }

            // Start a new backend process
            match restart_backend_process(loaded_plugins.clone(), &process_path, &socket_path).await
            {
                Ok(process) => {
                    current_process = Some(process); // Keep track of the new process
                }
                Err(err) => {
                    eprintln!("Error restarting backend process: {:?}", err);
                }
            }
        }

        // Sleep for a while before checking again
        sleep(Duration::from_secs(2)).await;
    }
}

// Helper function to get the last modified time of a file
async fn get_last_modified(path: &str) -> Option<SystemTime> {
    match fs::metadata(path).await {
        Ok(metadata) => metadata.modified().ok(),
        Err(err) => {
            eprintln!("Failed to get metadata for {}: {:?}", path, err);
            None
        }
    }
}

// Helper function to restart the backend process
async fn restart_backend_process(
    loaded_plugins: LoadedPluginsRegistry,
    process_path: &str,
    socket_path: &str,
) -> Result<Child, (hyper::StatusCode, String)> {
    if std::path::Path::new(socket_path).exists() {
        match fs::remove_file(&socket_path).await {
            Ok(_) => {}
            Err(err) => {
                eprintln!("Failed to remove socket file {}: {:?}", socket_path, err);
            }
        }
    }

    match Command::new(process_path).arg(socket_path).spawn() {
        Ok(process) => {
            println!("Backend process restarted, pid: {}", process.id().unwrap());

            sleep(Duration::from_secs(5)).await;

            let manifest = fetch_data_from_plugin(socket_path, "/manifest")
                .await
                .unwrap();

            if let Ok(manifest) = serde_json::from_str::<Manifest>(&manifest) {
                println!("Plugin manifest read: {:?}", &manifest);
                loaded_plugins
                    .lock()
                    .await
                    .insert(manifest.id.clone(), manifest.clone());
                Ok(process)
            } else {
                Err((
                    hyper::StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to parse plugin manifest"),
                ))
            }
        }
        Err(err) => Err((
            hyper::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to restart backend: {:?}", err),
        )),
    }
}

async fn fetch_data_from_plugin(socket_path: &str, uri: &str) -> Result<String, ()> {
    println!("Fetching data from {}", uri);

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
        .method("GET")
        .uri(uri)
        .body(Empty::<Bytes>::new())
        .unwrap();

    let res = request_sender.send_request(request).await.unwrap();
    let body = res.collect().await.unwrap().to_bytes();
    let string = String::from_utf8_lossy(&body);

    return Ok(string.to_string());
}

async fn get_plugin_status(State(state): State<AppState>) -> Json<Value> {
    let plugins = state.loaded_plugins.lock().await.clone();
    let plugins = serde_json::to_value(plugins).unwrap();

    Json(plugins)
}

// Proxy HTTP requests to the backend process
async fn proxy_to_backend(
    State(state): State<AppState>,
    Path((plugin_id, rest)): Path<(String, String)>,
) -> Result<Json<Value>, (hyper::StatusCode, String)> {
    let plugin_config = state.config.plugins.get(&plugin_id).unwrap();
    let target_path = rest;

    let res =
        fetch_data_from_plugin(&plugin_config.socket_path, &format!("/{}", &target_path)).await;

    match res {
        Ok(response_string) => Ok(Json(serde_json::from_str(&response_string).unwrap())),
        Err(_) => Err((
            hyper::StatusCode::INTERNAL_SERVER_ERROR,
            "error".to_string(),
        )),
    }
}
