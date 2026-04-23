use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::api::types::{
    Assertion, Edge, Extraction, Flow, FlowRunResult, Node, OnFail, PromptInput,
};

pub fn nodes_dir() -> PathBuf {
    let dir = PathBuf::from(".infynon/api/nodes");
    fs::create_dir_all(&dir).ok();
    dir
}

pub fn flows_dir() -> PathBuf {
    let dir = PathBuf::from(".infynon/api/flows");
    fs::create_dir_all(&dir).ok();
    dir
}

pub fn runs_dir() -> PathBuf {
    let dir = PathBuf::from(".infynon/api/runs");
    fs::create_dir_all(&dir).ok();
    dir
}

fn detect_project_yaml() -> bool {
    for dir in [nodes_dir(), flows_dir()] {
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if ext == "yaml" || ext == "yml" {
                    return true;
                }
            }
        }
    }
    false
}

fn existing_definition_path(dir: &Path, id: &str) -> Option<PathBuf> {
    ["yaml", "yml", "toml"]
        .into_iter()
        .map(|ext| dir.join(format!("{}.{}", id, ext)))
        .find(|path| path.exists())
}

fn yaml_definition_path(dir: &Path, id: &str) -> Option<PathBuf> {
    ["yaml", "yml"]
        .into_iter()
        .map(|ext| dir.join(format!("{}.{}", id, ext)))
        .find(|path| path.exists())
}

include!("storage/yaml_load.rs");
include!("storage/yaml_save.rs");
include!("storage/nodes.rs");
include!("storage/flows.rs");
include!("storage/runs.rs");

#[cfg(test)]
mod tests {
    use super::{convert_yaml_flow, flow_to_yaml_save, YamlFlow};
    use crate::api::types::{Edge, Flow};

    #[test]
    fn yaml_round_trip_preserves_per_incoming_edge_metadata() {
        let flow = Flow {
            id: "checkout".to_string(),
            name: "Checkout".to_string(),
            entry: "login".to_string(),
            edges: vec![
                Edge {
                    from: "login".to_string(),
                    to: "coupon".to_string(),
                    carry: vec!["token".to_string()],
                    condition: Some("status == 200".to_string()),
                },
                Edge {
                    from: "coupon".to_string(),
                    to: "cart".to_string(),
                    carry: vec!["coupon_code".to_string()],
                    condition: Some("body.valid == true".to_string()),
                },
                Edge {
                    from: "login".to_string(),
                    to: "cart".to_string(),
                    carry: vec!["token".to_string()],
                    condition: None,
                },
            ],
            description: Some("desc".to_string()),
            base_url: Some("http://localhost:3000".to_string()),
        };

        let save = flow_to_yaml_save(&flow);
        let yaml = serde_yaml::to_string(&save).unwrap();
        let parsed: YamlFlow = serde_yaml::from_str(&yaml).unwrap();
        let round_tripped = convert_yaml_flow(parsed);

        assert_eq!(round_tripped.edges.len(), 3);

        let login_coupon = round_tripped
            .edges
            .iter()
            .find(|edge| edge.from == "login" && edge.to == "coupon")
            .unwrap();
        let coupon_cart = round_tripped
            .edges
            .iter()
            .find(|edge| edge.from == "coupon" && edge.to == "cart")
            .unwrap();
        let login_cart = round_tripped
            .edges
            .iter()
            .find(|edge| edge.from == "login" && edge.to == "cart")
            .unwrap();

        assert_eq!(login_coupon.carry, vec!["token"]);
        assert_eq!(coupon_cart.carry, vec!["coupon_code"]);
        assert_eq!(login_cart.carry, vec!["token"]);
    }
}
