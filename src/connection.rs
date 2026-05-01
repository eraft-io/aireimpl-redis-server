use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::command::CommandTable;
use crate::protocol::parser::{ParseError, RespParser};
use crate::protocol::writer;
use crate::protocol::RespValue;
use crate::storage::db::Database;

use std::sync::Arc;
use tokio::sync::Mutex;

/// Handle a single client connection
pub async fn handle_connection(
    mut stream: TcpStream,
    db: Arc<Mutex<Database>>,
    cmd_table: Arc<CommandTable>,
) {
    let mut buf = BytesMut::with_capacity(4096);

    loop {
        // Read data from the client
        let _n = match stream.read_buf(&mut buf).await {
            Ok(0) => return, // Connection closed
            Ok(n) => n,
            Err(e) => {
                log::error!("Failed to read from connection: {}", e);
                return;
            }
        };

        // Try to parse and process commands
        loop {
            if buf.is_empty() {
                break;
            }

            match RespParser::parse(&buf) {
                Ok((value, consumed)) => {
                    let _ = buf.split_to(consumed);

                    // Extract args from the parsed command
                    let response = if let Some(args) = value.to_args() {
                        if !args.is_empty() {
                            let mut db = db.lock().await;
                            cmd_table.execute(&mut db, &args)
                        } else {
                            RespValue::error("ERR empty command")
                        }
                    } else {
                        RespValue::error("ERR invalid command format")
                    };

                    // Write response
                    let encoded = writer::encode(&response);
                    if let Err(e) = stream.write_all(&encoded).await {
                        log::error!("Failed to write response: {}", e);
                        return;
                    }
                }
                Err(ParseError::Incomplete) => {
                    // Need more data
                    break;
                }
                Err(e) => {
                    log::error!("Parse error: {}", e);
                    let response = RespValue::error(format!("ERR {}", e));
                    let encoded = writer::encode(&response);
                    let _ = stream.write_all(&encoded).await;
                    buf.clear();
                    break;
                }
            }
        }
    }
}
