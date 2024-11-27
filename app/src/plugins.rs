use std::time::{Duration, SystemTime};

use axum::extract::Request;
use hyper::Method;
use tokio::{
    fs,
    process::{Child, Command},
    time::sleep,
};

use crate::{
    ipc::plugin_request,
    types::{LoadedPluginsRegistry, Manifest},
};

pub async fn watch_plugin(
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
pub async fn get_last_modified(path: &str) -> Option<SystemTime> {
    match fs::metadata(path).await {
        Ok(metadata) => metadata.modified().ok(),
        Err(err) => {
            eprintln!("Failed to get metadata for {}: {:?}", path, err);
            None
        }
    }
}

// Helper function to restart the backend process
pub async fn restart_backend_process(
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

            let req = Request::builder()
                .uri("/manifest")
                .method("GET")
                .body("".to_string())
                .unwrap();

            let manifest = plugin_request(socket_path, req).await.unwrap();

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
