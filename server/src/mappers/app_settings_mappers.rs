use crate::settings::AppSettings;
use app_ops::{ApplicationStartUpDisplayInfo, CreateLoggingRequest};

impl From<&AppSettings> for CreateLoggingRequest {
    fn from(app_settings: &AppSettings) -> Self {
        CreateLoggingRequest {
            app_name: app_settings.runtime_info.app_name.clone(),
            settings: app_settings.settings.logging.clone(),
        }
    }
}

impl From<&AppSettings> for ApplicationStartUpDisplayInfo {
    fn from(app_settings: &AppSettings) -> Self {
        ApplicationStartUpDisplayInfo::new(
            app_settings.settings.environment.name.as_str(),
            app_settings.settings.environment.debug,
            app_settings.settings.http.port,
        )
    }
}
