use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use std::time::Duration;
use zero_2_prod::configuration;
use zero_2_prod::startup;
use zero_2_prod::telemetry;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber =
        telemetry::get_subscriber("zero-2-prod".into(), "info".into(), std::io::stdout);
    telemetry::init_subscriber(subscriber);

    // Panic if we can't read configuration
    let config = configuration::get_configuration().expect("Failed to read configuration");
    // We have removed the hard-coded `8000` - it's now coming from our settings!
    // 0.0.0.0 as host to instruct our application to accept connections from any network interface,
    // not just the local one.
    let address = format!("{}:{}", config.application.host, config.application.port);
    println!("app url: {}", address);
    let listener = TcpListener::bind(address)?;

    let pg_pool = PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(5))
        .connect_lazy_with(config.database.with_db());
    //.expect("Failed to create Postgres connection pool");

    // Bubble up the io::Error if we failed to bind the address
    // Otherwise call .await on our Server
    startup::run(listener, pg_pool)?.await?;
    Ok(())
}
