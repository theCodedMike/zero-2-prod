use crate::error::BizErrorEnum;
use tokio::task::JoinHandle;
use tracing::{subscriber, Subscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

/// Compose multiple layers into a `tracing`'s subscriber.
///
/// # Implementation Notes
///
/// We are using `impl Subscriber` as return type to avoid having to
/// spell out the actual type of the returned subscriber, which is
/// indeed quite complex.
/// We need to explicitly call out that the returned subscriber is
/// `Send` and `Sync` to make it possible to pass it to `init_subscriber`
/// later on.
pub fn get_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink,
) -> impl Subscriber + Send + Sync
where
    // This "weird" syntax is a higher-ranked trait bound (HRTB)
    // It basically means that Sink implements the `MakeWriter`
    // trait for all choices of the lifetime parameter `'a`
    // Check out https://doc.rust-lang.org/nomicon/hrtb.html
    // for more details.
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    // We removed the `env_logger` line we had before!
    // We are falling back to printing all spans at info-level or above
    // if the RUST_LOG environment variable has not been set.
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let formatting_layer = BunyanFormattingLayer::new(name, sink);
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

/// Register a subscriber as global default to process span data.
///
/// It should only be called once!
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) -> Result<(), BizErrorEnum> {
    // Redirect all `log`'s events to our subscriber
    LogTracer::init().map_err(|e| {
        tracing::error!("Failed to set logger");
        BizErrorEnum::SetLoggerError(e)
    })?;
    // `set_global_default` can be used by applications to specify
    // what subscriber should be used to process spans.
    subscriber::set_global_default(subscriber).map_err(|e| {
        tracing::error!("Failed to set subscriber");
        BizErrorEnum::SetSubscriberError(e)
    })?;
    Ok(())
}

pub fn spawn_blocking_with_tracing<F, R>(f: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let current_span = tracing::Span::current();
    tokio::task::spawn_blocking(move || current_span.in_scope(f))
}
