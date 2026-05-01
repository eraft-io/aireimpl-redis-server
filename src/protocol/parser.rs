use bytes::BytesMut;
use std::io;

use super::RespValue;

/// RESP protocol parser with incremental parsing support
pub struct RespParser;

#[derive(Debug)]
pub enum ParseError {
    Incomplete,
    InvalidFormat(String),
    Io(io::Error),
}

impl From<io::Error> for ParseError {
    fn from(e: io::Error) -> Self {
        ParseError::Io(e)
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Incomplete => write!(f, "Incomplete data"),
            ParseError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            ParseError::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl RespParser {
    /// Try to parse a complete RESP value from the buffer.
    /// Returns the parsed value and the number of bytes consumed.
    pub fn parse(buf: &BytesMut) -> Result<(RespValue, usize), ParseError> {
        if buf.is_empty() {
            return Err(ParseError::Incomplete);
        }

        match buf[0] {
            b'+' => Self::parse_simple_string(buf),
            b'-' => Self::parse_error(buf),
            b':' => Self::parse_integer(buf),
            b'$' => Self::parse_bulk_string(buf),
            b'*' => Self::parse_array(buf),
            _ => Self::parse_inline(buf),
        }
    }

    /// Parse inline command (space-separated, no RESP prefix)
    fn parse_inline(buf: &BytesMut) -> Result<(RespValue, usize), ParseError> {
        let line_end = Self::find_crlf(buf, 0).ok_or(ParseError::Incomplete)?;
        let line = &buf[..line_end];
        let line_str = std::str::from_utf8(line)
            .map_err(|_| ParseError::InvalidFormat("Invalid UTF-8 in inline command".into()))?;

        let args: Vec<RespValue> = line_str
            .split_whitespace()
            .map(|s| RespValue::BulkString(Some(s.as_bytes().to_vec())))
            .collect();

        if args.is_empty() {
            return Err(ParseError::InvalidFormat("Empty inline command".into()));
        }

        Ok((RespValue::Array(Some(args)), line_end + 2))
    }

    fn parse_simple_string(buf: &BytesMut) -> Result<(RespValue, usize), ParseError> {
        let line_end = Self::find_crlf(buf, 1).ok_or(ParseError::Incomplete)?;
        let s = std::str::from_utf8(&buf[1..line_end])
            .map_err(|_| ParseError::InvalidFormat("Invalid UTF-8".into()))?
            .to_string();
        Ok((RespValue::SimpleString(s), line_end + 2))
    }

    fn parse_error(buf: &BytesMut) -> Result<(RespValue, usize), ParseError> {
        let line_end = Self::find_crlf(buf, 1).ok_or(ParseError::Incomplete)?;
        let s = std::str::from_utf8(&buf[1..line_end])
            .map_err(|_| ParseError::InvalidFormat("Invalid UTF-8".into()))?
            .to_string();
        Ok((RespValue::Error(s), line_end + 2))
    }

    fn parse_integer(buf: &BytesMut) -> Result<(RespValue, usize), ParseError> {
        let line_end = Self::find_crlf(buf, 1).ok_or(ParseError::Incomplete)?;
        let s = std::str::from_utf8(&buf[1..line_end])
            .map_err(|_| ParseError::InvalidFormat("Invalid UTF-8".into()))?;
        let n: i64 = s
            .parse()
            .map_err(|_| ParseError::InvalidFormat("Invalid integer".into()))?;
        Ok((RespValue::Integer(n), line_end + 2))
    }

    fn parse_bulk_string(buf: &BytesMut) -> Result<(RespValue, usize), ParseError> {
        let line_end = Self::find_crlf(buf, 1).ok_or(ParseError::Incomplete)?;
        let len_str = std::str::from_utf8(&buf[1..line_end])
            .map_err(|_| ParseError::InvalidFormat("Invalid UTF-8".into()))?;
        let len: i64 = len_str
            .parse()
            .map_err(|_| ParseError::InvalidFormat("Invalid bulk string length".into()))?;

        if len == -1 {
            return Ok((RespValue::BulkString(None), line_end + 2));
        }

        let len = len as usize;
        let data_start = line_end + 2;
        let data_end = data_start + len;

        if buf.len() < data_end + 2 {
            return Err(ParseError::Incomplete);
        }

        let data = buf[data_start..data_end].to_vec();
        Ok((RespValue::BulkString(Some(data)), data_end + 2))
    }

    fn parse_array(buf: &BytesMut) -> Result<(RespValue, usize), ParseError> {
        let line_end = Self::find_crlf(buf, 1).ok_or(ParseError::Incomplete)?;
        let len_str = std::str::from_utf8(&buf[1..line_end])
            .map_err(|_| ParseError::InvalidFormat("Invalid UTF-8".into()))?;
        let len: i64 = len_str
            .parse()
            .map_err(|_| ParseError::InvalidFormat("Invalid array length".into()))?;

        if len == -1 {
            return Ok((RespValue::Array(None), line_end + 2));
        }

        let len = len as usize;
        let mut items = Vec::with_capacity(len);
        let mut pos = line_end + 2;

        for _ in 0..len {
            let sub_buf = BytesMut::from(&buf[pos..]);
            let (value, consumed) = Self::parse(&sub_buf)?;
            items.push(value);
            pos += consumed;
        }

        Ok((RespValue::Array(Some(items)), pos))
    }

    /// Find \r\n starting from position `start`
    fn find_crlf(buf: &BytesMut, start: usize) -> Option<usize> {
        for i in start..buf.len().saturating_sub(1) {
            if buf[i] == b'\r' && buf[i + 1] == b'\n' {
                return Some(i);
            }
        }
        None
    }
}
