use crate::CONFIG;
use tauri::{AppHandle, Manager};
use tauri_plugin_positioner::{Position, WindowExt};
#[tauri::command]
pub fn open_main_window(app: AppHandle) {
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
pub fn open_config_window(app: AppHandle) {
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
            .focused(true)
            .maximizable(false)
            .minimizable(false)
            .build()
            .unwrap();
    });
}

#[tauri::command]
pub fn open_login_window(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let login_window = app.get_window("Login");
        if login_window.is_some() {
            login_window.unwrap().set_focus().unwrap();
            return;
        }
        tauri::WindowBuilder::new(&app, "Login", tauri::WindowUrl::App("/login".into()))
            .title("Login")
            .inner_size(600.0, 500.0)
            .min_inner_size(600.0, 500.0)
            .theme(Some(tauri::Theme::Light))
            .focused(true)
            .maximizable(false)
            .minimizable(false)
            .build()
            .unwrap();
    });
}

#[tauri::command]
pub fn open_initial_configuration_window(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        tauri::WindowBuilder::new(
            &app,
            "initialConfiguration",
            tauri::WindowUrl::App("/initialConfiguration".into()),
        )
        .title("Initial Configuration")
        .inner_size(600.0, 500.0)
        .min_inner_size(600.0, 500.0)
        .focused(true)
        .theme(Some(tauri::Theme::Light))
        .maximizable(false)
        .minimizable(false)
        .build()
        .unwrap();
    });
}

pub fn close_all(app: AppHandle) {
    let windows = app.windows().into_iter().collect::<Vec<_>>();
    for (_, window) in windows {
        window.close().unwrap();
    }
}
