use sqlx::postgres::PgPoolOptions;
use std::{net::TcpListener, time::Duration};

use production::{configuration, email_client::EmailClient, startup, telemetry};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = telemetry::get_subscriber("production".into(), "info".into(), std::io::stdout);
    telemetry::init_subscriber(subscriber);

    let configuration =
        configuration::get_configuration().expect("Failed to read configuration file");

    let connection_pool = PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());

    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address");

    let email_client = EmailClient::new(
        configuration
            .email_client
            .url()
            .expect("Faile to parse URL"),
        sender_email,
        configuration.email_client.authorization_token,
    );

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address)?;

    startup::run(listener, connection_pool, email_client)?.await
}
