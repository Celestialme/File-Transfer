use std::time::SystemTime;

use crate::synchronizer::fstree;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Token {
    pub value: String,
    pub created_at: SystemTime,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub username: Option<String>,
    pub password: Option<String>,
    pub server_url: String,
    pub folder_path: String,
    pub token: Option<Token>,
    pub refresh_token: Option<Token>,
    pub is_configured: bool,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            username: None,
            password: None,
            server_url: "http://localhost:3000".to_string(),
            folder_path: "D:\\python\\codes\\fiverr\\testfolder\\f1".to_string(),
            token: None,
            refresh_token: None,
            is_configured: false,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SocketResponse {
    pub message: String,
    pub data: fstree::Node,
    pub timestamp: u64,
    pub r#type: String,
}
