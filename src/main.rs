use std::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Port 0 is special-cased at the OS level: trying to bind port 0 will trigger an OS scan for an available port which
    // will then be bound to the application.
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");

    // To obtain the port, otherwise can't test
    // cargo r -- --show-output
    let port = listener.local_addr().expect("Failed to get address").port();
    println!("==============>PORT: {}", port);

    // Bubble up the io::Error if we failed to bind the address
    // Otherwise call .await on our Server
    zero_2_prod::run(listener)?.await
}
