use axum::{extract::Path, routing::get, Router};
use hyper_util::rt::TokioIo;
use std::time::{Duration, SystemTime};
use tokio::{fs, process::Child, process::Command, time::sleep};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let socket_path = "/tmp/backend.sock";
    let process_path = "target/debug/discover";

    let monitor_task = monitor_process(process_path.to_string(), socket_path.to_string());
    let server_task = run_axum_server();

    // Use tokio::select! to run both tasks concurrently
    tokio::select! {
        _ = monitor_task => {
            eprintln!("File monitoring task exited.");
        }
        _ = server_task => {
            eprintln!("Axum server exited.");
        }
    }

    Ok(())
}

async fn run_axum_server() {
    let app = Router::new().route("/api/plugin/:plugin_id", get(proxy_to_backend));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

// Monitor the file for changes and restart the backend process if it changes
async fn monitor_process(process_path: String, socket_path: String) {
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
            match restart_backend_process(&process_path, &socket_path).await {
                Ok(process) => {
                    current_process = Some(process); // Keep track of the new process
                }
                Err(err) => {
                    eprintln!("Error restarting backend process: {:?}", err);
                }
            }
        }

        // Sleep for a while before checking again
        sleep(Duration::from_secs(5)).await;
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
            Ok(process)
        }
        Err(err) => Err((
            hyper::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to restart backend: {:?}", err),
        )),
    }
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
