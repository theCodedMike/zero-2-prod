use crate::auth::Credentials;
use crate::auth::UserId;
use crate::domain::SubscriberEmail;
use crate::error::BizErrorEnum;
use crate::idempotency::{IdempotencyKey, NextAction};
use crate::request::NewsletterData;
use crate::{idempotency, telemetry, utils};
use actix_web::http::header::HeaderMap;
use actix_web::{web, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use base64::Engine;
use secrecy::Secret;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

#[tracing::instrument(
    name = "/admin/newsletter: Publish a newsletter issue",
    skip_all,
    fields(user_id=tracing::field::Empty)
)]
pub async fn publish_newsletter(
    body: web::Form<NewsletterData>,
    pool: web::Data<PgPool>,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, BizErrorEnum> {
    // Get user_id
    let user_id = *user_id.into_inner();
    telemetry::record_field("user_id", &user_id);
    let NewsletterData {
        title,
        text_content,
        html_content,
        idempotency_key,
    } = body.into_inner();
    let idempotency_key: IdempotencyKey = idempotency_key.try_into()?;

    // Return early if we have a saved response in the database
    let mut transaction =
        match idempotency::try_processing(&pool, &idempotency_key, user_id).await? {
            NextAction::StartProcessing(t) => t,
            NextAction::ReturnSavedResponse(saved_response) => {
                FlashMessage::info(
                    "The newsletter issue has been accepted - emails will go out shortly.",
                )
                .send();
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

    let issue_id =
        insert_newsletter_issue(&mut transaction, &title, &text_content, &html_content).await?;

    enqueue_delivery_tasks(&mut transaction, issue_id).await?;

    // Get all confirmed subscriber's email
    /*let confirmed_emails = get_confirmed_subscribers(&pool).await?;
    if confirmed_emails.is_empty() {
        FlashMessage::info("No confirmed subscribers!!!").send();
        return Ok(utils::redirect_to("/admin/newsletter"));
    }*/

    // Send email
    /*for subscriber in confirmed_emails {
        email_client
            .send_email(
                &subscriber,
                &body_data.title,
                &body_data.html_content,
                &body_data.text_content,
            )
            .await?;
    }*/

    // Make response
    let response = utils::redirect_to("/admin/newsletter");
    let http_response =
        idempotency::update_response(transaction, user_id, &idempotency_key, response).await?;
    FlashMessage::info("The newsletter issue has been accepted - emails will go out shortly.")
        .send();
    Ok(http_response)
}

#[tracing::instrument(name = "Query confirmed subscribers", skip(pool))]
#[deprecated(since = "1.0", note = "refactoring")]
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
#[deprecated(since = "1.0", note = "refactoring")]
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

#[tracing::instrument(name = "Insert newsletter issue", skip_all)]
async fn insert_newsletter_issue(
    transaction: &mut Transaction<'static, Postgres>,
    title: &str,
    text_content: &str,
    html_content: &str,
) -> Result<Uuid, BizErrorEnum> {
    let newsletter_issue_id = Uuid::new_v4();
    sqlx::query!(
        r#"
            INSERT INTO newsletter_issues (
                newsletter_issue_id,
                title,
                text_content,
                html_content,
                published_at
            ) 
            VALUES ($1, $2, $3, $4, now())
        "#,
        newsletter_issue_id,
        title,
        text_content,
        html_content
    )
    .execute(transaction)
    .await
    .map_err(|e| BizErrorEnum::InsertNewsletterIssuesError(e))?;

    Ok(newsletter_issue_id)
}

#[tracing::instrument(name = "Insert issue delivery queue", skip_all)]
async fn enqueue_delivery_tasks(
    transaction: &mut Transaction<'static, Postgres>,
    newsletter_issue_id: Uuid,
) -> Result<(), BizErrorEnum> {
    sqlx::query!(
        r#"
            INSERT INTO issue_delivery_queue (
                newsletter_issue_id, 
                subscriber_email
            ) 
            SELECT $1, email 
            FROM subscriptions 
            WHERE status = 'confirmed'
        "#,
        newsletter_issue_id
    )
    .execute(transaction)
    .await
    .map_err(|e| BizErrorEnum::InsertIssueDeliveryQueueError(e))?;

    Ok(())
}
