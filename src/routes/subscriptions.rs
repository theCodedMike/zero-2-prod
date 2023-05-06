use crate::domain::{InvalidReason, NewSubscriber};
use crate::request::FormData;
use actix_web::{web, HttpResponse};
use chrono::Local;
use sqlx::{Error, PgPool};
use uuid::Uuid;

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    // `web::Form` is a wrapper around `FormData`
    // `form.0` gives us access to the underlying `FormData`
    let subscriber = match form.into_inner().try_into() {
        Ok(subscriber) => subscriber,
        Err(err) => {
            return HttpResponse::BadRequest()
                .reason(InvalidReason::as_str(&err))
                .finish()
        }
    };

    // insert
    match insert_subscriber(&pool, &subscriber).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(subscriber, pool)
)]
pub async fn insert_subscriber(pool: &PgPool, subscriber: &NewSubscriber) -> Result<(), Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
    "#,
        Uuid::new_v4(),
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
    Ok(())
}
