// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod synchronizer;
mod types;
use tauri::{AppHandle, Manager};
use tauri_plugin_positioner::{Position, WindowExt};

use crate::synchronizer::TRANSFERS;
fn main() {
    tauri::Builder::default()
        .setup(|app| {
            open_main_window(app.handle());
            open_config_window(app.handle());
            synchronizer::start(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            open_config_window,
            get_completed_transfers
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
        .filter(|transfer| transfer.state == "completed")
        .map(|transfer| transfer.clone())
        .collect()
}
