use notify::{
    event::{CreateKind, ModifyKind, RemoveKind, RenameMode},
    Event, EventKind, RecursiveMode, Result, Watcher,
};
use std::{
    collections::{HashMap, HashSet},
    sync::LazyLock,
};
use std::{
    path::{Path, PathBuf},
    sync::{mpsc, Arc, Mutex},
};
use tauri::Manager;
use tungstenite::{connect, http::Uri, ClientRequestBuilder};

use crate::{types::Transfer, CONFIG};
mod api;
mod debouncer;
mod fstree;

static IGNORE_LIST: LazyLock<Mutex<HashSet<PathBuf>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
pub static TRANSFERS: LazyLock<Mutex<HashMap<PathBuf, Transfer>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static SOCKET_ID: std::sync::Mutex<String> = std::sync::Mutex::new(String::new());
static WATCHER: std::sync::Mutex<Option<notify::ReadDirectoryChangesWatcher>> =
    std::sync::Mutex::new(None);
pub fn start(app: tauri::AppHandle) {
    std::thread::spawn(move || {
        let config = CONFIG.lock().unwrap().clone();
        let root_path = config.folder_path;
        *SOCKET_ID.lock().unwrap() = uuid::Uuid::new_v4().to_string();

        let (tx, rx) = mpsc::channel::<Result<Event>>();
        let mut watcher = notify::recommended_watcher(tx).unwrap();
        let root_path = PathBuf::from(root_path);
        if !root_path.exists() {
            return;
        }
        let local_tree = Arc::new(Mutex::new(fstree::build_tree(&root_path).unwrap()));
        let _root_path = root_path.clone();
        let _local_tree = local_tree.clone();
        fstree::save_tree(&local_tree.lock().unwrap(), "tree.json").unwrap();
        std::thread::spawn({
            let local_tree = _local_tree.clone();
            let root_path = _root_path.clone();
            let app = app.app_handle().clone();
            move || loop {
                let config = CONFIG.lock().unwrap().clone();
                let server = config.server_url;
                let socket_id = SOCKET_ID.lock().unwrap().clone();
                let uri: Uri = server
                    .replace("https://", "wss://")
                    .replace("http://", "ws://")
                    .parse()
                    .unwrap();
                let request = ClientRequestBuilder::new(uri).with_header("socket_id", &socket_id);
                match connect(request) {
                    Ok((mut socket, _response)) => {
                        println!("Connected to server");

                        while let Ok(msg) = socket.read() {
                            if msg.is_text() {
                                let text = msg.to_string();
                                println!("Received new tree");
                                let mut changes: Vec<fstree::Change> = Vec::new();
                                let remote_tree: fstree::Node =
                                    serde_json::from_str(&text).unwrap();

                                {
                                    let local = local_tree.lock().unwrap();
                                    fstree::diff_trees(
                                        "",
                                        Some(&local),
                                        Some(&remote_tree),
                                        &mut changes,
                                    );
                                }

                                let changes = fstree::detect_renames(changes);
                                println!("changes: {:?}", changes);
                                for change in changes {
                                    match change.change_type {
                                        fstree::ChangeType::Added => match change.node_type {
                                            fstree::NodeType::File => {
                                                println!("add {}", change.path);
                                                api::download(
                                                    app.app_handle().clone(),
                                                    &root_path,
                                                    &change.path,
                                                    local_tree.clone(),
                                                );
                                            }
                                            fstree::NodeType::Folder => {
                                                std::fs::create_dir_all(
                                                    Path::new(&root_path).join(&change.path),
                                                )
                                                .unwrap();
                                                let node = fstree::build_node(
                                                    &Path::new(&root_path),
                                                    &root_path.join(&change.path),
                                                );
                                                local_tree
                                                    .lock()
                                                    .unwrap()
                                                    .add_node(node.unwrap())
                                                    .unwrap();
                                            }
                                        },
                                        fstree::ChangeType::Deleted => match change.node_type {
                                            fstree::NodeType::File => {
                                                let _ = std::fs::remove_file(
                                                    Path::new(&root_path).join(&change.path),
                                                );

                                                println!("del {}", change.path);
                                                local_tree
                                                    .lock()
                                                    .unwrap()
                                                    .delete_node(
                                                        root_path
                                                            .join(&change.path)
                                                            .to_str()
                                                            .unwrap(),
                                                    )
                                                    .unwrap();
                                            }
                                            fstree::NodeType::Folder => {
                                                std::fs::remove_dir_all(
                                                    Path::new(&root_path).join(&change.path),
                                                )
                                                .unwrap();
                                                local_tree
                                                    .lock()
                                                    .unwrap()
                                                    .delete_node(
                                                        root_path
                                                            .join(&change.path)
                                                            .to_str()
                                                            .unwrap(),
                                                    )
                                                    .unwrap();
                                            }
                                        },
                                        fstree::ChangeType::Renamed { from } => {
                                            std::fs::rename(
                                                Path::new(&root_path).join(&from),
                                                Path::new(&root_path).join(&change.path),
                                            )
                                            .unwrap();
                                            local_tree
                                                .lock()
                                                .unwrap()
                                                .rename_node(
                                                    root_path.join(&from).to_str().unwrap(),
                                                    root_path.join(&change.path).to_str().unwrap(),
                                                )
                                                .unwrap();
                                        }
                                        fstree::ChangeType::Modified => {
                                            api::download(
                                                app.app_handle().clone(),
                                                &root_path,
                                                &change.path,
                                                local_tree.clone(),
                                            );
                                        }
                                    }
                                }

                                fstree::save_tree(&local_tree.lock().unwrap(), "tree.json")
                                    .unwrap();
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to connect: {}", e);
                    }
                }

                std::thread::sleep(std::time::Duration::from_secs(5));
            }
        });
        // Block forever, printing out events as they come in
        watcher
            .watch(std::path::Path::new(&root_path), RecursiveMode::Recursive)
            .unwrap();
        WATCHER.lock().unwrap().replace(watcher);
        let debouncer = debouncer::Debouncer::new(std::time::Duration::from_millis(1000));
        // watcher.unwatch(&root_path).unwrap();

        while let Ok(res) = rx.recv() {
            match res {
                Ok(event) => handle_event(
                    app.app_handle().clone(),
                    event,
                    local_tree.clone(),
                    root_path.clone(),
                    &debouncer,
                ),
                Err(e) => println!("watch error: {:?}", e),
            }
        }
    });
}
pub fn stop() {
    let _ = WATCHER.lock().unwrap().take();
}
fn handle_event(
    app: tauri::AppHandle,
    event: Event,
    tree: Arc<Mutex<fstree::Node>>,
    root_path: PathBuf,
    debouncer: &debouncer::Debouncer,
) {
    println!("detect");
    for path in event.paths.iter() {
        if IGNORE_LIST.lock().unwrap().contains(path.as_path()) {
            return;
        }
    }
    match event {
        val if val.kind == EventKind::Create(CreateKind::Any) => {
            std::thread::sleep(std::time::Duration::from_millis(100));
            let node = fstree::build_node(
                &root_path,
                std::path::Path::new(&val.paths[0].to_str().unwrap()),
            )
            .unwrap();
            tree.lock().unwrap().add_node(node).unwrap();
        }
        val if val.kind == EventKind::Remove(RemoveKind::Any) => {
            tree.lock()
                .unwrap()
                .delete_node(&val.paths[0].to_str().unwrap())
                .unwrap();
        }
        val if val.kind == EventKind::Modify(ModifyKind::Name(RenameMode::From)) => {
            tree.lock()
                .unwrap()
                .delete_node(&val.paths[0].to_str().unwrap())
                .unwrap();
        }
        val if val.kind == EventKind::Modify(ModifyKind::Name(RenameMode::To))
            || val.kind == EventKind::Modify(ModifyKind::Any) =>
        {
            let node = fstree::build_node(
                &root_path,
                std::path::Path::new(&val.paths[0].to_str().unwrap()),
            );
            if node.is_ok() {
                tree.lock().unwrap().add_node(node.unwrap()).unwrap();
            }
        }

        _ => {}
    }

    debouncer.call(move || {
        let mut changes: Vec<fstree::Change> = Vec::new();
        let saved_tree = std::fs::read_to_string("tree.json").unwrap();
        let tree2: fstree::Node = serde_json::from_str(&saved_tree).unwrap();

        fstree::diff_trees("", Some(&tree2), Some(&tree.lock().unwrap()), &mut changes);
        fstree::save_tree(&tree.lock().unwrap(), "tree.json").unwrap();

        let changes = fstree::detect_renames(changes);
        println!("changes: {:?}", changes);
        for change in changes {
            println!(
                "nodeType:{:?} -> {:?}: {}",
                change.node_type, change.change_type, change.path,
            );

            match change.change_type {
                fstree::ChangeType::Added => match change.node_type {
                    fstree::NodeType::File => api::upload(app.clone(), &root_path, &change.path),
                    fstree::NodeType::Folder => api::create_folder(&change.path),
                },
                fstree::ChangeType::Deleted => api::delete(&change.path),
                fstree::ChangeType::Renamed { from } => api::rename(&from, &change.path),
                fstree::ChangeType::Modified => api::upload(app.clone(), &root_path, &change.path),
            }
        }
    })
}
