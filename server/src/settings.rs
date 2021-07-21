use serde::{Deserialize};
use app_ops::{utils::HttpSettings,RuntimeInfo, CommonLogAttributes, LogSettings, load_settings};

const APP_NAME: &str="open-sse-server";
const APP_ENV_PREFIX: &str="SSE_";

#[derive(Debug, Deserialize, Clone)]
pub struct AppSettings {
    pub settings: Settings,
    pub runtime_info: RuntimeInfo,
}

impl AppSettings {
    pub fn load() -> AppSettings {
        AppSettings {
            settings: load_settings(APP_ENV_PREFIX, APP_NAME).expect("fail to load settings"),
            runtime_info: RuntimeInfo::new(APP_NAME),
        }
    }
}

impl CommonLogAttributes for AppSettings {
    fn get_commit_id(&self) -> String {
        self.runtime_info.git_commit_id.clone()
    }

    fn get_correlation_header_name(&self) -> String {
        self.settings.logging.correlation_id_http_header.clone()
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub environment: ExecutionEnvironment,
    pub http: HttpSettings,
    pub logging: LogSettings,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ExecutionEnvironment {
    pub name: String,
    pub debug: bool,
}
