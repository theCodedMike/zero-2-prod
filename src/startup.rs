use crate::configuration::{DatabaseSettings, Settings};
use crate::email_client::EmailClient;
use crate::error::BizErrorEnum;
use crate::routes;
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::net::TcpListener;
use std::time::Duration;
use tracing_actix_web::TracingLogger;

// A new type to hold the newly built server and its port
pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    // We have converted the `build` function into a constructor for `Application`.
    pub async fn build(config: Settings) -> Result<Self, BizErrorEnum> {
        let pg_pool = get_connection_pool(&config.database);

        // Build an `EmailClient` using `configuration`
        let sender_email = config.email_client.sender()?;
        let timeout = config.email_client.timeout();
        let email_client = EmailClient::new(
            config.email_client.base_url,
            sender_email,
            config.email_client.authorization_token,
            timeout,
        );

        // We have removed the hard-coded `8000` - it's now coming from our settings!
        // 0.0.0.0 as host to instruct our application to accept connections from any network interface,
        // not just the local one.
        let address = format!("{}:{}", config.application.host, config.application.port);
        let listener = TcpListener::bind(address).map_err(|e| {
            tracing::error!("Failed to bind to TcpListener");
            BizErrorEnum::BindTcpListenerError(e)
        })?;
        let port = listener.local_addr().unwrap().port();

        let server = run(listener, pg_pool, email_client, config.application.base_url)?;

        // We "save" the bound port in one of `Application`'s fields
        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    // A more expressive name that makes it clear that
    // this function only returns when the application is stopped.
    pub async fn run_until_stopped(self) -> Result<(), BizErrorEnum> {
        self.server.await.map_err(|e| {
            tracing::error!("Failed to run server.");
            BizErrorEnum::RunServerError(e)
        })
    }
}

// We need to define a wrapper type in order to retrieve the URL
// in the `subscribe` handler.
// Retrieval from the context, in actix-web, is type-based: using
// a raw `String` would expose us to conflicts.
#[derive(Debug)]
pub struct ApplicationBaseUrl(pub String);

fn run(
    listener: TcpListener,
    pg_pool: PgPool,
    email_client: EmailClient,
    app_base_url: String,
) -> Result<Server, BizErrorEnum> {
    // Wrap the connection in a smart pointer
    let connect_pool = web::Data::new(pg_pool);

    // Re-use the same HTTP client across multiple requests
    let email_client = web::Data::new(email_client);

    // Use at sending confirmation email
    let app_base_url = web::Data::new(ApplicationBaseUrl(app_base_url));

    // Capture `connection` from the surrounding environment
    let server = HttpServer::new(move || {
        App::new()
            // Middlewares are added using the `wrap` method on `App`
            .wrap(TracingLogger::default())
            // Register the connection as part of the application state
            // Get a pointer copy and attach it to the application state
            .app_data(connect_pool.clone())
            .app_data(email_client.clone())
            .app_data(app_base_url.clone())
            .route("/health_check", web::get().to(routes::health_check))
            .route("/subscriptions", web::post().to(routes::subscribe))
            .route("/subscriptions/confirm", web::get().to(routes::confirm))
            .route("/newsletters", web::post().to(routes::publish_newsletter))
    })
    .listen(listener)
    .map_err(|e| {
        tracing::error!("Failed to listen to TcpListener");
        BizErrorEnum::ListenTcpListenerError(e)
    })?
    .run();

    // No .await here!
    Ok(server)
}

pub fn get_connection_pool(config: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(5))
        .connect_lazy_with(config.with_db())
}
