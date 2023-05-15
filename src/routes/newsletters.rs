use crate::constant::{DELIMITER, SALT};
use crate::domain::SubscriberEmail;
use crate::email_client::EmailClient;
use crate::error::BizErrorEnum;
use crate::request::BodyData;
use actix_web::http::header::HeaderMap;
use actix_web::{web, HttpRequest, HttpResponse};
use base64::Engine;
use secrecy::{ExposeSecret, Secret};
use sha3::Digest;
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
    let uuid = validate_credentials(&credential, &pool).await?;
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

#[tracing::instrument(name = "Query users by username and password", skip(pool))]
async fn validate_credentials(
    credentials: &Credentials,
    pool: &PgPool,
) -> Result<uuid::Uuid, BizErrorEnum> {
    // generate password_hash
    let raw_password_and_salt = format!(
        "{}{}{}",
        credentials.password.expose_secret(),
        DELIMITER,
        SALT
    );
    let raw_password_hash = sha3::Sha3_256::digest(raw_password_and_salt.as_ref());
    // Uppercase hexadecimal encoding.
    let password_hash = format!("{:X}", raw_password_hash);

    // query user_id from table
    let user_id = sqlx::query!(
        r#"
        SELECT user_id FROM users WHERE username = $1 AND password_hash = $2
    "#,
        &credentials.username,
        password_hash
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to query users table: {:?}", e);
        BizErrorEnum::QueryUsersError(e)
    })?;

    // convert
    user_id
        .map(|row| row.user_id)
        .ok_or(BizErrorEnum::InvalidUsernameOrPassword)
}
