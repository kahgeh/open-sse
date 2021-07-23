use crate::application::{Application};
use app_ops::{ApplicationStartUpDisplayInfo, LoggingBuilder};
use tracing::{debug, error, info};
use crate::settings::AppSettings;
use crate::RunError::{StartUp, Run};
use serde::{Deserialize};
mod settings;
mod application;
mod mappers;
mod contracts;
mod routes;

#[derive(Debug, Deserialize, Clone)]
pub enum RunError {
    StartUp,
    Run,
}

#[actix_web::main]
async fn main()-> Result<(), RunError> {
    let app_settings = AppSettings::load();

    LoggingBuilder::new((&app_settings).into())
        .init_default();

    debug!("app settings loaded {:?}", app_settings);

    let ApplicationStartUpDisplayInfo{ environment_name, is_debug, port} = (&app_settings).into();
    info!(Environment=&environment_name[..], IsDebug=&is_debug[..], Port=&port[..], "Application started");

    let http_server= match Application::new(app_settings.settings.http.clone())
        .start(app_settings.clone()).await {
        Ok(services)=>services,
        Err(e)=>{
            error!("fail to start services {:?}", e);
            return Err(StartUp);
        },
    };

    match http_server.await {
        Ok(_) =>Ok(()),
        Err(e)=> {
            error!("server ran to an error {:?}", e);
            Err(Run)
        }
    }
}
