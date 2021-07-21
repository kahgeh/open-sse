use std::time::{SystemTime};
use std::env;
use time::{format_description,OffsetDateTime};
use actix_web::{Responder, HttpResponse, App, web};
use actix_web::web::Data;
use serde::{Serialize,Deserialize};
use actix_web::body::{MessageBody};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::error::Error;
use actix_service::{ServiceFactory};
use crate::utils::get_one_host_ip_address;


pub const DATE_ISO_FORMAT:&str="[year]-[month]-[day] [hour]:[minute]:[second]";

#[derive(Serialize)]
pub struct GetAppInfoResponse {
    pub app_name: String,
    pub git_commit_id: String,
    pub started : String,
    pub current_time : String,
    pub ip_address: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RuntimeInfo {
    pub app_name: String,
    pub git_commit_id: String,
    pub started : String,
    pub ip_address: String,
}

pub trait AppInfoSettings {
    fn get_app_info_response(&self)->GetAppInfoResponse;
}

impl RuntimeInfo {
    pub fn new(app_name: &str) -> RuntimeInfo {
        let git_commit_id=match env::var("LAST_COMMIT_SHA") {
            Ok(sha) => sha,
            _ => String::from("local-dev")
        };

        RuntimeInfo {
            app_name: String::from(app_name),
            git_commit_id,
            started: format_date_time(SystemTime::now(), DATE_ISO_FORMAT),
            ip_address: get_one_host_ip_address(),
        }
    }
}

pub fn format_date_time<T>(dt: T, format: &str) -> String
    where T: Into<OffsetDateTime>
{
    let format =  format_description::parse(format).unwrap();
    dt.into().format(&format).unwrap()
}

pub async fn ping() -> impl Responder {
    format!("application running\n")
}

pub async fn app_info(runtime_info:Data<RuntimeInfo>)-> impl Responder {
    HttpResponse::Ok().json(GetAppInfoResponse{
        app_name: runtime_info.app_name.clone(),
        git_commit_id: runtime_info.git_commit_id.clone(),
        started: runtime_info.started.clone(),
        current_time: format_date_time(SystemTime::now(),DATE_ISO_FORMAT),
        ip_address: runtime_info.ip_address.clone(),
    })
}

pub trait AppOpsExt<T,B> {
    fn use_ops_endpoints(self)->App<T, B>;
}

impl<T,B> AppOpsExt<T,B> for App<T, B>
    where
        B: MessageBody,
        T: ServiceFactory<
            ServiceRequest,
            Config = (),
            Response = ServiceResponse<B>,
            Error = Error,
            InitError = (),
        >,{

    fn use_ops_endpoints(self)->App<T, B>{
        self
            .route("/ping", web::get().to(ping))
            .route("/appinfo", web::get().to(app_info))
    }
}
