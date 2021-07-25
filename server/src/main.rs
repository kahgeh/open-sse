mod settings;
mod mappers;
mod application;
mod sse_exchange;
mod routes;
mod contracts;
mod readiness_checks;

use tracing::{debug, info, error};
use tokio::{select, signal::{ctrl_c}};
use crate::settings::{AppSettings};

use app_ops::{LoggingBuilder, ApplicationStartUpDisplayInfo};
use crate::application::{Application};
use crate::RunError::{NotReady, StartUp};
use serde::Deserialize;
use crate::readiness_checks::is_ready;

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

    debug!("app settings loaded {:?}", app_settings);

    if app_settings.settings.readiness.check {
        if !is_ready() {
            return Err(NotReady);
        }
        return Ok(());
    }

    let ApplicationStartUpDisplayInfo{ environment_name, is_debug, port} = (&app_settings).into();
    info!(Environment=&environment_name[..], IsDebug=&is_debug[..], Port=&port[..], "Application started");

    let (http_server, sse_exchange_task)= match Application::new(app_settings.settings.http.clone())
        .start(app_settings.clone()).await {
            Ok(services)=>services,
            Err(e)=>{
                error!("Fail to start services {:?}", e);
                return Err(StartUp);
            },
        };

    let http_server_task = tokio::spawn(http_server) ;
    select! {
        _ = http_server_task => {
            info!("http server stopped");
        }
        _ =sse_exchange_task => {
            info!("sse server stopped");
        }
        _ = ctrl_c() => {
            info!("application terminated because of cancellation signal ctrl+c");
        }
    };

    Ok(())
}