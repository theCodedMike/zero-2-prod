use crate::configuration::{DatabaseSettings, Settings};
use crate::email_client::EmailClient;
use crate::error::BizErrorEnum;
use crate::{auth, routes};
use actix_session::storage::RedisSessionStore;
use actix_session::SessionMiddleware;
use actix_web::cookie::Key;
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use actix_web_flash_messages::storage::CookieMessageStore;
use actix_web_flash_messages::FlashMessagesFramework;
use secrecy::{ExposeSecret, Secret};
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

        let server = run(
            listener,
            pg_pool,
            email_client,
            config.application.base_url,
            config.application.hmac_secret,
            config.redis_uri,
        )
        .await?;

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

async fn run(
    listener: TcpListener,
    pg_pool: PgPool,
    email_client: EmailClient,
    app_base_url: String,
    hmac_secret: Secret<String>,
    redis_uri: Secret<String>,
) -> Result<Server, BizErrorEnum> {
    // Wrap the connection in a smart pointer
    let connect_pool = web::Data::new(pg_pool);

    // Re-use the same HTTP client across multiple requests
    let email_client = web::Data::new(email_client);

    // Use at sending confirmation email
    let app_base_url = web::Data::new(ApplicationBaseUrl(app_base_url));

    // Flash message, CookieMessageStore enforces that the cookie used as storage is signed
    let secret_key = Key::from(hmac_secret.expose_secret().as_bytes());
    let message_store = CookieMessageStore::builder(secret_key.clone()).build();
    let message_framework = FlashMessagesFramework::builder(message_store).build();

    // Session
    let redis_store = RedisSessionStore::new(redis_uri.expose_secret())
        .await
        .map_err(|e| {
            tracing::error!("Failed to build a redis session store");
            BizErrorEnum::RedisSessionStoreBuildError(e)
        })?;

    // Capture `connection` from the surrounding environment
    let server = HttpServer::new(move || {
        App::new()
            // Middlewares are added using the `wrap` method on `App`
            .wrap(message_framework.clone())
            .wrap(TracingLogger::default())
            .wrap(SessionMiddleware::new(
                redis_store.clone(),
                secret_key.clone(),
            ))
            // Register the connection as part of the application state
            // Get a pointer copy and attach it to the application state
            .app_data(connect_pool.clone())
            .app_data(email_client.clone())
            .app_data(app_base_url.clone())
            .app_data(web::Data::new(HmacSecret(hmac_secret.clone())))
            .route("/", web::get().to(routes::home))
            .service(
                web::scope("/admin")
                    .wrap(actix_web_lab::middleware::from_fn(
                        auth::reject_anonymous_users,
                    ))
                    .route("/dashboard", web::get().to(routes::admin_dashboard))
                    .route("/password", web::get().to(routes::change_password_form))
                    .route("/password", web::post().to(routes::change_password))
                    .route(
                        "/newsletter",
                        web::get().to(routes::publish_newsletter_form),
                    )
                    .route("/newsletter", web::post().to(routes::publish_newsletter))
                    .route("/logout", web::post().to(routes::log_out)),
            )
            .route("/login", web::get().to(routes::login_form))
            .route("/login", web::post().to(routes::login))
            .route("/health_check", web::get().to(routes::health_check))
            .route("/subscriptions", web::post().to(routes::subscribe))
            .route("/subscriptions/confirm", web::get().to(routes::confirm))
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

#[derive(Debug, Clone)]
pub struct HmacSecret(pub Secret<String>);
