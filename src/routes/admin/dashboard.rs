use crate::error::BizErrorEnum;
use crate::session_state::TypedSession;
use crate::utils;
use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

#[tracing::instrument(name = "Get admin dashboard", skip(pool, session))]
pub async fn admin_dashboard(
    session: TypedSession,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, BizErrorEnum> {
    // Verify if the user is logged in
    let user_id = utils::validate_session(&session)?;

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
