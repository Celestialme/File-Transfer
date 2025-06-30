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
use crate::types::Transfer;

const SERVER: &str = "http://localhost:3000";

pub fn rename(path: &str, destination: &str) {
    let client = Client::new();

    let _ = client
        .post(format!("{SERVER}/rename"))
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
    let _ = client
        .post(format!("{SERVER}/createFolder"))
        .json(&json!({
            "destination": path,
            "socket_id":*SOCKET_ID.lock().unwrap()
        }))
        .send()
        .unwrap();
}

pub fn delete(path: &str) {
    let client = Client::new();
    let _ = client
        .post(format!("{SERVER}/delete"))
        .json(&json!({
            "path": path,
            "socket_id":*SOCKET_ID.lock().unwrap()
        }))
        .send()
        .unwrap();
}

pub fn upload(root_path: &PathBuf, destination: &str) {
    let absolute_path = root_path.join(destination);
    println!("Uploading {:?}", absolute_path);
    let client = Client::new();

    let form = reqwest::blocking::multipart::Form::new()
        .text("destination", destination.to_string())
        .text("socket_id", SOCKET_ID.lock().unwrap().to_string())
        .file("file", absolute_path)
        .unwrap();

    let _ = client
        .post(format!("{SERVER}/upload"))
        .multipart(form)
        .send()
        .unwrap();
}

pub fn download(
    app: tauri::AppHandle,
    root_path: &PathBuf,
    path: &str,
    local_tree: Arc<Mutex<fstree::Node>>,
) {
    let root_path = root_path.clone(); // PathBuf (owned)
    let path = path.to_string();
    let window = app.get_window("main").unwrap();
    std::thread::spawn(move || {
        let client = Client::new();
        let mut resp = client
            .post(format!("{SERVER}/download"))
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
                state: "active".to_string(),
                r#type: "download".to_string(),
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
                state: "active".to_string(),
                r#type: "download".to_string(),
                path: path.to_str().unwrap().to_string(),
            };
            window.emit("transfer", &transfer).unwrap();
            TRANSFERS.lock().unwrap().insert(path.clone(), transfer);
            println!("Downloaded: {} {:.2}%", path.display(), progress);
        }
        let node = fstree::build_node(&Path::new(&root_path), &root_path.join(&path));
        local_tree.lock().unwrap().add_node(node.unwrap()).unwrap();
        fstree::save_tree(&local_tree.lock().unwrap(), "tree.json").unwrap();
        drop(file);
        std::thread::sleep(std::time::Duration::from_millis(1000));
        IGNORE_LIST.lock().unwrap().remove(path.as_path());
        let transfer = Transfer {
            progress: 100,
            state: "completed".to_string(),
            r#type: "download".to_string(),
            path: path.to_str().unwrap().to_string(),
        };
        window.emit("transfer", &transfer).unwrap();
        TRANSFERS.lock().unwrap().insert(path.clone(), transfer);
    });
}
