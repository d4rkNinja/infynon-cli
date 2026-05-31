use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const BIN: &str = env!("CARGO_BIN_EXE_infynon");

struct TempWorkspace {
    root: PathBuf,
}

impl TempWorkspace {
    fn new(name: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "infynon-{}-{}-{}",
            name,
            std::process::id(),
            unique
        ));
        fs::create_dir_all(root.join(".infynon/api/flows")).unwrap();
        fs::create_dir_all(root.join(".infynon/api/nodes")).unwrap();
        Self { root }
    }

    fn write(&self, relative: &str, content: &str) {
        let path = self.root.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }
}

impl Drop for TempWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn snapshot(name: &str) -> String {
    fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("snapshots")
            .join(name),
    )
    .unwrap()
}

fn normalize(text: String) -> String {
    text.replace("\r\n", "\n")
}

fn normalize_json_run(text: String) -> String {
    let mut value: serde_json::Value = serde_json::from_str(&text).unwrap();
    scrub_duration_fields(&mut value);
    serde_json::to_string_pretty(&value).unwrap()
}

fn scrub_duration_fields(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            if map.contains_key("duration_ms") {
                map.insert("duration_ms".to_string(), serde_json::json!(0));
            }
            for value in map.values_mut() {
                scrub_duration_fields(value);
            }
        }
        serde_json::Value::Array(values) => {
            for value in values {
                scrub_duration_fields(value);
            }
        }
        _ => {}
    }
}

fn normalize_junit_run(text: String) -> String {
    regex::Regex::new(r#"time="[^"]+""#)
        .unwrap()
        .replace_all(&normalize(text), r#"time="0""#)
        .to_string()
}

fn assert_help_snapshot(args: &[&str], snapshot_name: &str) {
    let output = Command::new(BIN).args(args).output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        normalize(String::from_utf8(output.stdout).unwrap()).trim(),
        normalize(snapshot(snapshot_name)).trim()
    );
}

#[test]
fn weave_flow_run_no_input_exits_21_when_prompt_value_is_missing() {
    let workspace = TempWorkspace::new("weave-no-input");
    workspace.write(
        ".infynon/api/nodes/login.toml",
        r#"
id = "login"
name = "Login"
method = "GET"
path = "/ping"
headers = {}
extractions = []
assertions = []
tags = []

[[prompt_inputs]]
var = "otp"
label = "OTP"
secret = false
type = "text"
options = []
"#,
    );
    workspace.write(
        ".infynon/api/flows/auth.toml",
        r#"
id = "auth"
name = "Auth"
entry = "login"
edges = []
base_url = "http://example.test"
"#,
    );

    let output = Command::new(BIN)
        .args(["weave", "flow", "run", "auth", "--no-input"])
        .current_dir(&workspace.root)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(21));
}

#[test]
fn weave_flow_run_json_matches_snapshot_for_invalid_flow() {
    let workspace = TempWorkspace::new("weave-json");
    workspace.write(
        ".infynon/api/flows/broken.toml",
        r#"
id = "broken"
name = "Broken"
entry = "missing"
edges = []
base_url = "http://example.test"
"#,
    );

    let output = Command::new(BIN)
        .args([
            "weave",
            "flow",
            "run",
            "broken",
            "--format",
            "json",
            "--no-input",
        ])
        .current_dir(&workspace.root)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(22));
    assert_eq!(
        normalize_json_run(String::from_utf8(output.stdout).unwrap()).trim(),
        normalize_json_run(snapshot("weave_run_broken.json")).trim()
    );
}

#[test]
fn weave_flow_run_junit_matches_snapshot_for_invalid_flow() {
    let workspace = TempWorkspace::new("weave-junit");
    workspace.write(
        ".infynon/api/flows/broken.toml",
        r#"
id = "broken"
name = "Broken"
entry = "missing"
edges = []
base_url = "http://example.test"
"#,
    );

    let output = Command::new(BIN)
        .args([
            "weave",
            "flow",
            "run",
            "broken",
            "--format",
            "junit",
            "--no-input",
        ])
        .current_dir(&workspace.root)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(22));
    assert_eq!(
        normalize_junit_run(String::from_utf8(output.stdout).unwrap()).trim(),
        normalize_junit_run(snapshot("weave_run_broken.junit")).trim()
    );
}

#[test]
fn weave_flow_run_help_matches_snapshot() {
    assert_help_snapshot(
        &["weave", "flow", "run", "--help"],
        "weave_flow_run_help.txt",
    );
}

#[test]
fn pkg_explain_help_matches_snapshot() {
    assert_help_snapshot(&["pkg", "explain", "--help"], "pkg_explain_help.txt");
}
