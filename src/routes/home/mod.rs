use crate::utils;
use actix_web::HttpResponse;

pub async fn home() -> HttpResponse {
    let body = include_str!("home.html");
    utils::ok_to(body.into())
}
