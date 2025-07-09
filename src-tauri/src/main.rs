// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod synchronizer;
mod types;
use std::{
    collections::HashMap,
    path::Path,
    sync::{LazyLock, Mutex},
};

use reqwest::Client;
use serde_json::json;
use tauri::{
    AppHandle, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu,
    SystemTrayMenuItem,
};
use tauri_plugin_positioner::{Position, WindowExt};

use crate::{
    synchronizer::TRANSFERS,
    types::{Config, TransferState},
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
                open_initial_configuration_window(app.handle());
            } else if config.username.is_none() || config.password.is_none() {
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
            get_config,
            open_folder,
            force_sync
        ])
        .system_tray(system_tray)
        .on_system_tray_event(|app_handle, event| match event {
            SystemTrayEvent::LeftClick { .. } => {
                open_main_window(app_handle.clone());
            }
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "quit" => {
                    std::process::exit(0);
                }
                "show" => {
                    open_main_window(app_handle.clone());
                }
                "settings" => {
                    open_config_window(app_handle.clone());
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
fn open_main_window(app: AppHandle) {
    let config = CONFIG.lock().unwrap().clone();
    if !config.is_configured {
        open_initial_configuration_window(app);
        return;
    } else if config.username.is_none() || config.password.is_none() {
        open_login_window(app);
        return;
    }

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
        .always_on_top(true)
        .focused(false)
        .visible(false)
        .build()
        .unwrap();
    window.move_window(position).unwrap();
    window.show().unwrap();
    window.set_focus().unwrap();
    let _window = window.clone();
    window.on_window_event(move |event| match event {
        tauri::WindowEvent::Focused(focused) => {
            if !focused {
                _window.close().unwrap();
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
async fn update_config(app: AppHandle, config: Config) -> Result<(), HashMap<String, String>> {
    let app_dir = app.path_resolver().app_data_dir().unwrap();
    let config_path = app_dir.join("config.json");
    let mut error_map = HashMap::new();
    if config.server_url != CONFIG.lock().unwrap().server_url {
        let client = Client::new();
        let server = config.server_url.to_owned();
        let resp = client.get(format!("{server}/health/check")).send().await;
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
    synchronizer::stop();
    synchronizer::start(app);
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
    open_login_window(app.clone());
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

    let config = {
        let mut config = CONFIG.lock().unwrap();
        config.username.replace(username);
        config.password.replace(password);
        config.token.replace(token.to_owned());
        config.refresh_token.replace(refresh_token.to_owned());
        config.clone()
    };
    let _ = update_config(app.clone(), config).await;
    login_window.close().unwrap();
    open_main_window(app);
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
