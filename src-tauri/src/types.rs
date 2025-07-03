use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub username: String,
    pub password: String,
    pub server_url: String,
    pub folder_path: String,
    pub token: String,
    pub refresh_token: String,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            username: "".to_string(),
            password: "".to_string(),
            server_url: "http://localhost:3000".to_string(),
            folder_path: "D:\\python\\codes\\fiverr\\testfolder\\f1".to_string(),
            token: "".to_string(),
            refresh_token: "".to_string(),
        }
    }
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transfer {
    pub r#type: TransferType,
    pub state: TransferState,
    pub progress: u32,
    pub path: String,
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransferType {
    Upload,
    Download,
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransferState {
    Active,
    Completed,
}
