use crate::domain::SubscriberEmail;
use crate::email_client::EmailClient;
use crate::error::BizErrorEnum;
use crate::request::BodyData;
use crate::telemetry;
use actix_web::http::header::HeaderMap;
use actix_web::{web, HttpRequest, HttpResponse};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use base64::Engine;
use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;

#[tracing::instrument(
    name = "Publish a newsletter issue",
    skip(body, pool, email_client, request),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn publish_newsletter(
    body: web::Json<BodyData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    request: HttpRequest,
) -> Result<HttpResponse, BizErrorEnum> {
    // get credential from headers
    let credential = basic_authentication(request.headers())?;
    tracing::Span::current().record("username", &tracing::field::display(&credential.username));

    // validate username and password
    let uuid = validate_credentials(credential, &pool).await?;
    tracing::Span::current().record("user_id", &tracing::field::display(&uuid));

    // validate body
    let body_data = body.into_inner();
    if body_data.is_title_blank() {
        tracing::error!("Newsletter's title is empty.");
        return Err(BizErrorEnum::NewsletterTitleIsEmpty);
    }
    if body_data.is_content_blank() {
        tracing::error!("Newsletter's content is empty.");
        return Err(BizErrorEnum::NewsletterContentIsEmpty);
    }

    // get all confirmed subscriber's email
    let confirmed_emails = get_confirmed_subscribers(&pool).await?;
    if confirmed_emails.is_empty() {
        tracing::info!("No confirmed subscribers.");
        return Ok(HttpResponse::Ok().finish());
    }

    // send email
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

#[derive(Debug)]
struct Credentials {
    username: String,
    password: Secret<String>,
}
/// Password-based Authentication
///
/// Authorization: Basic {username}:{password}
#[tracing::instrument(name = "Get credentials from headers", skip(headers))]
fn basic_authentication(headers: &HeaderMap) -> Result<Credentials, BizErrorEnum> {
    let header_value = headers
        .get("Authorization")
        .ok_or(BizErrorEnum::AuthorizationHeaderIsMissing)?
        .to_str()
        .map_err(|e| BizErrorEnum::AuthorizationHeaderIsInvalidUtf8String(e))?;
    let base64_encoded_segment = header_value
        .strip_prefix("Basic ")
        .ok_or(BizErrorEnum::AuthorizationSchemeNotBasic)?;

    let decoded_bytes = base64::engine::general_purpose::STANDARD
        .decode(base64_encoded_segment)
        .map_err(|e| BizErrorEnum::Base64DecodeError(e))?;

    let decoded_credentials = String::from_utf8(decoded_bytes)
        .map_err(|e| BizErrorEnum::CredentialStringIsInvalidUtf8String(e))?;

    // Split into two segments, using ':' as delimiter
    let mut credentials = decoded_credentials.splitn(2, ':');

    let username = credentials
        .next()
        .filter(|str| !str.is_empty())
        .ok_or(BizErrorEnum::CredentialMissingUsername)?;
    let password = credentials
        .next()
        .filter(|str| !str.is_empty())
        .ok_or(BizErrorEnum::CredentialMissingPassword)?;

    Ok(Credentials {
        username: username.into(),
        password: Secret::new(password.into()),
    })
}

#[tracing::instrument(name = "Validate credentials", skip(credentials, pool))]
async fn validate_credentials(
    credentials: Credentials,
    pool: &PgPool,
) -> Result<uuid::Uuid, BizErrorEnum> {
    // query user_id, password_hash from table
    let (user_id, password_hash_from_db) = get_stored_credentials(&credentials.username, pool)
        .await?
        .ok_or(BizErrorEnum::InvalidUsername)?;

    // PHC string format takes care of salt for us, implicitly
    // Offload CPU-intensive task to a separate thread-pool using tokio::task::spawn_blocking.
    telemetry::spawn_blocking_with_tracing(move || {
        verify_password_hash(password_hash_from_db, credentials.password)
    })
    .await
    // spawn_blocking is fallible - we have a nested Result here!
    .map_err(|e| BizErrorEnum::SpawnBlockingTaskError(e))??;

    Ok(user_id)
}

#[tracing::instrument(name = "Get stored credentials", skip(pool))]
async fn get_stored_credentials(
    username: &str,
    pool: &PgPool,
) -> Result<Option<(uuid::Uuid, Secret<String>)>, BizErrorEnum> {
    let row = sqlx::query!(
        r#"
        SELECT user_id, password_hash FROM users WHERE username = $1
    "#,
        username
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to query users table: {:?}", e);
        BizErrorEnum::QueryUsersError(e)
    })?
    .map(|row| (row.user_id, Secret::new(row.password_hash)));

    Ok(row)
}

#[tracing::instrument(
    name = "Verify password hash",
    skip(password_hash_from_db, password_from_user)
)]
fn verify_password_hash(
    password_hash_from_db: Secret<String>,
    password_from_user: Secret<String>,
) -> Result<(), BizErrorEnum> {
    // parse hash in PHC string format
    let expected_password_hash = PasswordHash::new(&password_hash_from_db.expose_secret())
        .map_err(|e| BizErrorEnum::Argon2HashParseError(e))?;
    Argon2::default()
        .verify_password(
            password_from_user.expose_secret().as_bytes(),
            &expected_password_hash,
        )
        .map_err(|e| BizErrorEnum::InvalidPassword(e))?;
    Ok(())
}
