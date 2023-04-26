use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use sqlx::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

use crate::routes;

/// We need to mark `run` as public.
/// It is no longer a binary entrypoint, therefore we can mark it as async
/// without having to use any proc-macro incantation.
pub fn run(listener: TcpListener, pg_pool: PgPool) -> Result<Server, std::io::Error> {
    // Wrap the connection in a smart pointer
    let connect_pool = web::Data::new(pg_pool);
    // Capture `connection` from the surrounding environment
    let server = HttpServer::new(move || {
        App::new()
            // Middlewares are added using the `wrap` method on `App`
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(routes::health_check))
            .route("/subscriptions", web::post().to(routes::subscribe))
            // Register the connection as part of the application state
            // Get a pointer copy and attach it to the application state
            .app_data(connect_pool.clone())
    })
    .listen(listener)?
    .run();
    // No .await here!
    Ok(server)
}
