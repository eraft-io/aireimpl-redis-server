pub mod parser;
pub mod writer;

use std::fmt;

/// RESP protocol value types
#[derive(Debug, Clone, PartialEq)]
pub enum RespValue {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(Option<Vec<u8>>),
    Array(Option<Vec<RespValue>>),
}

impl RespValue {
    /// Create an OK simple string response
    pub fn ok() -> Self {
        RespValue::SimpleString("OK".to_string())
    }

    /// Create a null bulk string
    pub fn null() -> Self {
        RespValue::BulkString(None)
    }

    /// Create a null array
    pub fn null_array() -> Self {
        RespValue::Array(None)
    }

    /// Create an error response
    pub fn error(msg: impl Into<String>) -> Self {
        RespValue::Error(msg.into())
    }

    /// Create an integer response
    pub fn integer(n: i64) -> Self {
        RespValue::Integer(n)
    }

    /// Create a bulk string from bytes
    pub fn bulk(data: Vec<u8>) -> Self {
        RespValue::BulkString(Some(data))
    }

    /// Create a bulk string from &str
    pub fn bulk_string(s: &str) -> Self {
        RespValue::BulkString(Some(s.as_bytes().to_vec()))
    }

    /// Create an array of RespValues
    pub fn array(values: Vec<RespValue>) -> Self {
        RespValue::Array(Some(values))
    }

    /// Extract command arguments as byte slices from an Array
    pub fn to_args(&self) -> Option<Vec<Vec<u8>>> {
        match self {
            RespValue::Array(Some(items)) => {
                let mut args = Vec::with_capacity(items.len());
                for item in items {
                    match item {
                        RespValue::BulkString(Some(data)) => args.push(data.clone()),
                        _ => return None,
                    }
                }
                Some(args)
            }
            _ => None,
        }
    }
}

impl fmt::Display for RespValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RespValue::SimpleString(s) => write!(f, "{}", s),
            RespValue::Error(s) => write!(f, "ERR {}", s),
            RespValue::Integer(n) => write!(f, "{}", n),
            RespValue::BulkString(Some(data)) => {
                write!(f, "{}", String::from_utf8_lossy(data))
            }
            RespValue::BulkString(None) => write!(f, "(nil)"),
            RespValue::Array(Some(items)) => {
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", item)?;
                }
                Ok(())
            }
            RespValue::Array(None) => write!(f, "(nil)"),
        }
    }
}
