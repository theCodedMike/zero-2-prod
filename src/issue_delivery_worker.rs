use crate::configuration::Settings;
use crate::domain::SubscriberEmail;
use crate::email_client::EmailClient;
use crate::error::BizErrorEnum;
use crate::error::BizErrorEnum::QueryNewsletterIssuesError;
use crate::{startup, telemetry};
use sqlx::{PgPool, Postgres, Transaction};
use std::time::Duration;
use uuid::Uuid;

type PgTransaction = Transaction<'static, Postgres>;

#[tracing::instrument(name = "Run work", skip_all)]
pub async fn run_work_until_stopped(config: Settings) -> Result<(), BizErrorEnum> {
    let connection_pool = startup::get_connection_pool(&config.database);

    let email_client = config.email_client.client();

    worker_loop(connection_pool, email_client).await
}

#[tracing::instrument(name = "Worker loop", skip_all)]
async fn worker_loop(pool: PgPool, email_client: EmailClient) -> Result<(), BizErrorEnum> {
    loop {
        match try_execute_task(&pool, &email_client).await {
            Ok(ExecutionOutcome::EmptyQueue) => {
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
            Ok(ExecutionOutcome::TaskCompleted) => {}
            Err(_) => {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

pub enum ExecutionOutcome {
    TaskCompleted,
    EmptyQueue,
}

#[tracing::instrument(
    name = "Execute task in queue",
    skip_all,
    fields(
        newsletter_issue_id=tracing::field::Empty,
        subscriber_email=tracing::field::Empty
    ),
    err
)]
pub async fn try_execute_task(
    pool: &PgPool,
    email_client: &EmailClient,
) -> Result<ExecutionOutcome, BizErrorEnum> {
    // Query table: issue_delivery_queue
    let task = dequeue_task(pool).await?;
    if task.is_none() {
        return Ok(ExecutionOutcome::EmptyQueue);
    }

    let (transaction, issue_id, email) = task.unwrap();
    telemetry::record_field("newsletter_issue_id", issue_id);
    telemetry::record_field("subscriber_email", &email);
    // Send email
    match SubscriberEmail::parse(email.clone()) {
        Ok(email) => {
            let issue = get_issue(pool, issue_id).await?;
            if let Err(e) = email_client
                .send_email(
                    &email,
                    &issue.title,
                    &issue.html_content,
                    &issue.text_content,
                )
                .await
            {
                tracing::error!(
                    error.cause_chain = ?e,
                    error.message = %e,
                    "Failed to deliver issue to a confirmed subscriber. Skipping."
                );
            }
        }
        Err(e) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "Skipping a confirmed subscriber. Their stored contact details are invalid."
            );
        }
    }
    // Delete task
    delete_task(transaction, issue_id, &email).await?;

    Ok(ExecutionOutcome::TaskCompleted)
}

#[tracing::instrument(name = "Dequeue task", skip_all)]
async fn dequeue_task(
    pool: &PgPool,
) -> Result<Option<(PgTransaction, Uuid, String)>, BizErrorEnum> {
    let mut transaction = pool
        .begin()
        .await
        .map_err(|e| BizErrorEnum::PgPoolError(e))?;

    let record = sqlx::query!(
        r#"
            SELECT newsletter_issue_id, subscriber_email 
            FROM issue_delivery_queue
            FOR UPDATE
            SKIP LOCKED
            LIMIT 1
        "#
    )
    .fetch_optional(&mut transaction)
    .await
    .map_err(|e| BizErrorEnum::QueryIssueDeliveryQueueError(e))?;

    match record {
        None => Ok(None),
        Some(r) => Ok(Some((
            transaction,
            r.newsletter_issue_id,
            r.subscriber_email,
        ))),
    }
}

#[tracing::instrument(name = "Delete task", skip_all)]
async fn delete_task(
    mut transaction: PgTransaction,
    issue_id: Uuid,
    email: &str,
) -> Result<(), BizErrorEnum> {
    sqlx::query!(
        r#"
            DELETE FROM issue_delivery_queue 
            WHERE 
                newsletter_issue_id = $1 AND 
                subscriber_email = $2
        "#,
        issue_id,
        email
    )
    .execute(&mut transaction)
    .await
    .map_err(|e| BizErrorEnum::DeleteIssueDeliveryQueueError(e))?;

    transaction
        .commit()
        .await
        .map_err(|e| BizErrorEnum::TransactionCommitError(e))?;

    Ok(())
}
struct NewsletterIssue {
    title: String,
    text_content: String,
    html_content: String,
}

#[tracing::instrument(name = "Query newsletter issue", skip(pool))]
async fn get_issue(pool: &PgPool, issue_id: Uuid) -> Result<NewsletterIssue, BizErrorEnum> {
    let record = sqlx::query_as!(
        NewsletterIssue,
        r#"
            SELECT title, text_content, html_content 
            FROM newsletter_issues 
            WHERE newsletter_issue_id = $1
        "#,
        issue_id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| QueryNewsletterIssuesError(e))?;

    Ok(record)
}
