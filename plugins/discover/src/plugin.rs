use core::panic;
use std::sync::Arc;

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use elasticsearch::{cat::CatIndicesParts, http::transport::Transport, Elasticsearch, SearchParts};
use plugin_shared::{Datasource, Manifest};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::sync::Mutex;

#[derive(Deserialize, Debug)]
pub struct DiscoverSecrets {
    pub connection_string: String,
}

#[derive(Deserialize, Debug)]
pub struct DiscoverConfig {
    pub secrets: DiscoverSecrets,
}

#[derive(Clone)]
pub struct PluginState {
    pub es_client: Arc<Mutex<Elasticsearch>>,
}

pub async fn create_app() -> Router {
    // request back to the host app
    let config = reqwest::get("http://localhost:3000/api/config")
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();

    let config = config
        .get("plugins")
        .unwrap()
        .get("discover")
        .unwrap()
        .clone();

    let config: DiscoverConfig = serde_json::from_value(config).unwrap();

    println!("plugin config:\n{config:#?}");

    let transport = Transport::single_node(&config.secrets.connection_string).unwrap();

    let client = Elasticsearch::new(transport);

    let response = client
        .cat()
        .indices(CatIndicesParts::Index(&["*"]))
        .format("json")
        .send()
        .await
        .unwrap();

    if response.status_code() != 200 {
        panic!("could not connect to elasticsearch");
    }

    let app = Router::new()
        .route("/meta/manifest", get(manifest_handler))
        .route(
            "/datasource/elasticsearch/query",
            post(datasource_handler_query),
        )
        .with_state(PluginState {
            es_client: Arc::new(Mutex::new(client)),
        });

    return app;
}

async fn manifest_handler() -> Json<Manifest> {
    let manifest = Manifest {
        id: "discover".to_string(),
        version: "0.0.1".to_string(),
        exported_datasources: vec![Datasource {
            kind: "elasticsearch".to_string(),
        }],
    };

    Json(manifest)
}

// NOTE: this would be wrapped with {datasource: { id: string, kind: string }, query: BELOW_TYPE}

#[derive(Deserialize)]
pub struct QueryParams {
    pub collection: String,
    pub filter: Option<String>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

async fn datasource_handler_query(
    State(plugin_state): State<PluginState>,
    Json(params): Json<QueryParams>,
) -> Json<Value> {
    let client = plugin_state.es_client.lock().await;

    let response = client
        .search(SearchParts::Index(&[&params.collection]))
        .from(0)
        .size(10)
        .body(json!({
            "query": {
                "match": {
                    "message": "Elasticsearch rust"
                }
            }
        }))
        .send()
        .await
        .unwrap();

    let response_body = response.json::<Value>().await.unwrap();

    Json(response_body)
}
