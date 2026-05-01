use super::RespValue;

/// Encode a RespValue into RESP wire format bytes
pub fn encode(value: &RespValue) -> Vec<u8> {
    let mut buf = Vec::new();
    encode_into(value, &mut buf);
    buf
}

fn encode_into(value: &RespValue, buf: &mut Vec<u8>) {
    match value {
        RespValue::SimpleString(s) => {
            buf.push(b'+');
            buf.extend_from_slice(s.as_bytes());
            buf.extend_from_slice(b"\r\n");
        }
        RespValue::Error(s) => {
            buf.push(b'-');
            buf.extend_from_slice(s.as_bytes());
            buf.extend_from_slice(b"\r\n");
        }
        RespValue::Integer(n) => {
            buf.push(b':');
            buf.extend_from_slice(n.to_string().as_bytes());
            buf.extend_from_slice(b"\r\n");
        }
        RespValue::BulkString(None) => {
            buf.extend_from_slice(b"$-1\r\n");
        }
        RespValue::BulkString(Some(data)) => {
            buf.push(b'$');
            buf.extend_from_slice(data.len().to_string().as_bytes());
            buf.extend_from_slice(b"\r\n");
            buf.extend_from_slice(data);
            buf.extend_from_slice(b"\r\n");
        }
        RespValue::Array(None) => {
            buf.extend_from_slice(b"*-1\r\n");
        }
        RespValue::Array(Some(items)) => {
            buf.push(b'*');
            buf.extend_from_slice(items.len().to_string().as_bytes());
            buf.extend_from_slice(b"\r\n");
            for item in items {
                encode_into(item, buf);
            }
        }
    }
}
