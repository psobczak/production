use sqlx::{Connection, PgConnection};
use std::net::TcpListener;

#[tokio::test]
async fn health_check_works() {
    let address = spawn_app();

    let client = reqwest::Client::new();

    let response = client
        .get(format!("{address}/health_check"))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.content_length(), Some(0));
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn subscribe_returns_200_for_a_valid_form_data() {
    let address = spawn_app();
    let configuration =
        production::configuration::get_configuration().expect("Failed to read configuration");
    let connection_string = configuration.database.connection_string();
    let connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres");
    let client = reqwest::Client::new();
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let response = client
        .post(format!("{address}/subscriptions"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn returns_400_when_data_is_missing() {
    let address = spawn_app();
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(format!("{address}/subscriptions"))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            response.status(),
            400,
            "The API did not fail with 400 Bad Request when the payload was {}",
            error_message
        );
    }
}

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let server = production::startup::run(listener).expect("Failed to bind address");
    let _ = tokio::spawn(server);
    format!("http://127.0.0.1:{port}")
}
