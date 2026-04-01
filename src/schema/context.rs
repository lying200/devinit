//! 项目信息
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ProjectContext {
    pub languages: Vec<Language>,
    pub services: Vec<Service>,
    // TODO: 尚未实现自动检测，当前始终为空
    pub tools: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase", tag = "name")]
pub enum Language {
    Rust {
        #[serde(skip_serializing_if = "Option::is_none")]
        channel: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        version: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        components: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        targets: Option<Vec<String>>,
    },
    Python {
        #[serde(skip_serializing_if = "Option::is_none")]
        version: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        package: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        uv_enable: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        venv_enable: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        venv_quiet: Option<bool>,
    },
    Go {
        #[serde(skip_serializing_if = "Option::is_none")]
        version: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        package: Option<String>,
    },
    Java {
        #[serde(skip_serializing_if = "Option::is_none")]
        jdk_package: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        gradle_enable: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        maven_enable: Option<bool>,
    },
    #[serde(rename = "javascript")]
    JavaScript {
        #[serde(skip_serializing_if = "Option::is_none")]
        package: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        package_manager: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        corepack_enable: Option<bool>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase", tag = "name")]
pub enum Service {
    Postgres {
        #[serde(skip_serializing_if = "Option::is_none")]
        package: Option<String>,
    },
    Redis,
    Mysql {
        #[serde(skip_serializing_if = "Option::is_none")]
        package: Option<String>,
    },
}
