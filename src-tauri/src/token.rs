use std::{
    sync::Mutex,
    time::{Duration, SystemTime},
};
use tauri::AppHandle;
use tokio::sync::watch;

use crate::{logout, types::Token, update_config, CONFIG};
const REFRESH_TOKEN_EXPIRES: u64 = 24 * 60 * 60; // 1day
const TOKEN_EXPIRES: u64 = 15 * 60; // half hour
static TOKEN_WATCH_STOP: Mutex<Option<watch::Sender<bool>>> = Mutex::new(None);

pub fn stop() {
    if let Some(sender) = TOKEN_WATCH_STOP.lock().unwrap().as_mut() {
        let _ = sender.send(true);
    }
}
pub async fn watch_tokens(app: AppHandle) {
    let (tx, mut rx) = watch::channel(false);
    if let Some(sender) = TOKEN_WATCH_STOP.lock().unwrap().as_mut() {
        let _ = sender.send(true);
    }
    TOKEN_WATCH_STOP.lock().unwrap().replace(tx);
    let _app = app.clone();
    let refresh_token_task = async move {
        loop {
            let refresh_token = { CONFIG.lock().unwrap().refresh_token.clone() };
            let elsaped_time = refresh_token
                .unwrap()
                .created_at
                .elapsed()
                .unwrap()
                .as_secs();
            let sleep_time = REFRESH_TOKEN_EXPIRES.saturating_sub(elsaped_time);
            tokio::time::sleep(Duration::from_secs(sleep_time)).await;
            let refresh_token = { CONFIG.lock().unwrap().refresh_token.clone() };
            let elsaped_time = refresh_token
                .unwrap()
                .created_at
                .elapsed()
                .unwrap()
                .as_secs();
            let sleep_time = REFRESH_TOKEN_EXPIRES.saturating_sub(elsaped_time);
            if sleep_time == 0 {
                break;
            }
        }
        logout(_app.clone()).await;
    };

    let token_task = async move {
        loop {
            let mut config = { CONFIG.lock().unwrap().clone() };
            if config.token.is_none() || config.refresh_token.is_none() {
                break;
            }
            let token = config.token.as_ref().unwrap();
            let refresh_token = config.refresh_token.as_ref().unwrap();
            let elsaped_time = token.created_at.elapsed().unwrap().as_secs();
            let sleep_time = TOKEN_EXPIRES.saturating_sub(elsaped_time);
            tokio::time::sleep(Duration::from_secs(sleep_time)).await;
            println!("REFRESHING TOKEN");

            let client = reqwest::Client::new();

            let server = config.server_url.to_owned();
            let resp = client
                .post(format!("{server}/users/auth/keep-alive"))
                .header("x-refresh-token", &refresh_token.value)
                .send()
                .await
                .unwrap();
            let token = resp
                .headers()
                .get("authorization")
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();
            let refresh_token = resp
                .headers()
                .get("x-refresh-token")
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();

            config.token.replace(Token {
                value: token,
                created_at: SystemTime::now(),
            });

            config.refresh_token.replace(Token {
                value: refresh_token,
                created_at: SystemTime::now(),
            });

            let _ = update_config(app.clone(), config, true).await;
        }
    };
    tokio::select! {_ = rx.changed()=>{},
    _ = refresh_token_task=>{println!("Refresh token expired;logging out")},
    _= token_task=>{}};
}
