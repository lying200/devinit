//! 项目信息
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ProjectContext {
    pub project_name: String,
    pub project_path: String,
    pub language: Language,
    // 项目依赖服务，如 pg、redis 等
    pub services: Vec<Service>,
    // 项目依赖工具，如 git 等
    pub tools: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Rust,
    Python,
    Go,
    Java,
    Nodejs,
}

#[derive(Debug, Clone, Serialize)]
pub struct Service {
    pub name: String,
    pub version: Option<String>,
}
