use std::sync::Arc;

use axum::{
    extract::{Path, Request, State},
    routing::{any, get},
    Json, Router,
};
use hyper::StatusCode;
use serde_json::Value;

use crate::{
    ipc::plugin_request,
    types::{AppState, LoadedPluginsRegistry, PanoramaConfig},
};

pub async fn start_http_server(config: Arc<PanoramaConfig>, loaded_plugins: LoadedPluginsRegistry) {
    let app = Router::new()
        .route("/api/config", get(get_config))
        .route("/api/plugins", get(get_plugin_status))
        .route("/api/plugin/:plugin_id/*rest", any(proxy_to_plugin))
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

pub async fn get_config(State(state): State<AppState>) -> Json<PanoramaConfig> {
    let config = (*state.config).clone();

    Json(config)
}

pub async fn proxy_to_plugin(
    State(state): State<AppState>,
    Path((plugin_id, rest)): Path<(String, String)>,
    mut request: Request,
) -> Result<(StatusCode, Json<Value>), (hyper::StatusCode, String)> {
    let plugin_config = state.config.plugins.get(&plugin_id).unwrap();

    let query = &request.uri().query().unwrap_or("");
    let target_path = format!("/{}?{}", rest, query);

    // NOTE: rewrite the path a bit
    *request.uri_mut() = target_path.parse().unwrap();

    let res = plugin_request(&plugin_config.socket_path, request).await;

    match res {
        Ok((status, response_string)) => {
            if status.as_u16() >= 400 {
                return Err((status, response_string));
            }

            return Ok((
                status,
                Json(serde_json::from_str(&response_string).unwrap()),
            ));
        }
        Err(_) => Err((
            hyper::StatusCode::INTERNAL_SERVER_ERROR,
            "error".to_string(),
        )),
    }
}
