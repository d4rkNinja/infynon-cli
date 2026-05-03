use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct NinjaManifest {
    #[serde(default = "manifest_version")]
    pub version: u32,
    #[serde(default)]
    pub default_workspace: Option<String>,
    #[serde(default)]
    pub agent_root_path: Option<String>,
    #[serde(default)]
    pub workspaces: Vec<WorkspaceSummary>,
    #[serde(default)]
    pub tasks: Vec<TaskSummary>,
    #[serde(default = "timestamp_now")]
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct UserIdentity {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default = "timestamp_now")]
    pub created_at: String,
    #[serde(default = "timestamp_now")]
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AgentCommandTemplates {
    #[serde(default)]
    pub codex: AgentCommandGroup,
    #[serde(default)]
    pub claude: AgentCommandGroup,
    #[serde(default)]
    pub gemini: AgentCommandGroup,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AgentCommandGroup {
    #[serde(default)]
    pub open: String,
    #[serde(default)]
    pub bootstrap: String,
    #[serde(default)]
    pub bootstrap_background: String,
    #[serde(default)]
    pub task: AgentTaskCommands,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AgentTaskCommands {
    #[serde(default)]
    pub create: String,
    #[serde(default)]
    pub start: String,
    #[serde(default)]
    pub resume: String,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub update: String,
    #[serde(default)]
    pub result: String,
    #[serde(default)]
    pub complete: String,
    #[serde(default)]
    pub fail: String,
    #[serde(default)]
    pub kill: String,
    #[serde(default)]
    pub remove: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkspaceSummary {
    pub name: String,
    #[serde(default)]
    pub folder_name: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub folders: Vec<WorkspaceFolder>,
    #[serde(default)]
    pub models: WorkspaceModels,
    #[serde(default)]
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkspaceRecord {
    pub name: String,
    #[serde(default)]
    pub folder_name: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub folders: Vec<WorkspaceFolder>,
    #[serde(default)]
    pub models: WorkspaceModels,
    #[serde(default)]
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkspaceFolder {
    pub folder_name: String,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct WorkspaceModels {
    #[serde(default)]
    pub lite_model: WorkspaceModelSlot,
    #[serde(default)]
    pub frontier_model: WorkspaceModelSlot,
    #[serde(default)]
    pub highest_frontier_model: WorkspaceModelSlot,
    #[serde(default)]
    pub super_lite_model: WorkspaceModelSlot,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkspaceModelSlot {
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default = "thinking_auto")]
    pub thinking: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskSummary {
    pub id: String,
    #[serde(default)]
    pub parent_task_id: Option<String>,
    #[serde(default)]
    pub blocked_by: Option<String>,
    #[serde(default)]
    pub blocked_reason: Option<String>,
    #[serde(default)]
    pub workspace: Option<String>,
    #[serde(default)]
    pub folder_name: Option<String>,
    pub status: String,
    #[serde(default)]
    pub agent: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub thinking: Option<String>,
    #[serde(default)]
    pub pid: Option<u32>,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub markdown_path: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskRecord {
    pub id: String,
    #[serde(default)]
    pub parent_task_id: Option<String>,
    #[serde(default)]
    pub blocked_by: Option<String>,
    #[serde(default)]
    pub blocked_reason: Option<String>,
    #[serde(default)]
    pub workspace: Option<String>,
    #[serde(default)]
    pub folder_name: Option<String>,
    #[serde(default)]
    pub agent: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub thinking: Option<String>,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub pid: Option<u32>,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub result: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub started_at: Option<String>,
    #[serde(default)]
    pub ended_at: Option<String>,
}

pub fn timestamp_now() -> String {
    Utc::now().to_rfc3339()
}

fn manifest_version() -> u32 {
    1
}

impl Default for WorkspaceModelSlot {
    fn default() -> Self {
        Self {
            model: None,
            thinking: thinking_auto(),
        }
    }
}

fn thinking_auto() -> String {
    "auto".to_string()
}
