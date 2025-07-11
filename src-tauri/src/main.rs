// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod synchronizer;
mod token;
mod types;
mod windows;
use std::{
    collections::HashMap,
    path::Path,
    sync::{LazyLock, Mutex},
    time::SystemTime,
};

use reqwest::Client;
use serde_json::json;
use tauri::{
    AppHandle, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu,
    SystemTrayMenuItem,
};

use crate::{
    synchronizer::TRANSFERS,
    types::{Config, Token, TransferState},
};
static CONFIG: LazyLock<Mutex<Config>> = LazyLock::new(|| Mutex::new(Config::default()));

fn create_tray_menu() -> SystemTrayMenu {
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let show = CustomMenuItem::new("show".to_string(), "Show");
    let settings = CustomMenuItem::new("settings".to_string(), "Configuración");
    SystemTrayMenu::new()
        .add_item(show)
        .add_item(settings)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit)
}

#[tokio::main]
async fn main() {
    let system_tray = SystemTray::new().with_menu(create_tray_menu());
    tauri::Builder::default()
        .setup(|app| {
            std::fs::create_dir_all(app.path_resolver().app_data_dir().unwrap()).unwrap();
            set_config(app.handle());
            let config = CONFIG.lock().unwrap().clone();
            if !config.is_configured {
                windows::open_initial_configuration_window(app.handle());
            } else if config.username.is_none()
                || config.password.is_none()
                || config.token.is_none()
                || config.refresh_token.is_none()
            {
                windows::open_login_window(app.handle());
            } else {
                windows::open_main_window(app.handle());
                synchronizer::start(app.handle());
                tokio::spawn(token::watch_tokens(app.handle()));
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            windows::open_main_window,
            windows::open_config_window,
            windows::open_login_window,
            windows::open_initial_configuration_window,
            login,
            logout,
            get_completed_transfers,
            update_config,
            save_initial_config,
            get_config,
            open_folder,
            force_sync
        ])
        .system_tray(system_tray)
        .on_system_tray_event(|app_handle, event| match event {
            SystemTrayEvent::LeftClick { .. } => {
                windows::open_main_window(app_handle.clone());
            }
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "quit" => {
                    std::process::exit(0);
                }
                "show" => {
                    windows::open_main_window(app_handle.clone());
                }
                "settings" => {
                    windows::open_config_window(app_handle.clone());
                }
                _ => (),
            },
            _ => (),
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| match event {
            tauri::RunEvent::ExitRequested { api, .. } => {
                api.prevent_exit();
            }
            _ => {}
        });
}

#[tauri::command]
fn get_completed_transfers() -> Vec<types::Transfer> {
    TRANSFERS
        .lock()
        .unwrap()
        .values()
        .filter(|transfer| transfer.state == TransferState::Completed)
        .map(|transfer| transfer.clone())
        .collect()
}

#[tauri::command]
async fn update_config(
    app: AppHandle,
    config: Config,
    restart: bool,
) -> Result<(), HashMap<String, String>> {
    let app_dir = app.path_resolver().app_data_dir().unwrap();
    let config_path = app_dir.join("config.json");
    let mut error_map = HashMap::new();
    if config.server_url != CONFIG.lock().unwrap().server_url {
        let client = Client::new();
        let server = config.server_url.to_owned();
        let resp = client.get(format!("{server}/actuator/health")).send().await;
        if resp.is_err() || !resp.unwrap().status().is_success() {
            error_map.insert("server".to_string(), "server not reachable".to_string());
        }
    }

    if config.folder_path != CONFIG.lock().unwrap().folder_path {
        let folder_path = Path::new(&config.folder_path);
        if !folder_path.exists() || !folder_path.is_dir() {
            error_map.insert("folder".to_string(), "folder does not exist".to_string());
        }
    }

    if !error_map.is_empty() {
        return Err(error_map);
    }
    *CONFIG.lock().unwrap() = config.clone();
    if restart {
        synchronizer::stop();
        synchronizer::start(app);
    }
    std::fs::write(config_path, serde_json::to_string_pretty(&config).unwrap()).unwrap();
    Ok(())
}
#[tauri::command]
async fn save_initial_config(
    app: AppHandle,
    folder_path: String,
    server_url: String,
) -> Result<(), HashMap<String, String>> {
    let window = app.get_window("initialConfiguration").unwrap();
    let mut config = CONFIG.lock().unwrap().clone();
    let app_dir = app.path_resolver().app_data_dir().unwrap();
    let config_path = app_dir.join("config.json");
    let client = Client::new();
    let server_url = server_url.trim_end_matches('/').to_owned();
    let resp = client
        .get(format!("{server_url}/actuator/health"))
        .send()
        .await;
    let folder_exists = Path::new(&folder_path).exists();
    let mut error_map = HashMap::new();
    if resp.is_err() || !resp.unwrap().status().is_success() {
        error_map.insert("server".to_string(), "server not reachable".to_string());
    }
    if !folder_exists {
        error_map.insert("folder".to_string(), "folder does not exist".to_string());
    }
    if !error_map.is_empty() {
        return Err(error_map);
    }
    config.folder_path = folder_path;
    config.server_url = server_url;
    config.is_configured = true;
    std::fs::write(config_path, serde_json::to_string_pretty(&config).unwrap()).unwrap();
    *CONFIG.lock().unwrap() = config;
    window.close().unwrap();
    windows::open_login_window(app.clone());
    Ok(())
}

#[tauri::command]
fn get_config() -> Config {
    let config = CONFIG.lock().unwrap().clone();
    config
}

fn set_config(app: AppHandle) {
    let app_dir = app.path_resolver().app_data_dir().unwrap();
    let config_path = app_dir.join("config.json");
    let config_file = std::fs::read_to_string(config_path).unwrap_or_default();
    let config: Config = serde_json::from_str(&config_file).unwrap_or_default();
    *CONFIG.lock().unwrap() = config;
}

#[tauri::command]
async fn login(app: AppHandle, username: String, password: String) -> Result<(), String> {
    let server = CONFIG.lock().unwrap().server_url.to_owned();
    let login_window = app.get_window("Login").unwrap();
    let client = Client::new();
    let resp = client
        .post(format!("{server}/users/auth/login"))
        .json(&json!({"username": username, "password": password}))
        .send()
        .await
        .unwrap();
    println!("status {}", resp.status());
    if !resp.status().is_success() {
        return Err("Username or password incorrect".to_string());
    }

    let token = resp
        .headers()
        .get("authorization")
        .unwrap()
        .to_str()
        .unwrap();
    let refresh_token = resp
        .headers()
        .get("x-refresh-token")
        .unwrap()
        .to_str()
        .unwrap();

    let token = Token {
        value: token.to_string(),
        created_at: SystemTime::now(),
    };
    let refresh_token = Token {
        value: refresh_token.to_string(),
        created_at: SystemTime::now(),
    };

    let config = {
        let mut config = CONFIG.lock().unwrap();
        config.username.replace(username);
        config.password.replace(password);
        config.token.replace(token.to_owned());
        config.refresh_token.replace(refresh_token.to_owned());
        config.clone()
    };
    let _ = update_config(app.clone(), config, true).await;
    login_window.close().unwrap();

    tokio::spawn(token::watch_tokens(app.clone()));
    windows::open_main_window(app);
    Ok(())
}

#[tauri::command]
fn open_folder(path: &str) -> Result<(), String> {
    let path = std::path::Path::new(path);
    match opener::open(path) {
        Ok(()) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
fn force_sync(app: AppHandle) -> Result<(), String> {
    synchronizer::stop();
    synchronizer::start(app);
    Ok(())
}
#[tauri::command]
async fn logout(app: AppHandle) {
    let config = {
        let mut config = CONFIG.lock().unwrap();
        config.token.take();
        config.refresh_token.take();
        config.clone()
    };
    let _ = update_config(app.clone(), config, false).await.unwrap();
    synchronizer::stop();
    token::stop();
    windows::close_all(app.clone(), vec!["Login"]);
    windows::open_login_window(app.clone());
}
