use actix_web::{web, HttpResponse};
use anyhow::Context;
use chrono::Utc;
use rand::{distributions::Alphanumeric, Rng};
use serde::Deserialize;
use sqlx::{PgPool, Postgres, Transaction};
use std::convert::AsRef;
use strum::AsRefStr;
use uuid::Uuid;

use crate::{
    domain::{NewSubscriber, SubscriberEmail, SubscriberName},
    email_client::EmailClient,
    errors::{StoreTokenError, SubscribeError},
    startup::ApplicationBaseUrl,
};

pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    base_url: web::Data<ApplicationBaseUrl>,
) -> Result<HttpResponse, SubscribeError> {
    let mut transaction = pool
        .begin()
        .await
        .context("Failed to aquire a Postgres connectin from the pool")?;

    let new_subscriber = form.0.try_into().map_err(SubscribeError::ValidationError)?;
    let subscriber_id = insert_subscriber(&new_subscriber, &mut transaction)
        .await
        .context("Failed to insert new subscriber in the database")?;
    let subscription_token = generate_subscription_token();
    store_token(&mut transaction, subscriber_id, &subscription_token)
        .await
        .context("Failed to store the confirmation token for a new subscriber")?;
    send_confirmation_email(
        &email_client,
        new_subscriber,
        base_url.0.as_ref(),
        &subscription_token,
    )
    .await
    .context("Failed to send a confirmation email")?;

    transaction
        .commit()
        .await
        .context("Failed to commit transaction")?;

    Ok(HttpResponse::Created().finish())
}

#[derive(Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

#[derive(AsRefStr, Debug)]
#[strum(serialize_all = "snake_case")]
pub enum SubscriptionStatus {
    PendingConfirmation,
    Confirmed,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(form: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(form.name)?;
        let email = SubscriberEmail::parse(form.email)?;
        Ok(NewSubscriber::new(email, name))
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, transaction)
)]
pub async fn insert_subscriber(
    new_subscriber: &NewSubscriber,
    transaction: &mut Transaction<'_, Postgres>,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        subscriber_id,
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now(),
        SubscriptionStatus::PendingConfirmation.as_ref()
    )
    .execute(transaction)
    .await?;
    Ok(subscriber_id)
}

#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(email_client, new_subscriber)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: &str,
    subscription_token: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url, subscription_token
    );
    let plain_body = format!(
        "Welcome to our newsletter!\nVisit {} to confirm your subscription",
        confirmation_link
    );
    let html_body = format!(
        "Welcome to our newsletter!<br />\
        Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
    );

    email_client
        .send_email(new_subscriber.email, "Welcome!", &html_body, &plain_body)
        .await
}

/// Generate a random 25-character long case-sensitive subscrption token
fn generate_subscription_token() -> String {
    let mut rng = rand::thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

#[tracing::instrument(
    name = "Store subscription token in the database",
    skip(subscription_token, transaction)
)]
pub async fn store_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
    subscription_token: &str,
) -> Result<(), StoreTokenError> {
    sqlx::query!(
        r#"
    INSERT INTO subscription_tokens (subscription_token, subscriber_id) 
    VALUES ($1, $2)
    "#,
        subscription_token,
        subscriber_id
    )
    .execute(transaction)
    .await
    .map_err(StoreTokenError)?;
    Ok(())
}
