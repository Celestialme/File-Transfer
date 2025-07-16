use futures_util::StreamExt;
use futures_util::TryStreamExt;
use reqwest::blocking::multipart::Part;
use reqwest::blocking::Client;
use serde_json::json;
use std::sync::{Arc, Mutex};
use std::{fs, io::Write, path::PathBuf};
use tauri::async_runtime::block_on;
use tauri::Manager;
use tokio_util::io::ReaderStream;

use crate::synchronizer::{fstree, TRANSFERS};
use crate::types::{Transfer, TransferState, TransferType};
use crate::CONFIG;
pub fn rename(
    id: Arc<Mutex<Option<String>>>,
    parent_id: Arc<Mutex<Option<String>>>,
    path: &str,
    destination: &str,
) {
    let id = id.lock().unwrap().clone();
    if id.is_none() {
        return;
    }
    let id = id.unwrap();
    let parent_id = parent_id.lock().unwrap().clone();
    let client = Client::new();
    let config = CONFIG.lock().unwrap();
    let server = config.server_url.to_owned();
    let token = config.token.as_ref().unwrap().value.clone();
    let name = std::path::Path::new(destination)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let _ = client
        .put(format!("{server}/files/{id}"))
        .json(&json!({
            "path": path,
            "name": name,
            "destination": destination,
            "parentId": parent_id.unwrap_or_default()
        }))
        .header("authorization", token)
        .send()
        .map_err(|_| (println!("Failed to rename {path}")));
}

pub fn create_folder(
    path: &str,
    id: Arc<Mutex<Option<String>>>,
    parent_id: Arc<Mutex<Option<String>>>,
) {
    let client = Client::new();
    let config = CONFIG.lock().unwrap();
    let server = config.server_url.to_owned();
    let token = config.token.as_ref().unwrap().value.clone();
    let parent_id = parent_id.lock().unwrap().clone().unwrap();
    let name = std::path::Path::new(path)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let json_string =
        json!({"name": name, "parentFolderId": parent_id, "isFolder": true,"contentType":"folder", "size": 0}).to_string();
    let part = Part::text(json_string)
        .mime_str("application/json")
        .unwrap()
        .file_name("request.json");
    let form = reqwest::blocking::multipart::Form::new().part("request", part);

    let res = client
        .post(format!("{server}/files"))
        .header("authorization", token)
        .multipart(form)
        .send()
        .map_err(|_| (println!("Failed to create folder {path}")));

    if res.is_err() || !res.as_ref().unwrap().status().is_success() {
        return;
    }
    let resp: serde_json::Value = res.unwrap().json().unwrap();
    id.lock()
        .unwrap()
        .replace(resp["id"].as_str().unwrap().to_string());
}

pub fn delete(id: Arc<Mutex<Option<String>>>, path: &str) {
    let id = id.lock().unwrap().clone();
    if id.is_none() {
        return;
    }
    let id = id.unwrap();
    let client = Client::new();
    let config = CONFIG.lock().unwrap();
    let server = config.server_url.to_owned();
    let token = config.token.as_ref().unwrap().value.clone();
    let _ = client
        .delete(format!("{server}/files/{id}"))
        .json(&json!({
            "path": path
        }))
        .header("authorization", token)
        .send()
        .map_err(|_| (println!("Failed to delete {path}")));
}

pub fn upload(
    app: tauri::AppHandle,
    id: Arc<Mutex<Option<String>>>,
    parent_id: Arc<Mutex<Option<String>>>,
    root_path: &PathBuf,
    destination: &str,
    tree: Arc<Mutex<fstree::Node>>,
) {
    let file_id = id.lock().unwrap().clone();
    let parent_id = parent_id.lock().unwrap().clone();

    let config = CONFIG.lock().unwrap().clone();
    let server = config.server_url.to_owned();
    let token = config.token.as_ref().unwrap().value.clone();
    let absolute_path = root_path.join(destination);
    println!("Uploading {:?}", absolute_path);

    let destination = destination.to_string();
    let window = app.get_window("main");
    std::thread::spawn(move || {
        block_on(async move {
            let client = reqwest::Client::new();
            let file = tokio::fs::File::open(&absolute_path).await;
            if file.is_err() {
                println!(
                    "Failed to open file: can't upload {}",
                    absolute_path.display()
                );
                return;
            }
            let file = file.unwrap();
            let file_size = file.metadata().await.unwrap().len();
            let file_name = absolute_path.file_name().unwrap().to_str().unwrap();
            let _destination = destination.clone();
            let _window = window.clone();
            let stream = ReaderStream::new(file);
            let mut total = 0;
            let byte_stream = stream.inspect_ok(move |chunk| {
                total += chunk.len();
                let progress = ((total as f64 / file_size as f64) * 100.0) as u8;
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
            });
            let _destination = destination.clone();
            let _window = window.clone();
            let body = reqwest::Body::wrap_stream(byte_stream);
            TRANSFERS.lock().unwrap().insert(
                destination.clone().into(),
                Transfer {
                    progress: 0,
                    state: TransferState::Active,
                    r#type: TransferType::Upload,
                    path: destination.clone(),
                },
            );
            let file_name_encoded: String = urlencoding::encode(&file_name).to_string();
            println!("{:?}", file_id);
            let resp = client
                .post(format!("{server}/upload"))
                .header("Content-Type", "application/octet-stream")
                .header("fileName", file_name_encoded)
                .header("Content-Length", file_size.to_string())
                .header("authorization", token)
                .header("elementId", file_id.unwrap_or_default())
                .header("parentId", parent_id.unwrap_or_default())
                .body(body);
            let resp = resp.send().await;
            // Mark as completed
            let resp: serde_json::Value = resp.unwrap().json().await.unwrap();
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
            id.lock()
                .unwrap()
                .replace(resp["id"].as_str().unwrap().to_string());
            println!("id is {}", id.lock().unwrap().clone().unwrap());
            fstree::save_tree(&tree.lock().unwrap(), "tree.json").unwrap();
        });
    });
}

pub async fn download(
    app: tauri::AppHandle,
    root_path: &PathBuf,
    path: String,
    id: Arc<Mutex<Option<String>>>,
    hash: String,
    local_tree: Arc<Mutex<fstree::Node>>,
) {
    let id = id.lock().unwrap().clone();
    let app_dir = app.path_resolver().app_data_dir().unwrap();
    let temp_dir = app_dir.join("temp");
    fs::create_dir_all(&temp_dir).unwrap();
    let mut bytes = path.as_bytes().to_vec();
    bytes.extend(hash.as_bytes());
    let temp_name = fstree::hash_bytes(&bytes);
    let temp_file_path = temp_dir.join(temp_name);
    println!("Downloading {} to {}", path, temp_file_path.display());
    if id.is_none() || temp_file_path.exists() {
        return;
    }
    let config = CONFIG.lock().unwrap().clone();
    let server = config.server_url.to_owned();
    let token = config.token.as_ref().unwrap().value.clone();
    let full_path = root_path.join(&path);
    let destination = full_path.clone();
    let window = app.get_window("main");
    let client = reqwest::Client::new();

    let id = id.unwrap();
    let response = client
        .get(format!("{server}/files/{id}/download"))
        .header("authorization", token)
        .send()
        .await;
    let resp = match response {
        Ok(r) if r.status().is_success() => r,
        _ => {
            let _ = fs::remove_file(&temp_file_path)
                .map_err(|e| println!("Failed to remove temp file: {}", e));
            return;
        } // early return on error or non-2xx
    };

    let total_size = match resp.content_length() {
        Some(size) => size,
        None => {
            let _ = fs::remove_file(&temp_file_path)
                .map_err(|e| println!("Failed to remove temp file: {}", e));
            return;
        }
    };

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

    let mut file = fs::File::create(&temp_file_path).unwrap();
    let mut downloaded: u64 = 0;
    let mut stream = resp.bytes_stream();

    while let Some(chunk_result) = stream.next().await {
        let chunk = match chunk_result {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error downloading chunk: {e}");
                let _ = fs::remove_file(&temp_file_path)
                    .map_err(|e| println!("Failed to remove temp file: {}", e));
                return;
            }
        };
        file.write_all(&chunk).unwrap();
        downloaded += chunk.len() as u64;

        let progress = (downloaded as f64 / total_size as f64) * 100.0;

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

    let _ = fs::copy(&temp_file_path, &destination)
        .map_err(|e| println!("Failed to Move from temp dir: {}", e));
    let _ =
        fs::remove_file(&temp_file_path).map_err(|e| println!("Failed to remove temp file: {}", e));
    // Add to local tree
    if let Ok(Some(node)) = fstree::build_node(&root_path, &destination).map(Some) {
        local_tree.lock().unwrap().add_node(node).unwrap();
        fstree::save_tree(&local_tree.lock().unwrap(), "tree.json").unwrap();
    }

    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

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
