use std::rc::Rc;

use tracing::{Span,error};
use tracing_subscriber::{EnvFilter, Registry};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::layer::SubscriberExt;
use tracing_actix_web::{root_span, RootSpanBuilder, DefaultRootSpanBuilder};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{ Error};
use actix_web::web::Data;
use std::marker::PhantomData;
use serde::{Deserialize};

#[derive(Debug, Deserialize, Clone)]
pub struct LogSettings {
    pub level: String,
    pub correlation_id_http_header: String,
}

pub trait CommonLogAttributes {
    fn get_commit_id(&self)->String;
    fn get_correlation_header_name(&self)->String;
}

pub struct HttpAppRootSpanBuilder<T> {
    phantom: PhantomData<T>
}

impl<T:'static + CommonLogAttributes> RootSpanBuilder for HttpAppRootSpanBuilder<T> {
    fn on_request_start(request: &ServiceRequest) -> Span {
        let settings = match request.app_data::<Data<T>>() {
            Some(s)=>s.clone(),
            None => {
                error!("expected settings was not available");
                return root_span!(request);
            }
        };

        let header_value=match request.headers().get(settings.get_correlation_header_name()){
            Some(header)=>String::from(header.to_str().unwrap()),
            _=>String::from("none")
        };

        let git_commit_id = settings.get_commit_id();
        root_span!(
            request,
            correlation_id=header_value.as_str(),
            git_commit_id=git_commit_id.as_str())
    }

    fn on_request_end<B>(span: Span, outcome: &Result<ServiceResponse<B>, Error>) {
        DefaultRootSpanBuilder::on_request_end(span, outcome);
    }
}

pub struct CreateLoggingRequest {
    pub app_name: String,
    pub settings: LogSettings,
}

pub struct LoggingBuilder {
    settings: Rc<LogSettings>,
    app_name : String,
}

impl LoggingBuilder {
    pub fn new(request: CreateLoggingRequest) ->LoggingBuilder {
        LoggingBuilder {
            settings: Rc::new(request.settings),
            app_name: request.app_name,
        }
    }

    pub fn init_default(&self){
        let settings=Rc::clone(&self.settings);
        let app_name = &self.app_name[..];
        let log_level = settings.level.clone();
        let filter = EnvFilter::from(log_level);

        let formatting_layer = BunyanFormattingLayer::new(
            String::from(app_name),
            std::io::stdout);

        let subscriber = Registry::default()
            .with(filter)
            .with(JsonStorageLayer)
            .with(formatting_layer);

        tracing::subscriber::set_global_default(subscriber)
            .expect("Failed to install `tracing` subscriber.")
    }

}

