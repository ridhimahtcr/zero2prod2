use actix_web::HttpResponse;
use actix_web::http::header::LOCATION;
use actix_web::web;
use secrecy::Secret;


#[derive(serde::Deserialize)]
pub struct FormData {
    username: String,
    password: Secret<String>,
}

pub async fn login() -> HttpResponse {
    HttpResponse::SeeOther()
    .insert_header((LOCATION, "/"))
    .finish()
}