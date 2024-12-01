use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

pub type LoadedPluginsRegistry = Arc<Mutex<HashMap<String, Manifest>>>;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Plugin {
    pub binary_path: String,
    pub socket_path: String,
    pub secrets: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Clone, Serialize, Debug)]
pub struct PanoramaConfig {
    pub plugins: HashMap<String, Plugin>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Manifest {
    pub id: String,
}

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<PanoramaConfig>,
    pub loaded_plugins: Arc<Mutex<HashMap<String, Manifest>>>,
}
