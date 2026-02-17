use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum AgentKind {
    Codex,
    Claude,
}

impl AgentKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum InstallTarget {
    Codex,
    Claude,
    Both,
    Skip,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillBrief {
    pub name: String,
    pub description: String,
    pub overview: String,
    #[serde(default)]
    pub sections: Vec<SkillSection>,
    #[serde(default)]
    pub scripts: Vec<ResourceEntry>,
    #[serde(default)]
    pub references: Vec<ResourceEntry>,
    #[serde(default)]
    pub assets: Vec<ResourceEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSection {
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceEntry {
    pub path: String,
    #[serde(default)]
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RootConfigFile {
    pub root: Option<String>,
    pub default_agent: Option<AgentKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InstallState {
    pub version: u32,
    pub records: Vec<InstallRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallRecord {
    pub skill_name: String,
    pub alias: String,
    pub codex: Option<InstallBinding>,
    pub claude: Option<InstallBinding>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallBinding {
    pub install_path: String,
    pub render_path: String,
}
