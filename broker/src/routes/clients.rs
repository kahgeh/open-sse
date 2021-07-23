use actix_web::{post, Responder, web::{Data,Json}, HttpResponse};
use crate::contracts::SaveClientRequest;
use redis::aio::MultiplexedConnection;
use tracing::{error};

#[post("/clients")]
pub async fn save_client(save_request: Json<SaveClientRequest>
                         , redis_connection: Data<MultiplexedConnection>) -> impl Responder {
    let con = redis_connection.get_ref();
    let mut con = con.clone();

    match redis::cmd("SET")
        .arg(&[&save_request.client_id,&save_request.server_ip])
        .query_async::<_,()>(&mut con)
        .await {
        Ok(_) =>HttpResponse::Ok().finish(),
        Err(e)=>{
            error!("fail to set client {} because {:?}", save_request.client_id, e);
            HttpResponse::InternalServerError().finish()
        }
    }


}