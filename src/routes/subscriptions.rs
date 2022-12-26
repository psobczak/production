use actix_web::{web, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

pub async fn subscriptions(_: web::Form<FormData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
