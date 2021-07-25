use crate::application::{Application};
use app_ops::{ApplicationStartUpDisplayInfo, LoggingBuilder};
use tracing::{debug, error, info};
use tokio::{select, signal::{ctrl_c}};
use crate::settings::AppSettings;
use crate::RunError::{StartUp, NotReady};
use serde::{Deserialize};
use crate::readiness_checks::is_ready;

mod settings;
mod application;
mod mappers;
mod readiness_checks;
mod redis_helpers;
mod clients_service;

#[derive(Debug, Deserialize, Clone)]
pub enum RunError {
    NotReady,
    StartUp,
    Run,
}

#[actix_web::main]
async fn main()-> Result<(), RunError> {
    let app_settings = AppSettings::load();

    LoggingBuilder::new((&app_settings).into())
        .init_default();

    if app_settings.settings.readiness.check {
        if !is_ready(&app_settings) {
            return Err(NotReady);
        }
        return Ok(());
    }

    debug!("app settings loaded {:?}", app_settings);

    let ApplicationStartUpDisplayInfo{ environment_name, is_debug, port} = (&app_settings).into();
    info!(Environment=&environment_name[..], IsDebug=&is_debug[..], Port=&port[..], "Application started");

    let (http_server, grpc_server_task)= match Application::new(app_settings.settings.http.clone())
        .start(app_settings.clone()).await {
        Ok(services)=>services,
        Err(e)=>{
            error!("fail to start services {:?}", e);
            return Err(StartUp);
        },
    };

    let http_server_task = tokio::spawn(http_server);
    select! {
        _ = http_server_task => {
            info!("http server stopped");
        }
        _ = grpc_server_task => {
            info!("grpc server stopped");
        }
        _ = ctrl_c() => {
            info!("application terminated because of cancellation signal ctrl+c");
        }
    };

    Ok(())
}
