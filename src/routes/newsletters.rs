use crate::domain::SubscriberEmail;
use crate::email_client::EmailClient;
use crate::error::BizErrorEnum;
use crate::request::BodyData;
use actix_web::{web, HttpResponse};
use sqlx::PgPool;

#[tracing::instrument(
    name = "Publish newsletter to all confirmed subscribers",
    skip(body, pool, email_client)
)]
pub async fn publish_newsletter(
    body: web::Json<BodyData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
) -> Result<HttpResponse, BizErrorEnum> {
    // get all confirmed subscriber's email
    let confirmed_emails = get_confirmed_subscribers(&pool).await?;
    if confirmed_emails.is_empty() {
        tracing::info!("No confirmed subscribers.");
        return Ok(HttpResponse::Ok().finish());
    }

    // send email
    let body_data = body.into_inner();
    for subscriber in confirmed_emails {
        email_client
            .send_email(
                &subscriber,
                &body_data.title,
                &body_data.content.html,
                &body_data.content.text,
            )
            .await?;
    }
    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(name = "Query confirmed subscribers", skip(pool))]
async fn get_confirmed_subscribers(
    pool: &PgPool,
    // We are returning a `Vec` of `Result`s in the happy case.
    // This allows the caller to bubble up errors due to network issues or other
    // transient failures using the `?` operator, while the compiler
    // forces them to handle the subtler mapping error.
    // See http://sled.rs/errors.html for a deep-dive about this technique.
) -> Result<Vec<SubscriberEmail>, BizErrorEnum> {
    // We only need `Row` to map the data coming out of this query.
    // Nesting its definition inside the function itself is a simple way
    // to clearly communicate this coupling (and to ensure it doesn't
    // get used elsewhere by mistake).
    struct Row {
        email: String,
    }

    let rows = sqlx::query_as!(
        Row,
        r#"SELECT email FROM subscriptions WHERE status = 'confirmed'"#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to query confirmed subscribers: {:?}", e);
        BizErrorEnum::QuerySubscriptionsError(e)
    })?;

    let results = rows
        .into_iter()
        .filter_map(|row| match SubscriberEmail::parse(row.email) {
            Ok(subscriber) => Some(subscriber),
            Err(error) => {
                tracing::error!(
                    "A confirmed subscriber is using an invalid email address: {}",
                    error
                );
                None
            }
        })
        .collect();

    Ok(results)
}
