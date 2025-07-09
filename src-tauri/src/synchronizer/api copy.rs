use futures_util::StreamExt;
use futures_util::TryStreamExt;
use read_progress_stream::ReadProgressStream;
use reqwest::blocking::Client;
use serde_json::json;
use std::sync::{Arc, Mutex};
use std::{fs, io::Write, path::PathBuf};
use tauri::async_runtime::block_on;
use tauri::Manager;
use tokio_util::codec::{BytesCodec, FramedRead};

use crate::force_sync;
use crate::synchronizer::{fstree, IGNORE_LIST, SOCKET_ID, TRANSFERS};
use crate::types::{Transfer, TransferState, TransferType};
use crate::CONFIG;
pub fn rename(id: Option<String>, parent_id: Option<String>, path: &str, destination: &str) {
    let client = Client::new();
    let config = CONFIG.lock().unwrap();
    let server = config.server_url.to_owned();
    let _ = client
        .post(format!("{server}/rename"))
        .json(&json!({
            "path": path,
            "destination": destination,
        }))
        .header("Socket-ID", SOCKET_ID.lock().unwrap().clone())
        .header("Token", config.token.as_ref().unwrap())
        .header("ID", id.unwrap_or_default())
        .header("Parent-ID", parent_id.unwrap_or_default())
        .send()
        .map_err(|_| (println!("Failed to rename {path}")));
}

pub fn create_folder(path: &str) {
    let client = Client::new();
    let config = CONFIG.lock().unwrap();
    let server = config.server_url.to_owned();
    let _ = client
        .post(format!("{server}/files"))
        .json(&json!({
            "destination": path,
        }))
        .header("Socket-ID", SOCKET_ID.lock().unwrap().clone())
        .header("Token", config.token.as_ref().unwrap())
        .send()
        .map_err(|_| (println!("Failed to create folder {path}")));
}

pub fn delete(id: Option<String>, path: &str) {
    let client = Client::new();
    let config = CONFIG.lock().unwrap();
    let server = config.server_url.to_owned();
    let _ = client
        .post(format!("{server}/delete"))
        .json(&json!({
            "path": path
        }))
        .header("Socket-ID", SOCKET_ID.lock().unwrap().clone())
        .header("Token", config.token.as_ref().unwrap())
        .header("ID", id.unwrap_or_default())
        .send()
        .map_err(|_| (println!("Failed to delete {path}")));
}

pub fn upload(
    app: tauri::AppHandle,
    id: Option<String>,
    parent_id: Option<String>,
    root_path: &PathBuf,
    destination: &str,
) {
    let config = CONFIG.lock().unwrap().clone();
    let socket_id = SOCKET_ID.lock().unwrap().clone();
    let server = config.server_url.to_owned();
    let absolute_path = root_path.join(destination);
    println!("Uploading {:?}", absolute_path);

    let destination = destination.to_string();
    let window = app.get_window("main");
    std::thread::spawn(move || {
        block_on(async move {
            let client = reqwest::Client::new();
            let file = tokio::fs::File::open(&absolute_path).await.unwrap();
            let file_size = file.metadata().await.unwrap().len();
            let stream = FramedRead::new(file, BytesCodec::new()).map_ok(|r| r.freeze());
            let _destination = destination.clone();
            let _window = window.clone();
            let body = reqwest::Body::wrap_stream(ReadProgressStream::new(
                stream,
                Box::new(move |_, total| {
                    let progress = ((total as f64 / file_size as f64) * 100.0) as u8;
                    // println!("file:{}, progress: {}", _destination, progress);
                    let transfer = Transfer {
                        progress: progress as u32,
                        state: TransferState::Active,
                        r#type: TransferType::Upload,
                        path: _destination.clone(),
                    };
                    if let Some(ref window) = _window {
                        window.emit("transfer", &transfer).unwrap()
                    };
                    TRANSFERS
                        .lock()
                        .unwrap()
                        .insert(_destination.clone().into(), transfer);
                }),
            ));
            TRANSFERS.lock().unwrap().insert(
                destination.clone().into(),
                Transfer {
                    progress: 0,
                    state: TransferState::Active,
                    r#type: TransferType::Upload,
                    path: destination.clone(),
                },
            );
            let destination_encoded: String = urlencoding::encode(&destination).to_string();
            let _ = client
                .post(format!("{server}/upload"))
                .header("Content-Type", "application/octet-stream")
                .header("Socket-ID", socket_id)
                .header("Destination", destination_encoded)
                .header("Content-Length", file_size.to_string())
                .header("Token", config.token.as_ref().unwrap())
                .header("ID", id.unwrap_or_default())
                .header("Parent-ID", parent_id.unwrap_or_default())
                .body(body)
                .send()
                .await;
            // Mark as completed
            let transfer = Transfer {
                progress: 100,
                state: TransferState::Completed,
                r#type: TransferType::Upload,
                path: destination.clone(),
            };
            if let Some(window) = window {
                window.emit("transfer", &transfer).unwrap()
            };
            TRANSFERS
                .lock()
                .unwrap()
                .insert(destination.clone().into(), transfer);
            force_sync(app).unwrap();
        });
    });
}

pub async fn download(
    app: tauri::AppHandle,
    root_path: &PathBuf,
    path: String,
    local_tree: Arc<Mutex<fstree::Node>>,
) {
    let socket_id = SOCKET_ID.lock().unwrap().clone();
    let config = CONFIG.lock().unwrap().clone();
    let server = config.server_url.to_owned();
    let full_path = root_path.join(&path);
    let destination = full_path.clone();
    let window = app.get_window("main");
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{server}/download"))
        .json(&json!({ "path": path }))
        .header("Socket-ID", socket_id)
        .header("Token", config.token.as_ref().unwrap())
        .send()
        .await;

    let resp = match response {
        Ok(r) if r.status().is_success() => r,
        _ => return, // early return on error or non-2xx
    };

    let total_size = match resp.content_length() {
        Some(size) => size,
        None => {
            eprintln!("Missing content length");
            return;
        }
    };

    IGNORE_LIST
        .lock()
        .unwrap()
        .insert(destination.clone().into());

    TRANSFERS.lock().unwrap().insert(
        destination.clone(),
        Transfer {
            progress: 0,
            state: TransferState::Active,
            r#type: TransferType::Download,
            path: destination.to_string_lossy().to_string(),
        },
    );

    // Ensure parent directories exist
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).unwrap();
    }

    let mut file = fs::File::create(&destination).unwrap();
    let mut downloaded: u64 = 0;
    let mut stream = resp.bytes_stream();

    while let Some(chunk_result) = stream.next().await {
        let chunk = match chunk_result {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error downloading chunk: {e}");
                return;
            }
        };
        file.write_all(&chunk).unwrap();
        downloaded += chunk.len() as u64;

        let progress = (downloaded as f64 / total_size as f64) * 100.0;
        // println!(
        //     "file:{}, progress: {}",
        //     destination.to_string_lossy().to_string(),
        //     progress
        // );
        let transfer = Transfer {
            progress: progress as u32,
            state: TransferState::Active,
            r#type: TransferType::Download,
            path: destination.to_string_lossy().to_string(),
        };
        if let Some(ref window) = window {
            window.emit("transfer", &transfer).unwrap();
        }
        TRANSFERS
            .lock()
            .unwrap()
            .insert(destination.clone(), transfer);
    }

    // Add to local tree
    if let Ok(Some(node)) = fstree::build_node(&root_path, &destination).map(Some) {
        local_tree.lock().unwrap().add_node(node).unwrap();
        fstree::save_tree(&local_tree.lock().unwrap(), "tree.json").unwrap();
    }

    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    IGNORE_LIST.lock().unwrap().remove(destination.as_path());

    let transfer = Transfer {
        progress: 100,
        state: TransferState::Completed,
        r#type: TransferType::Download,
        path: destination.to_string_lossy().to_string(),
    };
    if let Some(window) = window {
        window.emit("transfer", &transfer).unwrap()
    };
    TRANSFERS.lock().unwrap().insert(destination, transfer);
}
