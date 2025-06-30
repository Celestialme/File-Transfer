use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub username: String,
    pub password: String,
    pub server_url: String,
    pub folder_path: String,
    pub token: String,
    pub refresh_token: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transfer {
    pub r#type: String,
    pub state: String,
    pub progress: u32,
    pub path: String,
}
