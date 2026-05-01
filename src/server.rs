use tokio::net::TcpListener;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::command::CommandTable;
use crate::config::Config;
use crate::connection;
use crate::storage::db::Database;

/// Start the Redis server
pub async fn run(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let addr = config.address();
    let listener = TcpListener::bind(&addr).await?;
    log::info!("Redis server listening on {}", addr);

    let db = Arc::new(Mutex::new(Database::new()));
    let cmd_table = Arc::new(CommandTable::new());

    // Spawn background active expiry task
    let db_clone = db.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(100));
        loop {
            interval.tick().await;
            let mut db = db_clone.lock().await;
            db.active_expire_cycle(20);
        }
    });

    loop {
        let (stream, addr) = listener.accept().await?;
        log::info!("New connection from {}", addr);

        let db = db.clone();
        let cmd_table = cmd_table.clone();

        tokio::spawn(async move {
            connection::handle_connection(stream, db, cmd_table).await;
            log::info!("Connection from {} closed", addr);
        });
    }
}
