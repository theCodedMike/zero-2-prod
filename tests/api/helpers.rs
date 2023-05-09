use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use std::time::Duration;
use uuid::Uuid;
use zero_2_prod::configuration;
use zero_2_prod::configuration::DatabaseSettings;
use zero_2_prod::email_client::EmailClient;
use zero_2_prod::startup;
use zero_2_prod::telemetry;

// Ensure that the `tracing` stack is only initialised once using `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    let mut writer = std::io::sink;
    // 这里先注释掉，会报类型不匹配
    /*if std::env::var("TEST_LOG").is_ok() {
        writer = std::io::stdout;
    }*/
    let subscriber = telemetry::get_subscriber(subscriber_name, default_filter_level, writer);
    telemetry::init_subscriber(subscriber);
});

pub struct TestApp {
    pub address: String,
    pub connect_pool: PgPool,
}

/// Spin up an instance of our application
/// and returns its address (i.e. http://localhost:XXXX)
/// Launch our application in the background ~somehow~
/// No .await call, therefore no need for `spawn_app` to be async now.
/// We are also running tests, so it is not worth it to propagate errors:
/// if we fail to perform the required setup we can just panic and crash
/// all the things.
pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut config = configuration::get_configuration().expect("Failed to read configuration");
    config.database.database_name = Uuid::new_v4().to_string();
    let pg_pool = configure_database(&config.database).await;

    // Build an `EmailClient` using `configuration`
    let sender_email = config
        .email_client
        .sender()
        .expect("Invalid sender email address");
    let email_client = EmailClient::new(
        config.email_client.base_url,
        sender_email,
        config.email_client.authorization_token,
        Duration::from_secs(5),
    );

    let server =
        startup::run(listener, pg_pool.clone(), email_client).expect("Failed to bind address");
    // Launch the server as a background task
    // tokio::spawn returns a handle to the spawned future,
    // but we have no use for it here, hence the non-binding let
    let _ = tokio::spawn(server);

    // We return the application address to the caller!
    TestApp {
        address,
        connect_pool: pg_pool,
    }
}

/// create a database and return a connection pool
async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database");
    // Migrate database
    let pg_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres");
    sqlx::migrate!("./migrations")
        .run(&pg_pool)
        .await
        .expect("Failed to migrate the database");
    pg_pool
}
