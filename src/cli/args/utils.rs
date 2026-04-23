pub(crate) fn parse_key_val(s: &str) -> Result<(String, String), String> {
    let pos = s
        .find('=')
        .ok_or_else(|| format!("Expected KEY=VALUE, got '{}'", s))?;
    Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}
