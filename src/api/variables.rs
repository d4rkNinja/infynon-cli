use std::collections::HashMap;
use serde_json::Value;

/// Substitute `{var_name}` placeholders in a string template.
///
/// Rules:
/// - If a template is exactly `{var_name}` (nothing else), the variable value
///   is returned as a JSON Value preserving its original type (number, bool, etc.)
///   — used for body field substitution.
/// - Otherwise (partial match, e.g. `Bearer {token}`), the value is coerced
///   to its string representation and spliced in — used for paths and headers.
pub fn substitute_str(template: &str, context: &HashMap<String, Value>) -> String {
    let mut result = template.to_string();
    for (key, val) in context {
        let placeholder = format!("{{{}}}", key);
        let replacement = value_to_str(val);
        result = result.replace(&placeholder, &replacement);
    }
    result
}

/// Substitute placeholders in a JSON body template string.
///
/// - First does string substitution for each `{var}` placeholder.
/// - For fields that are exactly `{var}` we do type-aware replacement so that
///   integers/booleans stay as their proper JSON types.
pub fn substitute_body(body_json: &str, context: &HashMap<String, Value>) -> Value {
    // Start with naive string substitution for string-embedded placeholders
    // (e.g. "Bearer {token}")
    let mut substituted = body_json.to_string();
    for (key, val) in context {
        let placeholder = format!("\"{{{}}}\"", key);
        // Replace JSON string placeholder "  {var}  " with the raw JSON value
        // when the entire string value is just the placeholder.
        match val {
            Value::String(s) => {
                // String values: substitute in-place (keep quotes)
                let str_placeholder = format!("{{{}}}", key);
                substituted = substituted.replace(&str_placeholder, s);
            }
            Value::Number(_) | Value::Bool(_) | Value::Null => {
                // Non-string values: replace the quoted placeholder with the raw value
                let raw = val.to_string();
                substituted = substituted.replace(&placeholder, &raw);
            }
            _ => {
                // Arrays/objects: replace quoted placeholder with raw JSON
                let raw = val.to_string();
                substituted = substituted.replace(&placeholder, &raw);
            }
        }
    }

    // Also do a pass for any remaining {var} inside strings
    for (key, val) in context {
        let str_placeholder = format!("{{{}}}", key);
        if substituted.contains(&str_placeholder) {
            substituted = substituted.replace(&str_placeholder, &value_to_str(val));
        }
    }

    // Parse the result; fall back to a JSON string if it fails
    serde_json::from_str(&substituted).unwrap_or_else(|_| Value::String(substituted))
}

/// Substitute placeholders in a path string (e.g. `/users/{user_id}`).
pub fn substitute_path(path: &str, context: &HashMap<String, Value>) -> String {
    substitute_str(path, context)
}

/// Substitute placeholders in all header values.
pub fn substitute_headers(
    headers: &HashMap<String, String>,
    context: &HashMap<String, Value>,
) -> HashMap<String, String> {
    headers
        .iter()
        .map(|(k, v)| (k.clone(), substitute_str(v, context)))
        .collect()
}

/// Merge the `carry` variables from a source context into a destination context.
/// If `carry` is empty, all variables are carried.
pub fn carry_context(
    source: &HashMap<String, Value>,
    dest: &mut HashMap<String, Value>,
    carry: &[String],
) {
    if carry.is_empty() {
        // carry everything
        for (k, v) in source {
            dest.insert(k.clone(), v.clone());
        }
    } else {
        for key in carry {
            if let Some(val) = source.get(key) {
                dest.insert(key.clone(), val.clone());
            }
        }
    }
}

fn value_to_str(val: &Value) -> String {
    match val {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_substitute_str() {
        let mut ctx: HashMap<String, Value> = HashMap::new();
        ctx.insert("token".to_string(), json!("abc123"));
        ctx.insert("user_id".to_string(), json!(42));

        assert_eq!(
            substitute_str("Bearer {token}", &ctx),
            "Bearer abc123"
        );
        assert_eq!(
            substitute_str("/users/{user_id}", &ctx),
            "/users/42"
        );
    }

    #[test]
    fn test_substitute_body_preserves_types() {
        let mut ctx: HashMap<String, Value> = HashMap::new();
        ctx.insert("product_id".to_string(), json!(99));
        ctx.insert("active".to_string(), json!(true));
        ctx.insert("name".to_string(), json!("test-user"));

        let template = r#"{"product_id": "{product_id}", "active": "{active}", "name": "{name}"}"#;
        let result = substitute_body(template, &ctx);

        assert_eq!(result["product_id"], json!(99));
        assert_eq!(result["active"], json!(true));
        assert_eq!(result["name"], json!("test-user"));
    }
}
