use production::{configuration, startup::Application, telemetry};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = telemetry::get_subscriber("production".into(), "info".into(), std::io::stdout);
    telemetry::init_subscriber(subscriber);

    let configuration =
        configuration::get_configuration().expect("Failed to read configuration file");

    let application = Application::build(&configuration).await?;
    application.run_until_stopped().await?;

    Ok(())
}
