use std::env;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the socket path from arguments
    let args: Vec<String> = env::args().collect();
    let socket_path = &args[1];

    // Connect to the Unix socket
    let mut socket = UnixStream::connect(socket_path).await?;

    println!("Connected to socket at {}", socket_path);

    loop {
        // Read data from the parent
        let mut buf = vec![0; 1024];
        let n = socket.read(&mut buf).await?;
        println!(
            "Received from parent: {}",
            String::from_utf8_lossy(&buf[..n])
        );

        // Send data back to the parent
        let response = "Hello from child";
        socket.write_all(response.as_bytes()).await?;
        println!("Sent to parent: {}", response);
    }
}
