use std::sync::Arc;

use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use serde_json::Value;

use crate::{
    ipc::fetch_data_from_plugin,
    types::{AppState, LoadedPluginsRegistry, PanoramaConfig},
};

pub async fn run_axum_server(config: Arc<PanoramaConfig>, loaded_plugins: LoadedPluginsRegistry) {
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

pub async fn get_plugin_status(State(state): State<AppState>) -> Json<Value> {
    let plugins = state.loaded_plugins.lock().await.clone();
    let plugins = serde_json::to_value(plugins).unwrap();

    Json(plugins)
}

// Proxy HTTP requests to the backend process
pub async fn proxy_to_backend(
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
