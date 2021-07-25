use actix_web::{ App, HttpServer, web::{scope,Data}};
use tracing_actix_web::TracingLogger;
use derive_more::{Display, Error};

use app_ops::{utils::HttpSettings, AppOpsExt, HttpAppRootSpanBuilder, GetAppInfoResponseBuild};
use actix_web::dev::Server;

use crate::settings::{AppSettings};
use crate::application::StartUpError::{FailToStartHttpServer, FailToStartTcpListener, FailToGetRedisConnection};
use tracing::error;
use crate::redis_helpers::open_redis;
use crate::clients_service::BrokerManagedClientsService;
use broker_protobufs::contracts_broker_v0_1_x::clients_server::ClientsServer;
use std::net::SocketAddr;
use redis::aio::MultiplexedConnection;
use tokio::task::JoinHandle;

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

async fn start_grpc_server(addr: SocketAddr, redis_connection_grpc: Data<MultiplexedConnection>)->std::io::Result<()>{
    let result = tonic::transport::Server::builder()
        .add_service(ClientsServer::new(
            BrokerManagedClientsService::new(redis_connection_grpc)))
        .serve(addr).await;

    if let Err(e) = result {
        error!("gRPC server stopped with error {:?}", e);
    }
    Ok(())
}

impl Application {
    pub fn new(settings: HttpSettings) ->Application {
        Application {
            settings
        }
    }

    pub async fn start(&self, app_settings:AppSettings) -> Result<(Server,JoinHandle<tokio::io::Result<()>>), StartUpError> {
        let listener = match self.settings.create_listener() {
            Ok(l) => l,
            Err(e) => return Err(FailToStartTcpListener(e)),
        };

        let url_prefix = self.settings.url_prefix.clone();
        let app_settings = Data::new(app_settings);
        let grpc_endpoint = format!("0.0.0.0:{}",app_settings.settings.grpc.port);
        let grpc_addr = grpc_endpoint.parse().unwrap();

        let app_info_response_build: Data<GetAppInfoResponseBuild> =
            Data::new(app_settings.to_get_app_info_response_build());
        let redis_connection_string = app_settings.settings.outgoing_endpoints.redis.as_str();
        let redis_client = open_redis(redis_connection_string)?;

        let redis_connection = match redis_client.get_multiplexed_async_connection().await {
            Ok(connection) => connection,
            Err(e) => {
                error!("Fail to get redis connection {:?}", redis_connection_string);
                return Err(FailToGetRedisConnection(e));
            }
        };
        let redis_connection = Data::new(redis_connection);
        let redis_connection_grpc = Data::clone(&redis_connection);
        let http_server = HttpServer::new(move || {
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
            .listen(listener).map_err(|e| FailToStartHttpServer(e))?;

        let grpc_server = tokio::spawn(start_grpc_server(grpc_addr, redis_connection_grpc));

        Ok((http_server.run(), grpc_server))
    }
}

