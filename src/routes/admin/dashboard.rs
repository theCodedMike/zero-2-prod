use crate::auth::UserId;
use crate::error::BizErrorEnum;
use crate::utils;
use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

#[tracing::instrument(name = "Get admin dashboard", skip(pool, user_id))]
pub async fn admin_dashboard(
    user_id: web::ReqData<UserId>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, BizErrorEnum> {
    // Verify if the user is logged in
    let user_id = *user_id.into_inner();

    // Get username
    let username = query_username(user_id, &pool).await?;

    let body = include_str!("dashboard.html").replace("{}", &username);
    Ok(utils::ok_to(body))
}

#[tracing::instrument(name = "Query username", skip(pool))]
pub async fn query_username(user_id: Uuid, pool: &PgPool) -> Result<String, BizErrorEnum> {
    let record = sqlx::query!(
        r#"
            SELECT username FROM users WHERE user_id = $1
    "#,
        user_id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| BizErrorEnum::QueryUsersError(e))?;

    Ok(record.username)
}
