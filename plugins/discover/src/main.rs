use std::env;

use plugin::create_app;
use plugin_shared::bootstrap_plugin;

mod plugin;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let socket_path = &args[1];

    println!("starting Discover plugin on unix socket: {}", &socket_path);

    let app = create_app().await;
    bootstrap_plugin(socket_path, app).await
}
