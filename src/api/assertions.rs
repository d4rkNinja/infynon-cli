use serde_json::Value;
use std::collections::HashMap;

use crate::api::types::AssertionResult;

/// Evaluate an assertion expression against a response.
///
/// Supported forms:
///   status == <code>
///   status != <code>
///   status >= <code>
///   status <= <code>
///   body.<path> exists
///   body.<path> not exists
///   body.<path> == <value>
///   body.<path> != <value>
///   body.<path> > <number>
///   body.<path> >= <number>
///   body.<path> < <number>
///   body.<path> <= <number>
///   body.<path> contains "<string>"
///   header.<name> == <value>
///   header.<name> contains "<string>"
///   header.<name> exists
pub fn evaluate(
    check: &str,
    status: u16,
    body: &Value,
    headers: &HashMap<String, String>,
) -> AssertionResult {
    let (passed, actual, message) = eval_inner(check.trim(), status, body, headers);
    AssertionResult {
        check: check.to_string(),
        passed,
        actual,
        message,
    }
}

fn eval_inner(
    check: &str,
    status: u16,
    body: &Value,
    headers: &HashMap<String, String>,
) -> (bool, String, Option<String>) {
    let parts: Vec<&str> = check.splitn(3, ' ').collect();

    match parts.as_slice() {
        // status <op> <code>
        ["status", op, rhs] => {
            let actual_str = status.to_string();
            if let Ok(rhs_n) = rhs.parse::<u16>() {
                let passed = compare_numbers(status as f64, op, rhs_n as f64);
                (passed, actual_str, None)
            } else {
                (
                    false,
                    actual_str,
                    Some(format!("Cannot parse status code '{}'", rhs)),
                )
            }
        }

        // body.<path> exists / not exists
        ["body", "exists"] => {
            let exists = !body.is_null();
            (exists, exists.to_string(), None)
        }

        ["body", "not", "exists"] => {
            let missing = body.is_null();
            (missing, (!missing).to_string(), None)
        }

        [subject, "exists"] if subject.starts_with("body.") => {
            let path = &subject["body.".len()..];
            let val = json_path(body, path);
            let exists = val.is_some() && val != Some(&Value::Null);
            (exists, exists.to_string(), None)
        }

        [subject, "not", "exists"] if subject.starts_with("body.") => {
            let path = &subject["body.".len()..];
            let val = json_path(body, path);
            let missing = val.is_none() || val == Some(&Value::Null);
            (missing, (!missing).to_string(), None)
        }

        // header.<name> exists
        [subject, "exists"] if subject.starts_with("header.") => {
            let name = &subject["header.".len()..];
            let key = name.to_lowercase();
            let found =
                headers.contains_key(&key) || headers.keys().any(|k| k.to_lowercase() == key);
            (found, found.to_string(), None)
        }

        // body.<path> <op> <rhs>
        [subject, op, rhs] if subject.starts_with("body.") => {
            let path = &subject["body.".len()..];
            let val = json_path(body, path);
            eval_value_op(val, op, rhs)
        }

        // header.<name> <op> <rhs>
        [subject, op, rhs] if subject.starts_with("header.") => {
            let name = &subject["header.".len()..];
            let key = name.to_lowercase();
            let header_val = headers
                .get(&key)
                .or_else(|| {
                    headers
                        .iter()
                        .find(|(k, _)| k.to_lowercase() == key)
                        .map(|(_, v)| v)
                })
                .cloned()
                .unwrap_or_default();
            let v = Value::String(header_val.clone());
            eval_value_op(Some(&v), op, rhs)
        }

        _ => (
            false,
            String::new(),
            Some(format!("Unrecognized assertion syntax: '{}'", check)),
        ),
    }
}

fn eval_value_op(val: Option<&Value>, op: &str, rhs: &str) -> (bool, String, Option<String>) {
    let actual_str = match val {
        Some(v) => json_to_display(v),
        None => "<missing>".to_string(),
    };

    let rhs_clean = rhs.trim_matches('"');

    match op {
        "==" => {
            if let Some(v) = val {
                let passed = value_eq(v, rhs_clean);
                (passed, actual_str, None)
            } else {
                (false, actual_str, None)
            }
        }
        "!=" => {
            if let Some(v) = val {
                let passed = !value_eq(v, rhs_clean);
                (passed, actual_str, None)
            } else {
                (true, actual_str, None) // missing != anything is true
            }
        }
        ">" | ">=" | "<" | "<=" => {
            if let Some(v) = val {
                if let Some(n) = value_as_f64(v) {
                    if let Ok(rhs_n) = rhs_clean.parse::<f64>() {
                        let passed = compare_numbers(n, op, rhs_n);
                        (passed, actual_str, None)
                    } else {
                        (
                            false,
                            actual_str,
                            Some(format!("'{}' is not a number", rhs_clean)),
                        )
                    }
                } else {
                    (
                        false,
                        actual_str.clone(),
                        Some(format!("Value '{}' is not a number", actual_str)),
                    )
                }
            } else {
                (false, actual_str, Some("Field missing".to_string()))
            }
        }
        "contains" => {
            if let Some(v) = val {
                let s = json_to_display(v);
                let passed = s.contains(rhs_clean);
                (passed, actual_str, None)
            } else {
                (false, actual_str, Some("Field missing".to_string()))
            }
        }
        _ => (
            false,
            actual_str,
            Some(format!("Unknown operator '{}'", op)),
        ),
    }
}

fn compare_numbers(lhs: f64, op: &str, rhs: f64) -> bool {
    match op {
        "==" => (lhs - rhs).abs() < f64::EPSILON,
        "!=" => (lhs - rhs).abs() >= f64::EPSILON,
        ">" => lhs > rhs,
        ">=" => lhs >= rhs,
        "<" => lhs < rhs,
        "<=" => lhs <= rhs,
        _ => false,
    }
}

fn value_eq(val: &Value, rhs: &str) -> bool {
    match val {
        Value::String(s) => s == rhs,
        Value::Number(n) => {
            if let Ok(r) = rhs.parse::<f64>() {
                n.as_f64()
                    .map(|v| (v - r).abs() < f64::EPSILON)
                    .unwrap_or(false)
            } else {
                false
            }
        }
        Value::Bool(b) => {
            if rhs == "true" {
                *b
            } else if rhs == "false" {
                !*b
            } else {
                false
            }
        }
        Value::Null => rhs == "null",
        _ => *val == rhs,
    }
}

fn value_as_f64(val: &Value) -> Option<f64> {
    match val {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.parse().ok(),
        Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
        _ => None,
    }
}

fn json_to_display(val: &Value) -> String {
    match val {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        other => other.to_string(),
    }
}

/// Resolve a dot-notation path in a JSON value.
/// e.g. `json_path(&body, "user.id")` → `body["user"]["id"]`
pub fn json_path<'a>(root: &'a Value, path: &str) -> Option<&'a Value> {
    let mut cur = root;
    for part in path.split('.') {
        // Try object key first
        if let Some(next) = cur.get(part) {
            cur = next;
        } else if let Ok(idx) = part.parse::<usize>() {
            // Try array index
            if let Some(next) = cur.get(idx) {
                cur = next;
            } else {
                return None;
            }
        } else {
            return None;
        }
    }
    Some(cur)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn empty_headers() -> HashMap<String, String> {
        HashMap::new()
    }

    #[test]
    fn test_status_eq() {
        let r = evaluate("status == 200", 200, &json!({}), &empty_headers());
        assert!(r.passed);
    }

    #[test]
    fn test_status_ne() {
        let r = evaluate("status == 404", 200, &json!({}), &empty_headers());
        assert!(!r.passed);
    }

    #[test]
    fn test_body_exists() {
        let body = json!({"cart_id": "abc"});
        let r = evaluate("body.cart_id exists", 200, &body, &empty_headers());
        assert!(r.passed);
    }

    #[test]
    fn test_root_body_exists() {
        let body = json!({"cart_id": "abc"});
        let r = evaluate("body exists", 200, &body, &empty_headers());
        assert!(r.passed);
    }

    #[test]
    fn test_body_not_exists() {
        let body = json!({"cart_id": "abc"});
        let r = evaluate("body.other exists", 200, &body, &empty_headers());
        assert!(!r.passed);
    }

    #[test]
    fn test_body_eq_string() {
        let body = json!({"status": "ok"});
        let r = evaluate("body.status == ok", 200, &body, &empty_headers());
        assert!(r.passed);
    }

    #[test]
    fn test_body_eq_number() {
        let body = json!({"count": 5});
        let r = evaluate("body.count > 0", 200, &body, &empty_headers());
        assert!(r.passed);
    }

    #[test]
    fn test_json_path_nested() {
        let body = json!({"user": {"id": 42}});
        let val = json_path(&body, "user.id");
        assert_eq!(val, Some(&json!(42)));
    }
}
