use production::{configuration, startup};
use sqlx::PgPool;
use std::net::TcpListener;

use production::telemetry;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = telemetry::get_subscriber("production".into(), "info".into(), std::io::stdout);
    telemetry::init_subscriber(subscriber);

    let config = configuration::get_configuration().expect("Failed to read configuration file");
    let connection = PgPool::connect(&config.database.connection_string())
        .await
        .expect("Failed to connect to Postgres");
    let address = format!("127.0.0.1:{}", config.application_port);
    let listener = TcpListener::bind(address)?;
    startup::run(listener, connection)?.await
}
