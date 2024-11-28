use axum::{routing::get, Json, Router};
use plugin_shared::{Datasource, Manifest};
use serde_json::Value;

pub async fn create_app() -> Router {
    let app = Router::new().route("/meta/manifest", get(handler));

    return app;
}

async fn handler() -> Json<Manifest> {
    // request back to the host app
    let resp = reqwest::get("http://localhost:3000/api/plugins")
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();

    println!("fetching data from parent app:\n{resp:#?}");

    let manifest = Manifest {
        id: "discover".to_string(),
        version: "0.0.1".to_string(),
        exported_datasources: vec![Datasource {
            kind: "elasticsearch".to_string(),
        }],
    };

    Json(manifest)
}
