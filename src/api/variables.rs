use std::collections::HashMap;
use std::sync::OnceLock;
use serde_json::Value;

// ── .env file loader ──────────────────────────────────────────────────────────

static DOTENV: OnceLock<HashMap<String, String>> = OnceLock::new();

pub fn get_placeholder_regex() -> &'static regex::Regex {
    static PLACEHOLDER_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    PLACEHOLDER_RE.get_or_init(|| {
        regex::Regex::new(r"\{([A-Za-z_][A-Za-z0-9_]*)\}").unwrap()
    })
}

fn get_dotenv() -> &'static HashMap<String, String> {
    DOTENV.get_or_init(|| {
        let mut map = HashMap::new();
        // Try .infynon/.env first (managed by `infynon weave env`), then fall back to root .env
        let content = std::fs::read_to_string(".infynon/.env")
            .or_else(|_| std::fs::read_to_string(".env"))
            .unwrap_or_default();
        for line in content.lines() {
                let line = line.trim();
                // Skip comments and empty lines
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some(eq_pos) = line.find('=') {
                    let key = line[..eq_pos].trim().to_string();
                    let mut val = line[eq_pos + 1..].trim().to_string();
                    // Strip surrounding quotes (single or double)
                    if (val.starts_with('"') && val.ends_with('"'))
                        || (val.starts_with('\'') && val.ends_with('\''))
                    {
                        val = val[1..val.len() - 1].to_string();
                    }
                    if !key.is_empty() {
                        map.insert(key, val);
                    }
                }
            }
        map
    })
}

/// Look up an environment variable: first from .infynon/.env, then process env.
/// Returns None if not found in either.
fn lookup_env_var(name: &str) -> Option<String> {
    let dotenv = get_dotenv();
    if let Some(val) = dotenv.get(name) {
        return Some(val.clone());
    }
    std::env::var(name).ok()
}

/// Check whether a variable name exists in the .infynon/.env file or process env.
/// Used by the prompt system to skip asking for vars that are already set.
pub fn env_has_var(name: &str) -> bool {
    get_dotenv().contains_key(name) || std::env::var(name).is_ok()
}

fn substitute_env_pass(s: &str) -> String {
    let mut output = String::with_capacity(s.len());
    let mut i = 0;
    let bytes = s.as_bytes();
    while i < bytes.len() {
        if bytes[i] == b'{' {
            if let Some(close) = s[i..].find('}') {
                let inner = &s[i + 1..i + close];
                if inner.starts_with('$') {
                    let env_name = &inner[1..];
                    if let Some(val) = lookup_env_var(env_name) {
                        output.push_str(&val);
                    } else {
                        output.push_str(&s[i..i + close + 1]);
                    }
                    i += close + 1;
                    continue;
                }
            }
        }
        output.push(bytes[i] as char);
        i += 1;
    }
    output
}

/// Substitute only `{$ENV_VAR}` placeholders in a string (used for label display).
pub fn substitute_env_placeholders(s: &str) -> String {
    substitute_env_pass(s)
}

// ── Substitution ──────────────────────────────────────────────────────────────

/// Substitute `{var_name}` or `{$ENV_VAR}` placeholders in a string template.
///
/// Rules:
/// - `{$VAR_NAME}` → look up environment variable VAR_NAME (from .env then process env)
/// - `{var_name}`  → look up in context map
/// - If the template is exactly `{var_name}` (nothing else), the variable value
///   is returned as a JSON Value preserving its original type (number, bool, etc.)
///   — used for body field substitution.
/// - Otherwise (partial match, e.g. `Bearer {token}`), the value is coerced
///   to its string representation and spliced in — used for paths and headers.
pub fn substitute(template: &str, context: &HashMap<String, Value>) -> String {
    substitute_str(template, context)
}

pub fn substitute_str(template: &str, context: &HashMap<String, Value>) -> String {
    // First pass: handle {$ENV_VAR} placeholders
    let mut result = substitute_env_pass(template);

    // Second pass: handle {var_name} context placeholders
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
    // Handle {$ENV_VAR} in body
    let mut substituted = substitute_env_pass(body_json);

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

/// Convert `--set KEY=VALUE` pairs into a context map, parsing JSON values where possible.
pub fn parse_set_vars(set_vars: &[(String, String)]) -> HashMap<String, Value> {
    set_vars
        .iter()
        .map(|(k, v)| {
            let val = serde_json::from_str::<Value>(v)
                .unwrap_or_else(|_| Value::String(v.clone()));
            (k.clone(), val)
        })
        .collect()
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
