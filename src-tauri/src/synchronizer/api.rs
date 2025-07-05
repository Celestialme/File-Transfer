use reqwest::blocking::Client;
use serde_json::json;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::{
    fs::{self, File},
    io::{self, Read, Write},
    path::PathBuf,
};
use tauri::Manager;

use crate::synchronizer::{fstree, IGNORE_LIST, SOCKET_ID, TRANSFERS};
use crate::types::{Transfer, TransferState, TransferType};
use crate::CONFIG;

pub fn rename(path: &str, destination: &str) {
    let client = Client::new();
    let config = CONFIG.lock().unwrap();
    let server = config.server_url.to_owned();
    let _ = client
        .post(format!("{server}/rename"))
        .json(&json!({
            "path": path,
            "destination": destination,
            "socket_id":*SOCKET_ID.lock().unwrap()
        }))
        .send()
        .unwrap();
}

pub fn create_folder(path: &str) {
    let client = Client::new();
    let config = CONFIG.lock().unwrap();
    let server = config.server_url.to_owned();
    let _ = client
        .post(format!("{server}/createFolder"))
        .json(&json!({
            "destination": path,
            "socket_id":*SOCKET_ID.lock().unwrap()
        }))
        .send()
        .unwrap();
}

pub fn delete(path: &str) {
    let client = Client::new();
    let config = CONFIG.lock().unwrap();
    let server = config.server_url.to_owned();
    let _ = client
        .post(format!("{server}/delete"))
        .json(&json!({
            "path": path,
            "socket_id":*SOCKET_ID.lock().unwrap()
        }))
        .send()
        .unwrap();
}

pub fn upload(app: tauri::AppHandle, root_path: &PathBuf, destination: &str) {
    let config = CONFIG.lock().unwrap().clone();
    let server = config.server_url.to_owned();
    let absolute_path = root_path.join(destination);
    println!("Uploading {:?}", absolute_path);

    let destination = destination.to_string();
    let window = app.get_window("main");

    std::thread::spawn(move || {
        let client = Client::new();

        // Open the file and get its size
        let mut file = File::open(&absolute_path).unwrap();
        let file_size = file.metadata().unwrap().len();

        // Add to transfers with initial state
        TRANSFERS.lock().unwrap().insert(
            destination.clone().into(),
            Transfer {
                progress: 0,
                state: TransferState::Active,
                r#type: TransferType::Upload,
                path: destination.clone(),
            },
        );

        let mut uploaded: u64 = 0;
        let mut buffer = [0; 8192]; // 8KB buffer
        let destination_encoded: String = urlencoding::encode(&destination).to_string();
        // Create the request with headers
        let request = client
            .post(format!("{server}/upload"))
            .header("Content-Type", "application/octet-stream")
            .header("Socket-ID", SOCKET_ID.lock().unwrap().to_string())
            .header("Destination", destination_encoded)
            .header("Content-Length", file_size.to_string());

        // Read file in chunks and build the body
        let mut body_data = Vec::new();
        loop {
            let n = file.read(&mut buffer).unwrap();
            if n == 0 {
                break;
            }
            body_data.extend_from_slice(&buffer[..n]);
            uploaded += n as u64;

            let progress = (uploaded as f64 / file_size as f64) * 100.0;
            let transfer = Transfer {
                progress: progress as u32,
                state: TransferState::Active,
                r#type: TransferType::Upload,
                path: destination.clone(),
            };
            if let Some(ref window) = window {
                window.emit("transfer", &transfer).unwrap()
            };
            TRANSFERS
                .lock()
                .unwrap()
                .insert(destination.clone().into(), transfer);

            // Add a small delay to show progress (optional)
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        // Send the request with the body
        let _response = request.body(body_data).send().unwrap();

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
    });
}

pub fn download(
    app: tauri::AppHandle,
    root_path: &PathBuf,
    path: &str,
    local_tree: Arc<Mutex<fstree::Node>>,
) {
    let config = CONFIG.lock().unwrap().clone();
    let server = config.server_url.to_owned();
    let root_path = root_path.clone(); // PathBuf (owned)
    let path = path.to_string();
    let window = app.get_window("main");
    std::thread::spawn(move || {
        let client = Client::new();
        let mut resp = client
            .post(format!("{server}/download"))
            .json(&json!({"path": path}))
            .send()
            .unwrap();
        let total_size = resp
            .content_length()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Missing content length"))
            .unwrap();
        let path = root_path.join(path);
        IGNORE_LIST.lock().unwrap().insert(path.clone().into());
        TRANSFERS.lock().unwrap().insert(
            path.clone(),
            Transfer {
                progress: 0,
                state: TransferState::Active,
                r#type: TransferType::Download,
                path: path.to_str().unwrap().to_string(),
            },
        );
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut file = File::create(&path).unwrap();

        let mut downloaded: u64 = 0;
        let mut buffer = [0; 8192]; // 8KB buffer
        loop {
            std::thread::sleep(std::time::Duration::from_millis(100));
            let n = resp.read(&mut buffer).unwrap();
            if n == 0 {
                break;
            }
            file.write_all(&buffer[..n]).unwrap();
            downloaded += n as u64;

            let progress = (downloaded as f64 / total_size as f64) * 100.0;
            let transfer = Transfer {
                progress: progress as u32,
                state: TransferState::Active,
                r#type: TransferType::Download,
                path: path.to_str().unwrap().to_string(),
            };
            if let Some(ref window) = window {
                window.emit("transfer", &transfer).unwrap()
            };
            TRANSFERS.lock().unwrap().insert(path.clone(), transfer);
            // println!("Downloaded: {} {:.2}%", path.display(), progress);
        }
        let node = fstree::build_node(&Path::new(&root_path), &root_path.join(&path));
        local_tree.lock().unwrap().add_node(node.unwrap()).unwrap();
        fstree::save_tree(&local_tree.lock().unwrap(), "tree.json").unwrap();
        drop(file);
        std::thread::sleep(std::time::Duration::from_millis(1000));
        IGNORE_LIST.lock().unwrap().remove(path.as_path());
        let transfer = Transfer {
            progress: 100,
            state: TransferState::Completed,
            r#type: TransferType::Download,
            path: path.to_str().unwrap().to_string(),
        };
        if let Some(window) = window {
            window.emit("transfer", &transfer).unwrap()
        };
        TRANSFERS.lock().unwrap().insert(path.clone(), transfer);
    });
}
