use serde::Deserialize;

#[derive(Deserialize)]
pub struct SaveClientRequest {
    pub client_id: String,
    pub server_ip: String,
}
