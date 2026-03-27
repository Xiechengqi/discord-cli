use serde_json::Value;

use crate::errors::{AppError, AppResult};

pub fn required_string<'a>(params: &'a Value, key: &str) -> AppResult<&'a str> {
    params
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::InvalidParams(format!("`{key}` is required")))
}

pub fn optional_string<'a>(params: &'a Value, key: &str) -> Option<&'a str> {
    params.get(key).and_then(Value::as_str).map(str::trim).filter(|s| !s.is_empty())
}

pub fn optional_u64(params: &Value, key: &str, default: u64) -> u64 {
    params
        .get(key)
        .and_then(Value::as_u64)
        .unwrap_or(default)
}
