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
    let username = if let Some(user_id) = session.get_user_id()? {
        query_username(user_id, &pool).await?
    } else {
        // If a user tries to navigate directly to /admin/dashboard and they are not logged in,
        // they will be redirected to the login form.
        return Ok(utils::redirect_to("/login"));
    };

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
