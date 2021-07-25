use actix_web::{ App, HttpServer, web::{scope,Data}};
use tracing_actix_web::TracingLogger;
use derive_more::{Display, Error};

use app_ops::{utils::HttpSettings, AppOpsExt, HttpAppRootSpanBuilder, GetAppInfoResponseBuild};
use actix_web_utils::create_cors_policy;
use actix_web::dev::Server;
use tokio::task::JoinHandle;

use crate::settings::{AppSettings};
use crate::application::StartUpError::{FailToStartHttpServer, FailToStartTcpListener, FailToStartUpBrokerConnection};
use crate::sse_exchange::SseExchange;
use crate::routes::*;
use ginepro::LoadBalancedChannel;
use tracing::error;
use broker_protobufs::contracts_broker_v0_1_x::clients_client::ClientsClient;
use std::sync::Mutex;

#[derive(Debug, Display, Error)]
pub enum StartUpError {
    FailToStartTcpListener(std::io::Error),
    FailToStartHttpServer(std::io::Error),
    FailToStartUpBrokerConnection
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

    pub async fn start(&self, app_settings:AppSettings) -> Result<(Server, JoinHandle<tokio::io::Result<()>>), StartUpError>{
        let listener = match self.settings.create_listener() {
            Ok(l)=>l,
            Err(e)=> return Err(FailToStartTcpListener(e)),
        };

        let url_prefix = self.settings.url_prefix.clone();
        let app_settings = Data::new(app_settings);
        let (sse_exchange_task, sse_exchange) = SseExchange::start();
        let sse_exchange = Data::new(sse_exchange);
        let endpoint = app_settings.settings.outgoing_endpoints.broker.split(":").collect::<Vec<&str>>();
        let lb_channel = match LoadBalancedChannel::builder(
                (endpoint[0], endpoint[1].parse::<u16>().unwrap())
            )
            .await {
            Ok(v)=>v.channel(),
            Err(e)=>{
                error!("fail to initialise dns resolver for connection to broker {:?}", e);
                return Err(FailToStartUpBrokerConnection);
            }
        };
        let clients_client = Data::new(Mutex::new(ClientsClient::new(lb_channel)));

        let app_info_response_build:Data<GetAppInfoResponseBuild> =
            Data::new(app_settings.to_get_app_info_response_build());
        let server=HttpServer::new(move ||{
            App::new()
                .app_data(Data::clone(&app_settings))
                .app_data(Data::new(app_settings.settings.clone()))
                .app_data(Data::new(app_settings.runtime_info.clone()))
                .app_data(Data::clone(&app_info_response_build))
                .app_data(sse_exchange.clone())
                .app_data(clients_client.clone())
                .wrap(create_cors_policy(&app_settings.settings.http.allowed_origin))
                .wrap(TracingLogger::<HttpAppRootSpanBuilder<AppSettings>>::new())
                .use_ops_endpoints()
                .service(
                    scope(url_prefix.as_str())
                        .service(receive_connect_request)
                        .service(receive_send_request)
                        .service(get_stats)
                        .service(noop)
                )
        })
            .listen(listener).map_err(|e|FailToStartHttpServer(e))?;

        Ok((server.run(),sse_exchange_task))
    }
}

