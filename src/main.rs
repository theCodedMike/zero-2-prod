use std::fmt::{Debug, Display};
use tokio::task::JoinError;
use zero_2_prod::configuration;
use zero_2_prod::error::BizErrorEnum;
use zero_2_prod::issue_delivery_worker;
use zero_2_prod::startup::Application;
use zero_2_prod::telemetry;

#[tokio::main]
async fn main() -> Result<(), BizErrorEnum> {
    let subscriber =
        telemetry::get_subscriber("zero-2-prod".into(), "info".into(), std::io::stdout);
    telemetry::init_subscriber(subscriber)?;

    // Panic if we can't read configuration
    let config = configuration::get_configuration()?;
    /*
     * By running all async expressions on the current task,
     *  the expressions are able to run concurrently but not in parallel.
     * This means all expressions are run on the same thread and if one branch blocks the thread,
     *  all other expressions will be unable to continue.
     * If parallelism is required,
     *  spawn each async expression using tokio::spawn and pass the join handle to select!.
     */
    let application = Application::build(config.clone()).await?;
    let application_task = tokio::spawn(application.run_until_stopped());

    let worker = issue_delivery_worker::run_work_until_stopped(config);
    let worker_task = tokio::spawn(worker);

    tokio::select! {
        o = application_task => report_exit("API", o),
        o = worker_task => report_exit("Background worker", o),
    }

    Ok(())
}

fn report_exit(task_name: &str, outcome: Result<Result<(), impl Debug + Display>, JoinError>) {
    match outcome {
        Ok(inner) => match inner {
            Ok(_) => {
                tracing::info!("{} has exited", task_name)
            }
            Err(e) => {
                tracing::error!(error.cause_chain = ?e, error.message = %e, "{} failed", task_name)
            }
        },
        Err(e) => {
            tracing::error!(error.cause_chain = ?e, error.message = %e, "{} task failed to complete", task_name)
        }
    }
}
