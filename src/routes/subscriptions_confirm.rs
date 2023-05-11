use crate::request::ConfirmData;
use actix_web::{web, HttpResponse};
use sqlx::{Error, PgPool};
use uuid::Uuid;

#[tracing::instrument(name = "Confirm a pending subscriber", skip(confirm, pool))]
pub async fn confirm(confirm: web::Query<ConfirmData>, pool: web::Data<PgPool>) -> HttpResponse {
    // query subscription_tokens table
    let subscriber_id =
        match get_subscriber_id_from_token(&pool, &confirm.into_inner().subscription_token).await {
            Ok(id) => id,
            Err(_) => return HttpResponse::InternalServerError().finish(),
        };

    // update subscriptions table
    match subscriber_id {
        None => HttpResponse::Unauthorized().finish(),
        Some(subscriber_id) => {
            if confirm_subscriber(&pool, subscriber_id).await.is_err() {
                return HttpResponse::InternalServerError().finish();
            }
            HttpResponse::Ok().finish()
        }
    }
}

#[tracing::instrument(
    name = "Get subscriber_id from subscription_tokens by token",
    skip(pool)
)]
async fn get_subscriber_id_from_token(pool: &PgPool, token: &str) -> Result<Option<Uuid>, Error> {
    let result = sqlx::query!(
        "SELECT subscriber_id FROM subscription_tokens WHERE subscription_token = $1",
        token
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(result.map(|r| r.subscriber_id))
}

#[tracing::instrument(
    name = "Update status of subscriptions by subscriber_id",
    skip(pool)
)]
async fn confirm_subscriber(pool: &PgPool, id: Uuid) -> Result<(), Error> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        id
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}
