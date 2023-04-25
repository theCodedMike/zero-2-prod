use env_logger::Env;
use sqlx::PgPool;
use std::net::TcpListener;
use zero_2_prod::configuration;
use zero_2_prod::startup;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // `init` does call `set_logger`, so this is all we need to do.
    // We are falling back to printing all logs at info-level or above
    // if the RUST_LOG environment variable has not been set.
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    // Panic if we can't read configuration
    let config = configuration::get_configuration().expect("Failed to read configuration");
    // We have removed the hard-coded `8000` - it's now coming from our settings!
    let address = format!("127.0.0.1:{}", config.application_port);
    let listener = TcpListener::bind(address)?;

    let pg_pool = PgPool::connect(config.database.connection_string().as_str())
        .await
        .expect("Failed to connect to Postgres");

    // Bubble up the io::Error if we failed to bind the address
    // Otherwise call .await on our Server
    startup::run(listener, pg_pool)?.await
}
