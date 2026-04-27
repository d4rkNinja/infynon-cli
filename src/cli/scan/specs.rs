pub fn parse_pkg_spec(spec: &str) -> (String, String) {
    let spec = spec.trim();
    if let Some(stripped) = spec.strip_prefix('@') {
        if let Some(pos) = stripped.rfind('@') {
            let pos = pos + 1;
            let name = spec[..pos].to_string();
            let version = spec[pos + 1..].to_string();
            if !version.is_empty() {
                return (name, version);
            }
        }
        return (spec.to_string(), String::new());
    }
    for sep in &["==", "~=", "<=", ">=", "!="] {
        if let Some(pos) = spec.find(sep) {
            let raw_name = spec[..pos].trim();
            let name = raw_name
                .split('[')
                .next()
                .unwrap_or(raw_name)
                .trim()
                .to_string();
            let version = spec[pos + sep.len()..]
                .split(',')
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            if !name.is_empty() && !version.is_empty() {
                return (name, version);
            }
        }
    }
    for sep in &[">", "<"] {
        if let Some(pos) = spec.find(sep) {
            let name = spec[..pos]
                .split('[')
                .next()
                .unwrap_or(&spec[..pos])
                .trim()
                .to_string();
            let version = spec[pos + 1..]
                .split(',')
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            if !name.is_empty() && !version.is_empty() {
                return (name, version);
            }
        }
    }
    if spec.contains(':') && !spec.starts_with("http") && !spec.starts_with("git") {
        if let Some(pos) = spec.find(':') {
            let name = spec[..pos].trim().to_string();
            let version = spec[pos + 1..].trim().to_string();
            if !name.is_empty() && !version.is_empty() {
                return (name, version);
            }
        }
    }
    if let Some(pos) = spec.rfind('@') {
        let name = spec[..pos].trim().to_string();
        let version = spec[pos + 1..].trim().to_string();
        if !name.is_empty() && !version.is_empty() {
            return (name, version);
        }
    }
    (spec.to_string(), String::new())
}
