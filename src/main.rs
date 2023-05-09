use zero_2_prod::configuration;
use zero_2_prod::startup::Application;
use zero_2_prod::telemetry;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber =
        telemetry::get_subscriber("zero-2-prod".into(), "info".into(), std::io::stdout);
    telemetry::init_subscriber(subscriber);

    // Panic if we can't read configuration
    let config = configuration::get_configuration().expect("Failed to read configuration");
    let application = Application::build(config).await?;
    application.run_until_stopped().await?;

    Ok(())
}
