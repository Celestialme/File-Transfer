// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod synchronizer;
mod types;
use std::sync::{LazyLock, Mutex};

use tauri::{AppHandle, Manager};
use tauri_plugin_positioner::{Position, WindowExt};

use crate::{
    synchronizer::TRANSFERS,
    types::{Config, TransferState},
};
static CONFIG: LazyLock<Mutex<Config>> = LazyLock::new(|| Mutex::new(Config::default()));
#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .setup(|app| {
            std::fs::create_dir_all(app.path_resolver().app_data_dir().unwrap()).unwrap();
            set_config(app.handle());
            open_main_window(app.handle());
            open_config_window(app.handle());

            synchronizer::start(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            open_config_window,
            get_completed_transfers,
            update_config,
            get_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

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
fn get_config() -> Config {
    let config = CONFIG.lock().unwrap().clone();
    config
}

fn set_config(app: AppHandle) {
    let app_dir = app.path_resolver().app_data_dir().unwrap();
    let config_path = app_dir.join("config.json");
    let config_file = std::fs::read_to_string(config_path).unwrap();
    let config: Config = serde_json::from_str(&config_file).unwrap_or_default();
    *CONFIG.lock().unwrap() = config;
}
