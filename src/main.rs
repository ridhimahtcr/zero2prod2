//! src/main.rs

use std::net::TcpListener;
use zero2prod2::configuration::get_configuration;
use zero2prod2::startup::run;
use sqlx::PgPool;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};
use tracing_log::LogTracer;
use zero2prod2::telemetry::{get_subscriber, init_subscriber};
use secrecy::ExposeSecret;


#[tokio::main]
async fn main() -> Result<(), std::io::Error> {

    let subscriber = get_subscriber("zero2prod2".into(), "info".into(), std::io::stdout
    );


    LogTracer::init().expect("Failed to set logger");

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or(EnvFilter::new("info"));
    let formatting_layer = BunyanFormattingLayer::new(
        "zero2prod2".into(),
        std::io::stdout
    );

    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);

    set_global_default(subscriber).expect("Failed to set subscriber");

    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_pool = PgPool::connect(
        &configuration.database.connection_string().expose_secret())
        .await
        .expect("Failed to connect to Postgres.");

    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool)?.await?;
    Ok(())
}