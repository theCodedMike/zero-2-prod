use crate::error::BizErrorEnum;
use crate::request::ConfirmData;
use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

#[tracing::instrument(
    name = "/subscriptions/confirm: Confirm a pending subscriber",
    skip(confirm, pool)
)]
pub async fn confirm(
    confirm: web::Query<ConfirmData>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, BizErrorEnum> {
    let token = confirm.into_inner().subscription_token;

    // query subscriber_id from subscription_tokens table
    let subscriber_id = get_subscriber_id_from_token(&pool, &token).await?;

    // update subscriptions table
    match subscriber_id {
        None => {
            tracing::error!(
                "Failed to query subscriber_id from subscription_tokens: token = {}",
                &token
            );
            Err(BizErrorEnum::SubscriptionTokenInvalidError)
        }
        Some(subscriber_id) => {
            confirm_subscriber(&pool, subscriber_id).await?;
            Ok(HttpResponse::Ok().finish())
        }
    }
}

#[tracing::instrument(
    name = "Get subscriber_id from subscription_tokens by token",
    skip(pool)
)]
async fn get_subscriber_id_from_token(
    pool: &PgPool,
    token: &str,
) -> Result<Option<Uuid>, BizErrorEnum> {
    let result = sqlx::query!(
        "SELECT subscriber_id FROM subscription_tokens WHERE subscription_token = $1",
        token
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to query subscription_tokens: {:?}", e);
        BizErrorEnum::QuerySubscriptionTokensError(e)
    })?;
    Ok(result.map(|r| r.subscriber_id))
}

#[tracing::instrument(name = "Update status of subscriptions by subscriber_id", skip(pool))]
async fn confirm_subscriber(pool: &PgPool, id: Uuid) -> Result<(), BizErrorEnum> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        id
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update subscriptions: {:?}", e);
        BizErrorEnum::UpdateSubscriptionsError(e)
    })?;
    Ok(())
}
