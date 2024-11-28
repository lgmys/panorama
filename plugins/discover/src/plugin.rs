use axum::{routing::get, Json, Router};
use plugin_shared::{Datasource, Manifest};

pub async fn create_app() -> Router {
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

    return app;
}

async fn handler() -> Json<String> {
    Json(format!("Hello, World!"))
}
