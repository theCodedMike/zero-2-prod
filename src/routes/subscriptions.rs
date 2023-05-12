use crate::domain::NewSubscriber;
use crate::email_client::EmailClient;
use crate::error::{StoreTokenError, SubscribeError};
use crate::request::FormData;
use crate::startup::ApplicationBaseUrl;
use actix_web::{web, HttpResponse};
use chrono::Local;
use rand::distributions::Alphanumeric;
use rand::Rng;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool, email_client),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    app_base_url: web::Data<ApplicationBaseUrl>,
) -> Result<HttpResponse, SubscribeError> {
    // `web::Form` is a wrapper around `FormData`
    // `form.0` gives us access to the underlying `FormData`
    let subscriber = form
        .into_inner()
        .try_into()
        .map_err(SubscribeError::ValidationError)?;

    // get a transaction object
    let mut transaction = pool.begin().await.map_err(SubscribeError::PoolError)?;

    // insert subscriptions table
    let subscriber_id = insert_subscriber(&mut transaction, &subscriber)
        .await
        .map_err(SubscribeError::InsertSubscriberError)?;
    let subscription_token = generate_subscription_token();

    // insert subscription_tokens table
    store_token(&mut transaction, subscriber_id, &subscription_token).await?;

    // explicitly commit
    transaction
        .commit()
        .await
        .map_err(SubscribeError::TransactionCommitError)?;

    // send confirmation email
    send_confirmation_email(
        &email_client,
        &subscriber,
        &app_base_url,
        &subscription_token,
    )
    .await?;

    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(subscriber, pool)
)]
async fn insert_subscriber(
    pool: &mut Transaction<'_, Postgres>,
    subscriber: &NewSubscriber,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'pending_confirmation')
    "#,
        subscriber_id,
        subscriber.email(),
        subscriber.name(),
        Local::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        // 注意: 这里使用了:?，不使用也可以，使用:?可以以debug的格式展示错误信息，也更具体
        tracing::error!("Failed to execute query: {:?}", e);
        e
        // Using the `?` operator to return early
        // if the function failed, returning a sqlx::Error
        // We will talk about error handling in depth later!
    })?;
    Ok(subscriber_id)
}

#[tracing::instrument(name = "Store subscriber id and token in the database", skip(pool))]
async fn store_token(
    pool: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
    subscription_token: &str,
) -> Result<(), StoreTokenError> {
    sqlx::query!(
        r#"
        INSERT INTO subscription_tokens (subscription_token, subscriber_id)
        VALUES ($1, $2)
    "#,
        subscription_token,
        subscriber_id
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}

#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(email_client, new_subscriber)
)]
async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: &NewSubscriber,
    app_base_url: &ApplicationBaseUrl,
    subscription_token: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        app_base_url.0.as_str(),
        subscription_token
    );
    let html_body = format!(
        "Welcome to our newsletter!<br />\
                Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
    );
    let plain_body = format!(
        "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
        confirmation_link
    );
    email_client
        .send_email(
            new_subscriber.get_email(),
            "Welcome!",
            &html_body,
            &plain_body,
        )
        .await
}

/// Generate a random 25-characters-long case-sensitive subscription token.
fn generate_subscription_token() -> String {
    let mut rng = rand::thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}
