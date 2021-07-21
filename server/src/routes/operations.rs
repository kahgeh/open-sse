use actix_web::{post, Responder, HttpResponse};
use actix_web::web::{Data};
use crate::sse_exchange::{SseExchange};

#[post("/terminate")]
pub async fn terminate(sse_exchange: Data<SseExchange>)-> impl Responder {
    match sse_exchange.stop().await {
        Ok(_)=> HttpResponse::Ok().finish(),
        Err(_)=>HttpResponse::InternalServerError().finish()
    }
}