// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod synchronizer;
mod types;
use std::sync::{LazyLock, Mutex};

use reqwest::Client;
use serde_json::json;
use tauri::{AppHandle, Manager};
use tauri_plugin_positioner::{Position, WindowExt};

use crate::{
    synchronizer::TRANSFERS,
    types::{Config, LoginResponse, TransferState},
};
static CONFIG: LazyLock<Mutex<Config>> = LazyLock::new(|| Mutex::new(Config::default()));
#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .setup(|app| {
            std::fs::create_dir_all(app.path_resolver().app_data_dir().unwrap()).unwrap();
            set_config(app.handle());
            let config = CONFIG.lock().unwrap().clone();
            if !config.is_configured {
                open_initial_configuration_window(app.handle());
            } else if !config.username.is_none() || config.password.is_none() {
                open_login_window(app.handle());
            } else {
                open_main_window(app.handle());
                synchronizer::start(app.handle());
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            open_main_window,
            open_config_window,
            open_login_window,
            open_initial_configuration_window,
            login,
            get_completed_transfers,
            update_config,
            save_initial_config,
            get_config
        ])
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
fn open_main_window(app: AppHandle) {
    let main_window = app.get_window("main");
    if main_window.is_some() {
        main_window.unwrap().set_focus().unwrap();
        return;
    }
    let position = Position::TopRight;
    let window = tauri::WindowBuilder::new(&app, "main", tauri::WindowUrl::App("/".into()))
        .title("File Transfer")
        .inner_size(400.0, 500.0)
        .theme(Some(tauri::Theme::Light))
        .resizable(false)
        .maximizable(false)
        .minimizable(false)
        .decorations(false)
        .build()
        .unwrap();
    window.move_window(position).unwrap();
    let _window = window.clone();
    window.on_window_event(move |event| match event {
        tauri::WindowEvent::Focused(focused) => {
            if !focused {
                // _window.clone().close().unwrap();
            }
        }
        _ => {}
    });
}

#[tauri::command]
fn open_config_window(app: AppHandle) {
    let config_window = app.get_window("config");
    if config_window.is_some() {
        config_window.unwrap().set_focus().unwrap();
        return;
    }
    tauri::async_runtime::spawn(async move {
        tauri::WindowBuilder::new(&app, "config", tauri::WindowUrl::App("/config".into()))
            .title("Configuración")
            .inner_size(600.0, 500.0)
            .min_inner_size(600.0, 500.0)
            .theme(Some(tauri::Theme::Light))
            .maximizable(false)
            .minimizable(false)
            .build()
            .unwrap();
    });
}

#[tauri::command]
fn open_login_window(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        tauri::WindowBuilder::new(&app, "Login", tauri::WindowUrl::App("/login".into()))
            .title("Login")
            .inner_size(600.0, 500.0)
            .min_inner_size(600.0, 500.0)
            .theme(Some(tauri::Theme::Light))
            .maximizable(false)
            .minimizable(false)
            .build()
            .unwrap();
    });
}

#[tauri::command]
fn open_initial_configuration_window(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        tauri::WindowBuilder::new(
            &app,
            "initialConfiguration",
            tauri::WindowUrl::App("/initialConfiguration".into()),
        )
        .title("Initial Configuration")
        .inner_size(600.0, 500.0)
        .min_inner_size(600.0, 500.0)
        .theme(Some(tauri::Theme::Light))
        .maximizable(false)
        .minimizable(false)
        .build()
        .unwrap();
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
fn update_config(app: AppHandle, config: Config) {
    let app_dir = app.path_resolver().app_data_dir().unwrap();
    let config_path = app_dir.join("config.json");
    *CONFIG.lock().unwrap() = config.clone();
    synchronizer::stop();
    synchronizer::start(app);
    std::fs::write(config_path, serde_json::to_string_pretty(&config).unwrap()).unwrap();
}
#[tauri::command]
fn save_initial_config(app: AppHandle, folder_path: String, server_url: String) {
    let window = app.get_window("initialConfiguration").unwrap();
    window.close().unwrap();
    open_login_window(app.clone());
    let mut config = CONFIG.lock().unwrap();
    let app_dir = app.path_resolver().app_data_dir().unwrap();
    let config_path = app_dir.join("config.json");
    config.folder_path = folder_path;
    config.server_url = server_url;
    config.is_configured = true;
    std::fs::write(config_path, serde_json::to_string_pretty(&*config).unwrap()).unwrap();
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
        .post(format!("{server}/login"))
        .json(&json!({"username": username, "password": password}))
        .send()
        .await
        .unwrap();
    println!("status {}", resp.status());
    if !resp.status().is_success() {
        return Err("Username or password incorrect".to_string());
    }
    let loging_response: LoginResponse = serde_json::from_str(&resp.text().await.unwrap()).unwrap();
    let config = {
        let mut config = CONFIG.lock().unwrap();
        config.username.replace(username);
        config.password.replace(password);
        config.token.replace(loging_response.token.clone());
        config
            .refresh_token
            .replace(loging_response.refresh_token.clone());
        config.clone()
    };
    update_config(app.clone(), config);
    login_window.close().unwrap();
    open_main_window(app);
    Ok(())
}
