//! Control-channel protocol: one JSON object per line.
//!
//! Request:  `{"id": 1, "method": "edit.undo", "params": {...}}` (`id`, `params` optional)
//! Response: `{"id": 1, "ok": <value>}` or `{"id": 1, "error": "<message>"}`
//!
//! Methods are engine command ids plus the UI-level methods handled by the UI layer
//! (`app.commands`, `app.state`, `ui.screenshot`). Input is untrusted: parsing never panics
//! and rejects oversized lines.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Longest accepted request line, in bytes.
pub const MAX_LINE: usize = 64 * 1024;
/// Longest accepted method name, in bytes.
pub const MAX_METHOD: usize = 128;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    #[serde(default)]
    pub id: Value,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Response {
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ok: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Response {
    pub fn ok(id: Value, value: Value) -> Self {
        Self {
            id,
            ok: Some(value),
            error: None,
        }
    }

    pub fn error(id: Value, msg: impl Into<String>) -> Self {
        Self {
            id,
            ok: None,
            error: Some(msg.into()),
        }
    }

    pub fn from_result(id: Value, result: Result<Value, String>) -> Self {
        match result {
            Ok(v) => Self::ok(id, v),
            Err(e) => Self::error(id, e),
        }
    }

    /// One line of JSON, without the trailing newline.
    pub fn to_line(&self) -> String {
        serde_json::to_string(self)
            .unwrap_or_else(|_| r#"{"id":null,"error":"cannot encode response"}"#.to_owned())
    }
}

/// Parse one request line. On failure, returns the best-effort id plus an error message.
pub fn parse_request(line: &str) -> Result<Request, Response> {
    if line.len() > MAX_LINE {
        return Err(Response::error(Value::Null, "request line too long"));
    }
    let req: Request = serde_json::from_str(line.trim())
        .map_err(|e| Response::error(Value::Null, format!("bad request: {e}")))?;
    if req.method.is_empty() || req.method.len() > MAX_METHOD {
        return Err(Response::error(req.id, "bad method name"));
    }
    Ok(req)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use serde_json::json;

    #[test]
    fn parses_minimal_request() {
        let r = parse_request(r#"{"method":"edit.undo"}"#).unwrap();
        assert_eq!(r.method, "edit.undo");
        assert_eq!(r.id, Value::Null);
        assert_eq!(r.params, Value::Null);
    }

    #[test]
    fn parses_full_request() {
        let r = parse_request(r#"{"id":7,"method":"ui.screenshot","params":{"path":"a.png"}}"#)
            .unwrap();
        assert_eq!(r.id, json!(7));
        assert_eq!(r.params, json!({"path": "a.png"}));
    }

    #[test]
    fn rejects_garbage_and_unknown_fields() {
        assert!(parse_request("not json").is_err());
        assert!(parse_request(r#"{"method":"x","extra":1}"#).is_err());
        assert!(parse_request(r#"{"id":1,"method":""}"#).is_err());
    }

    #[test]
    fn rejects_oversized_line() {
        let long = format!(r#"{{"method":"{}"}}"#, "a".repeat(MAX_LINE));
        let err = parse_request(&long).unwrap_err();
        assert_eq!(err.error.as_deref(), Some("request line too long"));
    }

    #[test]
    fn response_lines() {
        assert_eq!(
            Response::ok(json!(1), json!(true)).to_line(),
            r#"{"id":1,"ok":true}"#
        );
        assert_eq!(
            Response::error(Value::Null, "x").to_line(),
            r#"{"id":null,"error":"x"}"#
        );
    }

    proptest! {
        #[test]
        fn parse_never_panics(line in ".{0,256}") {
            let _ = parse_request(&line);
        }

        #[test]
        fn valid_method_round_trips(method in "[a-z][a-z._]{0,40}", id in any::<u32>()) {
            let line = json!({"id": id, "method": method}).to_string();
            let r = parse_request(&line).unwrap();
            prop_assert_eq!(r.method, method);
            prop_assert_eq!(r.id, json!(id));
        }
    }
}
