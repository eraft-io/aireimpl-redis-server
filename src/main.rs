mod protocol;
mod command;
mod storage;
mod server;
mod connection;
mod config;
mod persistence;

use config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();

    let config = Config::default();
    log::info!("Starting Redis server (Rust implementation)...");
    log::info!("Listening on {}:{}", config.bind, config.port);

    server::run(config).await
}
