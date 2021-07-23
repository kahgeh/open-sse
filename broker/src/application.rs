use actix_web::{ App, HttpServer, web::{scope,Data}};
use tracing_actix_web::TracingLogger;
use derive_more::{Display, Error};

use app_ops::{utils::HttpSettings, AppOpsExt, HttpAppRootSpanBuilder, GetAppInfoResponseBuild};
use actix_web::dev::Server;

use crate::settings::{AppSettings};
use crate::application::StartUpError::{FailToStartHttpServer, FailToStartTcpListener, FailToGetRedisClient, FailToGetRedisConnection};
use tracing::error;

#[derive(Debug, Display, Error)]
pub enum StartUpError {
    FailToStartTcpListener(std::io::Error),
    FailToStartHttpServer(std::io::Error),
    FailToGetRedisClient(redis::RedisError),
    FailToGetRedisConnection(redis::RedisError),
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

    pub async fn start(&self, app_settings:AppSettings) -> Result<Server, StartUpError>{
        let listener = match self.settings.create_listener() {
            Ok(l)=>l,
            Err(e)=> return Err(FailToStartTcpListener(e)),
        };

        let url_prefix = self.settings.url_prefix.clone();
        let app_settings = Data::new(app_settings);
        let app_info_response_build:Data<GetAppInfoResponseBuild> =
            Data::new(app_settings.to_get_app_info_response_build());
        let redis_connection_string=app_settings.settings.outgoing_endpoints.redis.as_str();
        let redis_client = match redis::Client::open(redis_connection_string){
            Ok(client)=>client,
            Err(e)=> {
                error!("Fail to create a client to {} ( error - {:?} )",
                    &app_settings.settings.outgoing_endpoints.redis,
                    e);
                return Err(FailToGetRedisClient(e));
            }
        };

        let redis_connection= match redis_client.get_multiplexed_async_connection().await{
            Ok(connection)=>connection,
            Err(e)=>{
                error!("Fail to get redis connection");
                return Err(FailToGetRedisConnection(e));
            }
        };
        let redis_connection = Data::new(redis_connection);
        let server=HttpServer::new(move ||{
            App::new()
                .app_data(Data::clone(&app_settings))
                .app_data(Data::new(app_settings.settings.clone()))
                .app_data(Data::new(app_settings.runtime_info.clone()))
                .app_data(Data::clone(&app_info_response_build))
                .app_data(Data::clone(&redis_connection))
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

