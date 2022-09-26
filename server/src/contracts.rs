use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(crate) struct HelloResponse {
    pub message: String,
}
