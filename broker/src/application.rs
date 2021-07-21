use actix_web::{ App, HttpServer, web::{scope,Data}};
use tracing_actix_web::TracingLogger;
use derive_more::{Display, Error};

use app_ops::{utils::HttpSettings,AppOpsExt, HttpAppRootSpanBuilder};
use actix_web::dev::Server;

use crate::settings::{AppSettings};
use crate::application::StartUpError::{FailToStartHttpServer, FailToStartTcpListener};

#[derive(Debug, Display, Error)]
pub enum StartUpError {
    FailToStartTcpListener(std::io::Error),
    FailToStartHttpServer(std::io::Error),
}

pub struct Application {
    settings: HttpSettings,
}

impl Application {
    pub fn new(settings: HttpSettings) ->Application {
        Application {
            settings
        }
    }

    pub fn start(&self, app_settings:AppSettings) -> Result<Server, StartUpError>{
        let listener = match self.settings.create_listener() {
            Ok(l)=>l,
            Err(e)=> return Err(FailToStartTcpListener(e)),
        };

        let url_prefix = self.settings.url_prefix.clone();
        let app_settings = Data::new(app_settings);
        let server=HttpServer::new(move ||{
            App::new()
                .app_data(Data::clone(&app_settings))
                .app_data(Data::new(app_settings.settings.clone()))
                .app_data(Data::new(app_settings.runtime_info.clone()))
                .wrap(TracingLogger::<HttpAppRootSpanBuilder<AppSettings>>::new())
                .use_ops_endpoints()
                .service(
                    scope(url_prefix.as_str())
                )
        })
            .listen(listener).map_err(|e|FailToStartHttpServer(e))?;

        Ok(server.run())
    }
}

