mod settings;
mod mappers;
mod application;
mod sse_exchange;
mod routes;
mod contracts;

use tracing::{debug, info, error};
use tokio::{select, signal::{ctrl_c}};
use futures::future::{join_all};
use crate::settings::{AppSettings};

use app_ops::{LoggingBuilder, ApplicationStartUpDisplayInfo};
use crate::application::{Application, StartUpError};

#[actix_web::main]
async fn main()-> Result<(), StartUpError> {
    let app_settings = AppSettings::load();

    LoggingBuilder::new((&app_settings).into())
        .init_default();

    debug!("app settings loaded {:?}", app_settings);

    let ApplicationStartUpDisplayInfo{ environment_name, is_debug, port} = (&app_settings).into();
    info!(Environment=&environment_name[..], IsDebug=&is_debug[..], Port=&port[..], "Application started");

    let (http_server, sse_exchange_task)= match Application::new(app_settings.settings.http.clone())
        .start(app_settings.clone()){
            Ok(services)=>services,
            Err(e)=>{
                error!("Fail to start services {:?}", e);
                return Err(e);
            },
        };

    let services_task = join_all(vec![tokio::spawn(http_server),sse_exchange_task]) ;
    select! {
        _ = services_task => {
            info!("services stopped");
        }
        _ = ctrl_c() => {
            info!("application terminated because of cancellation signal ctrl+c");
        }
    };

    Ok(())
}