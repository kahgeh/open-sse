use actix_web::{get, post, Responder, HttpResponse, web::{Data}, HttpRequest};
use tracing::{error, warn};
use actix_web::web::Bytes;
use actix_web_utils::{get_header};
use broker_protobufs::contracts_broker_v0_1_x::clients_client::ClientsClient;
use crate::settings::Settings;
use crate::sse_exchange::{SseExchange, Event};
use ginepro::LoadBalancedChannel;
use broker_protobufs::contracts_broker_v0_1_x::SaveClientRequest;
use app_ops::RuntimeInfo;
use std::sync::Mutex;

fn convert_ipv4_string_to_4bytes(ipv4_string:&str)->Vec<u8>{
    let parts= ipv4_string.split(".").collect::<Vec<&str>>();

    let mut bytes:Vec<u8> = vec![];
    for part in parts {
        bytes.push(part.parse::<u8>().unwrap())
    }
    bytes
}

#[get("/clients/{client_id}/events")]
pub async fn receive_connect_request(req: HttpRequest,
                                     clients_client: Data<Mutex<ClientsClient<LoadBalancedChannel>>>,
                                     runtime_info: Data<RuntimeInfo>,
                                     sse_exchange: Data<SseExchange>)-> impl Responder {
    let client_id = req.match_info().query("client_id");

    let save_client_request = SaveClientRequest {
        client_id: client_id.to_string(),
        server_ipv4_addr:  convert_ipv4_string_to_4bytes(runtime_info.ip_address.as_str()).into()
    };
    {
        let mut clients_client = clients_client.lock().unwrap();
        match clients_client.save(save_client_request).await {
            Ok(_)=>{},
            Err(e)=>{

                error!("fail to save client into client-register, {:?}", e);
                return HttpResponse::InternalServerError()
                    .finish();
            }
        }
    }

    match (*sse_exchange).connect(client_id).await {
        Ok(rx)=>{
            HttpResponse::Ok()
                .append_header(("content-type", "text/event-stream"))
                .append_header(("cache-control", "no-cache"))
                .append_header(("connection", "keep-alive"))
                .append_header(("access-control-allow-origin", "*"))
                .streaming(rx)
        },
        Err(_)=> {
            error!("fail to establish connection");
            HttpResponse::InternalServerError()
                .finish()
        }
    }

}

#[post("/clients/{client_id}/events")]
pub async fn receive_send_request(req: HttpRequest,
                                  body: Bytes,
                                  settings: Data<Settings>,
                                  sse_exchange: Data<SseExchange>) -> impl Responder {
    let client_id = req.match_info().query("client_id");
    let correlation_id_header_key=settings.logging.correlation_id_http_header.clone();
    let correlation_id = get_header!(req, &correlation_id_header_key);

    let result_converting_body_to_string = String::from_utf8(body.to_vec());

    if result_converting_body_to_string.is_err() {
        error!("there is an issue with the payload");
        return HttpResponse::BadRequest()
            .finish();
    }

    let payload = result_converting_body_to_string.unwrap();

    if !sse_exchange.publish(Event::new(client_id, correlation_id,payload.as_str())).await {
        error!("fail to send events");
        return HttpResponse::InternalServerError()
            .finish();
    }
    HttpResponse::Ok().finish()
}