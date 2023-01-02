use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::net::TcpListener;

use production::{configuration, email_client::EmailClient, startup, telemetry};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = telemetry::get_subscriber("production".into(), "info".into(), std::io::stdout);
    telemetry::init_subscriber(subscriber);

    let config = configuration::get_configuration().expect("Failed to read configuration file");
    let connection = PgPool::connect(config.database.connection_string().expose_secret())
        .await
        .expect("Failed to connect to Postgres");

    let sender_email = config
        .email_client
        .sender()
        .expect("Invalid sender email address");

    let email_client = EmailClient::new(
        config.email_client.url().expect("Faile to parse URL"),
        sender_email,
        config.email_client.authorization_token,
    );

    let address = format!("127.0.0.1:{}", config.application_port);
    let listener = TcpListener::bind(address)?;

    startup::run(listener, connection, email_client)?.await
}
