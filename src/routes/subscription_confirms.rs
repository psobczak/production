use actix_web::{web, HttpResponse};
use serde::Deserialize;

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(_parameters))]
pub async fn confirm(_parameters: web::Query<Parameters>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
