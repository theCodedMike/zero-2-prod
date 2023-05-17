use crate::request::ErrorData;
use crate::startup::HmacSecret;
use actix_web::http::header::ContentType;
use actix_web::{web, HttpResponse};

#[tracing::instrument(name = "Get login page", skip(query, secret))]
pub async fn login_form(
    query: Option<web::Query<ErrorData>>,
    secret: web::Data<HmacSecret>,
) -> HttpResponse {
    // HMAC to verify integrity and provenance for our query parameters
    let error_msg = match query {
        None => "".into(),
        Some(query) => match query.into_inner().verify(&secret) {
            Ok(error) => {
                format!("<p><i>{}</i></p>", htmlescape::encode_minimal(&error))
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to verify query parameters using the HMAC tag: {}",
                    e
                );
                "".into()
            }
        },
    };
    let login_page = include_str!("login.html").replace("{}", &error_msg);

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(login_page)
}
