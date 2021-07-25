use actix_web::web::Data;
use redis::aio::MultiplexedConnection;
use tracing::{error};
use broker_protobufs::contracts_broker_v0_1_x::{SaveClientRequest,SaveResponse, clients_server::Clients};
use tonic::{Response, Request, Status, Code::{Internal}};

fn get_ip_address_string(raw:&Vec<u8>)->String{
    format!("{}.{}.{}.{}", raw[0], raw[1], raw[2], raw[3])
}

pub struct BrokerManagedClientsService {
    redis_connection: Data<MultiplexedConnection>
}

impl BrokerManagedClientsService {
    pub fn new(redis_connection: Data<MultiplexedConnection>)->Self{
        BrokerManagedClientsService {
            redis_connection
        }
    }
}
#[tonic::async_trait]
impl Clients for BrokerManagedClientsService{
    async fn save(&self, request: Request<SaveClientRequest>)->Result<Response<SaveResponse>,Status> {
        let con = self.redis_connection.get_ref();
        let mut con = con.clone();

        let client_id = &request.get_ref().client_id;
        let server_ip = get_ip_address_string(&request.get_ref().server_ipv4_addr);
        match redis::cmd("SET")
            .arg(&[client_id, &server_ip])
            .query_async::<_,()>(&mut con)
            .await {
            Ok(_) =>Ok(Response::new(SaveResponse{
                success: false,
                message: "saved".into(),
            })),
            Err(e)=>{
                error!("fail to set client {} because {:?}", client_id, e);
                Err(Status::new(Internal, "failed"))
            }
        }
    }
}
