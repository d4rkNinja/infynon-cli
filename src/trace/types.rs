use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TraceLayer {
    Canonical,
    Team,
    User,
}

impl TraceLayer {
    pub fn as_str(&self) -> &'static str {
        match self {
            TraceLayer::Canonical => "canonical",
            TraceLayer::Team => "team",
            TraceLayer::User => "user",
        }
    }
}

impl FromStr for TraceLayer {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "canonical" => Ok(Self::Canonical),
            "team" => Ok(Self::Team),
            "user" => Ok(Self::User),
            _ => Err(format!("invalid layer '{}'", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TraceScope {
    Repo,
    Branch,
    PullRequest,
    File,
    User,
    Session,
    Package,
}

impl TraceScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            TraceScope::Repo => "repo",
            TraceScope::Branch => "branch",
            TraceScope::PullRequest => "pr",
            TraceScope::File => "file",
            TraceScope::User => "user",
            TraceScope::Session => "session",
            TraceScope::Package => "package",
        }
    }
}

impl FromStr for TraceScope {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "repo" => Ok(Self::Repo),
            "branch" => Ok(Self::Branch),
            "pr" | "pullrequest" | "pull-request" => Ok(Self::PullRequest),
            "file" => Ok(Self::File),
            "user" => Ok(Self::User),
            "session" => Ok(Self::Session),
            "package" => Ok(Self::Package),
            _ => Err(format!("invalid scope '{}'", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NoteStatus {
    Active,
    Stale,
    Archived,
}

impl NoteStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            NoteStatus::Active => "active",
            NoteStatus::Stale => "stale",
            NoteStatus::Archived => "archived",
        }
    }
}

impl FromStr for NoteStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "active" => Ok(Self::Active),
            "stale" => Ok(Self::Stale),
            "archived" => Ok(Self::Archived),
            _ => Err(format!("invalid status '{}'", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SourceKind {
    Redis,
    Postgres,
    Mysql,
    Sqlite,
}

impl SourceKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            SourceKind::Redis => "redis",
            SourceKind::Postgres => "postgres",
            SourceKind::Mysql => "mysql",
            SourceKind::Sqlite => "sqlite",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SyncDirection {
    Pull,
    Push,
    Both,
}

impl SyncDirection {
    pub fn as_str(&self) -> &'static str {
        match self {
            SyncDirection::Pull => "pull",
            SyncDirection::Push => "push",
            SyncDirection::Both => "both",
        }
    }
}

impl FromStr for SyncDirection {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "pull" => Ok(Self::Pull),
            "push" => Ok(Self::Push),
            "both" => Ok(Self::Both),
            _ => Err(format!("invalid direction '{}'", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TraceConfig {
    #[serde(default)]
    pub repo_name: String,
    #[serde(default)]
    pub owner: String,
    #[serde(default)]
    pub default_user: Option<String>,
    #[serde(default)]
    pub default_source: Option<String>,
    #[serde(default)]
    pub sources: Vec<TraceSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceSource {
    pub id: String,
    pub kind: SourceKind,
    pub url: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub owner_user: Option<String>,
    #[serde(default)]
    pub database: Option<String>,
    #[serde(default)]
    pub namespace: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub password_env: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceNote {
    pub id: String,
    pub title: String,
    pub body: String,
    pub layer: TraceLayer,
    pub scope: TraceScope,
    pub target: String,
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub related_pr: Option<u64>,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub actor: Option<String>,
    pub status: NoteStatus,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SyncState {
    #[serde(default)]
    pub runs: Vec<SyncRun>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRun {
    pub timestamp: String,
    pub direction: SyncDirection,
    #[serde(default)]
    pub source_id: Option<String>,
    #[serde(default)]
    pub summary: String,
}

#[derive(Debug, Clone)]
pub struct PackageRisk {
    pub package: String,
    pub version: String,
    pub ecosystem: String,
    pub severity: String,
    pub vulnerability_id: String,
    pub source_file: String,
    pub installed_by: Option<String>,
}

// ─── Knowledge Graph types ───────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EntityKind {
    File,
    Package,
    Person,
    Decision,
    Endpoint,
    Module,
    Pr,
    Branch,
    Note,
    Vulnerability,
}

impl EntityKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            EntityKind::File => "file",
            EntityKind::Package => "package",
            EntityKind::Person => "person",
            EntityKind::Decision => "decision",
            EntityKind::Endpoint => "endpoint",
            EntityKind::Module => "module",
            EntityKind::Pr => "pr",
            EntityKind::Branch => "branch",
            EntityKind::Note => "note",
            EntityKind::Vulnerability => "vulnerability",
        }
    }
}

impl FromStr for EntityKind {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "file" => Ok(Self::File),
            "package" => Ok(Self::Package),
            "person" => Ok(Self::Person),
            "decision" => Ok(Self::Decision),
            "endpoint" => Ok(Self::Endpoint),
            "module" => Ok(Self::Module),
            "pr" => Ok(Self::Pr),
            "branch" => Ok(Self::Branch),
            "note" => Ok(Self::Note),
            "vulnerability" | "vuln" => Ok(Self::Vulnerability),
            _ => Err(format!("invalid entity kind '{}'", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RelationType {
    DependsOn,
    IntroducedBy,
    ModifiedBy,
    Affects,
    DecidedBy,
    RelatesTo,
    Supersedes,
    ConflictsWith,
    Documents,
    Exposes,
    Owns,
}

impl RelationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RelationType::DependsOn => "depends_on",
            RelationType::IntroducedBy => "introduced_by",
            RelationType::ModifiedBy => "modified_by",
            RelationType::Affects => "affects",
            RelationType::DecidedBy => "decided_by",
            RelationType::RelatesTo => "relates_to",
            RelationType::Supersedes => "supersedes",
            RelationType::ConflictsWith => "conflicts_with",
            RelationType::Documents => "documents",
            RelationType::Exposes => "exposes",
            RelationType::Owns => "owns",
        }
    }
}

impl FromStr for RelationType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "depends_on" => Ok(Self::DependsOn),
            "introduced_by" => Ok(Self::IntroducedBy),
            "modified_by" => Ok(Self::ModifiedBy),
            "affects" => Ok(Self::Affects),
            "decided_by" => Ok(Self::DecidedBy),
            "relates_to" => Ok(Self::RelatesTo),
            "supersedes" => Ok(Self::Supersedes),
            "conflicts_with" => Ok(Self::ConflictsWith),
            "documents" => Ok(Self::Documents),
            "exposes" => Ok(Self::Exposes),
            "owns" => Ok(Self::Owns),
            _ => Err(format!("invalid relation type '{}'", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KgEntity {
    pub id: String,
    pub kind: EntityKind,
    pub name: String,
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, String>,
    #[serde(default = "default_branch")]
    pub branch: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KgEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub relation: RelationType,
    #[serde(default = "default_weight")]
    pub weight: f64,
    #[serde(default = "default_branch")]
    pub branch: String,
    #[serde(default)]
    pub evidence: String,
    pub created_at: String,
}

fn default_branch() -> String {
    "main".to_string()
}

fn default_weight() -> f64 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KgGraph {
    pub entities: Vec<KgEntity>,
    pub edges: Vec<KgEdge>,
}
