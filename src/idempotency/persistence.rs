use crate::error::BizErrorEnum;
use crate::idempotency::IdempotencyKey;
use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use sqlx::postgres::{PgHasArrayType, PgTypeInfo};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "header_pair")]
struct HeaderPairRecord {
    name: String,
    value: Vec<u8>,
}

impl PgHasArrayType for HeaderPairRecord {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_header_pair")
    }
}

#[tracing::instrument(
    name = "Query response from idempotency",
    skip(pool, idempotency_key, user_id)
)]
pub async fn get_saved_response(
    pool: &PgPool,
    idempotency_key: &IdempotencyKey,
    user_id: Uuid,
) -> Result<Option<HttpResponse>, BizErrorEnum> {
    // ask sqlx to forcefully assume that the columns will not be null - if we are wrong, it will
    // cause an error at runtime (!)
    let response = sqlx::query!(
        r#"
            SELECT 
                response_status_code as "response_status_code!", 
                response_headers as "response_headers!: Vec<HeaderPairRecord>", 
                response_body as "response_body!"
            FROM idempotency 
            WHERE user_id = $1 AND idempotency_key = $2
    "#,
        user_id,
        idempotency_key.as_ref()
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| BizErrorEnum::QueryIdempotencyError(e))?;

    match response {
        None => Ok(None),
        Some(r) => {
            let status_code = StatusCode::from_u16(
                r.response_status_code
                    .try_into()
                    .map_err(|e| BizErrorEnum::ResponseStatusCodeTryIntoError(e))?,
            )
            .map_err(|_e| BizErrorEnum::StatusCodeConvertError)?;

            let mut response = HttpResponse::build(status_code);
            for HeaderPairRecord { name, value } in r.response_headers {
                response.append_header((name, value));
            }

            Ok(Some(response.body(r.response_body)))
        }
    }
}

#[tracing::instrument(
    name = "Update response in idempotency",
    skip(transaction, user_id, idempotency_key, http_response)
)]
pub async fn update_response(
    // No longer a `Pool`!
    mut transaction: Transaction<'static, Postgres>,
    user_id: Uuid,
    idempotency_key: &IdempotencyKey,
    http_response: HttpResponse,
) -> Result<HttpResponse, BizErrorEnum> {
    // http_response.body()
    // the trait `MessageBody` is not implemented for `&BoxBody`
    let (response_head, body) = http_response.into_parts();
    // `MessageBody::Error` is not `Send` + `Sync`
    let body = actix_web::body::to_bytes(body)
        .await
        .map_err(|_e| BizErrorEnum::HttpResponseBodyToBytesError)?;
    let status_code = response_head.status().as_u16() as i16;
    let headers = {
        let mut headers = Vec::with_capacity(response_head.headers().len());
        for (name, value) in response_head.headers().iter() {
            let name = name.as_str().to_owned();
            let value = value.as_bytes().to_owned();
            headers.push(HeaderPairRecord { name, value })
        }
        headers
    };

    // We are using a custom type and sqlx::query! is not powerful enough to learn about
    // it at compile-time in order to check our query. Use query_unchecked!
    sqlx::query_unchecked!(
        r#"
            UPDATE idempotency 
            SET 
                response_status_code = $1, 
                response_headers = $2, 
                response_body = $3
            WHERE user_id = $4 AND idempotency_key = $5 
        "#,
        status_code,
        headers,
        body.as_ref(),
        user_id,
        idempotency_key.as_ref()
    )
    .execute(&mut transaction)
    .await
    .map_err(|e| BizErrorEnum::InsertIdempotencyError(e))?;

    // Commit transaction
    transaction
        .commit()
        .await
        .map_err(|e| BizErrorEnum::TransactionCommitError(e))?;

    let response = response_head.set_body(body).map_into_boxed_body();
    Ok(response)
}

#[allow(clippy::large_enum_variant)]
pub enum NextAction {
    StartProcessing(Transaction<'static, Postgres>),
    ReturnSavedResponse(HttpResponse),
}

#[tracing::instrument(
    name = "Try processing insert operation",
    skip(pool, idempotency_key, user_id)
)]
pub async fn try_processing(
    pool: &PgPool,
    idempotency_key: &IdempotencyKey,
    user_id: Uuid,
) -> Result<NextAction, BizErrorEnum> {
    let mut transaction = pool
        .begin()
        .await
        .map_err(|e| BizErrorEnum::PgPoolError(e))?;

    let rows_affected = sqlx::query!(
        r#"
            INSERT INTO idempotency(
                user_id, 
                idempotency_key, 
                created_at
            ) 
            VALUES ($1, $2, now()) 
            ON CONFLICT DO NOTHING
        "#,
        user_id,
        idempotency_key.as_ref()
    )
    .execute(&mut transaction)
    .await
    .map_err(|e| BizErrorEnum::InsertIdempotencyError(e))?
    .rows_affected();

    if rows_affected > 0 {
        // Return transaction for later usage
        Ok(NextAction::StartProcessing(transaction))
    } else {
        let saved_response = get_saved_response(pool, idempotency_key, user_id)
            .await?
            .ok_or(BizErrorEnum::FindExpectedSavedRecordError)?;
        Ok(NextAction::ReturnSavedResponse(saved_response))
    }
}
