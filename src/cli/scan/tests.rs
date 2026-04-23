use super::{parse_pkg_spec, tool_to_osv_ecosystem};

#[test]
fn parses_scoped_npm_specs() {
    assert_eq!(
        parse_pkg_spec("@types/node@20.0.0"),
        ("@types/node".into(), "20.0.0".into())
    );
}

#[test]
fn parses_python_constraint_specs() {
    assert_eq!(
        parse_pkg_spec("requests[security]==2.31.0"),
        ("requests".into(), "2.31.0".into())
    );
}

#[test]
fn parses_colon_version_specs() {
    assert_eq!(
        parse_pkg_spec("laravel/framework:12.0.0"),
        ("laravel/framework".into(), "12.0.0".into())
    );
}

#[test]
fn maps_tool_names_to_osv_ecosystems() {
    assert_eq!(tool_to_osv_ecosystem("pip"), "PyPI");
    assert_eq!(tool_to_osv_ecosystem("cargo"), "crates.io");
    assert_eq!(tool_to_osv_ecosystem("pub"), "pub.dev");
}
