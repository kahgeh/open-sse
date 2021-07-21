use serde::Serialize;

#[derive(Serialize)]
pub struct GetStatisticResponse {
    pub client_count: u32,
}