use serde::{Deserialize};
use app_ops::{utils::HttpSettings, RuntimeInfo, CommonLogAttributes, LogSettings, load_settings, AppInfoResponseCase, GetAppInfoResponseBuild};

const APP_NAME: &str="open-sse-server";
const APP_ENV_PREFIX: &str="sse";

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
    pub fn to_get_app_info_response_build(&self) ->GetAppInfoResponseBuild{
        GetAppInfoResponseBuild {
            response: (&self.runtime_info).into(),
            json_case: self.settings.app_info_response_case.clone()
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
    pub app_info_response_case: AppInfoResponseCase,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ExecutionEnvironment {
    pub name: String,
    pub debug: bool,
}
