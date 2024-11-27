use futures::future;
use http::run_axum_server;
use plugins::watch_plugin;
use std::{collections::HashMap, sync::Arc};
use tokio::{fs, sync::Mutex, task};
use types::{Manifest, PanoramaConfig};

mod http;
mod ipc;
mod plugins;
mod types;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = "./panorama.toml";
    let file = fs::read_to_string(&config_path).await.unwrap();
    let config: PanoramaConfig = toml::from_str(&file).unwrap();
    let loaded_plugins = Arc::new(Mutex::new(HashMap::<String, Manifest>::new()));

    let mut plugin_tasks: Vec<task::JoinHandle<_>> = vec![];

    let values = config.plugins.clone().into_values();

    for plugin_config in values {
        let loaded_plugins = loaded_plugins.clone();

        let task = task::spawn(async move {
            watch_plugin(
                loaded_plugins,
                plugin_config.binary_path.clone(),
                plugin_config.socket_path.clone(),
            )
            .await;
        });
        plugin_tasks.push(task);
    }

    let server_task = run_axum_server(Arc::new(config.clone()), loaded_plugins.clone());

    tokio::select! {
        _ = future::select_all(plugin_tasks) => {
            eprintln!("Plugin monitoring task exited.");
        }
        _ = server_task => {
            eprintln!("Http server exited.");
        }
    }

    Ok(())
}
