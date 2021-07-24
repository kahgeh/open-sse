use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ReadinessSettings{
    pub check: bool,
    pub retry_count: u8,
    pub retry_interval_in_seconds: u16,
}
