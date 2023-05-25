use crate::auth::Credentials;
use crate::auth::UserId;
use crate::domain::SubscriberEmail;
use crate::email_client::EmailClient;
use crate::error::BizErrorEnum;
use crate::idempotency::{IdempotencyKey, NextAction};
use crate::request::NewsletterData;
use crate::{idempotency, telemetry, utils};
use actix_web::http::header::HeaderMap;
use actix_web::{web, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use base64::Engine;
use secrecy::Secret;
use sqlx::PgPool;

#[tracing::instrument(
    name = "/admin/newsletter: Publish a newsletter issue",
    skip(body, pool, email_client, user_id),
    fields(user_id=tracing::field::Empty)
)]
pub async fn publish_newsletter(
    body: web::Form<NewsletterData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, BizErrorEnum> {
    // Get user_id
    let user_id = *user_id.into_inner();
    telemetry::record_field("user_id", &user_id);
    let body_data = body.into_inner();
    let idempotency_key: IdempotencyKey = body_data.idempotency_key.try_into()?;

    // Return early if we have a saved response in the database
    let transaction = match idempotency::try_processing(&pool, &idempotency_key, user_id).await? {
        NextAction::StartProcessing(t) => t,
        NextAction::ReturnSavedResponse(saved_response) => {
            FlashMessage::info("The newsletter issue has been published!").send();
            return Ok(saved_response);
        }
    };

    // Validate newsletter's body 可以放在前端校验
    /*if body_data.is_title_blank() {
        FlashMessage::error("Newsletter's title is blank.").send();
        return Ok(utils::redirect_to("/admin/newsletter"));
    }
    if body_data.is_html_blank() {
        FlashMessage::error("Newsletter's html content is blank.").send();
        return Ok(utils::redirect_to("/admin/newsletter"));
    }
    if body_data.is_text_blank() {
        FlashMessage::error("Newsletter's text content is blank.").send();
        return Ok(utils::redirect_to("/admin/newsletter"));
    }*/

    // Get all confirmed subscriber's email
    let confirmed_emails = get_confirmed_subscribers(&pool).await?;
    if confirmed_emails.is_empty() {
        FlashMessage::info("No confirmed subscribers!!!").send();
        return Ok(utils::redirect_to("/admin/newsletter"));
    }

    // Send email
    for subscriber in confirmed_emails {
        email_client
            .send_email(
                &subscriber,
                &body_data.title,
                &body_data.html_content,
                &body_data.text_content,
            )
            .await?;
    }

    // Make response
    FlashMessage::info("The newsletter issue has been published!").send();
    let response = utils::redirect_to("/admin/newsletter");
    let http_response =
        idempotency::update_response(transaction, user_id, &idempotency_key, response).await?;
    Ok(http_response)
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
