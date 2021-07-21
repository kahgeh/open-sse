use actix_web::{get, post, Responder, HttpResponse};
use actix_web::web::{Data};
use crate::sse_exchange::{SseExchange};
use crate::contracts::GetStatisticResponse;

#[get("/stats")]
pub async fn get_stats(sse_exchange: Data<SseExchange>)-> impl Responder {
    HttpResponse::Ok().json(GetStatisticResponse{
        client_count: sse_exchange.get_client_count().await.unwrap_or_else(|_|0)
    })
}

#[post("/noop")]
pub async fn noop(sse_exchange: Data<SseExchange>)-> impl Responder {
    match sse_exchange.wait().await {
        Ok(_)=> HttpResponse::Ok().finish(),
        Err(_)=>HttpResponse::InternalServerError().finish()
    }
}

