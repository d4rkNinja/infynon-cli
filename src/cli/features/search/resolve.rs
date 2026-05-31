pub(super) fn resolve_search_ecosystems(
    ecosystem: Option<&str>,
) -> Result<Vec<&'static str>, String> {
    let Some(raw) = ecosystem else {
        return Ok(crate::ecosystems::catalog::DEFAULT_SEARCH_ECOSYSTEMS.to_vec());
    };

    let canonical =
        crate::ecosystems::catalog::canonical_search_ecosystem(raw).ok_or_else(|| {
            format!(
                "Search is not implemented for ecosystem '{}'.",
                raw.trim().to_ascii_lowercase()
            )
        })?;

    Ok(vec![canonical])
}
