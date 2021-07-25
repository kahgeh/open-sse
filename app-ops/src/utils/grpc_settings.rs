use serde::{Deserialize};

#[derive(Debug, Deserialize, Clone)]
pub struct GrpcSettings {
    pub port: u16,
}

