use std::net::TcpListener;
use std::io::Error;
use serde::{Deserialize};

#[derive(Debug, Deserialize, Clone)]
pub struct HttpSettings {
    pub port: u16,
    pub url_prefix: String,
    pub allowed_origin: String,
}

impl HttpSettings {
    pub fn create_listener(&self) -> Result<TcpListener, Error> {
        let address = format!("0.0.0.0:{}", self.port);
        TcpListener::bind(&address)
    }
}