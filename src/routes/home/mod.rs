use crate::util;
use actix_web::HttpResponse;

pub async fn home() -> HttpResponse {
    let body = include_str!("home.html");
    util::ok_to_return(body.into())
}
