use crate::utils;
use actix_web::HttpResponse;

#[tracing::instrument(name = "/: Homepage")]
pub async fn home() -> HttpResponse {
    let body = include_str!("home.html");
    utils::ok_to(body.into())
}
