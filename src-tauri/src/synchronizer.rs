use crate::{
    types::{SocketResponse, Transfer},
    CONFIG,
};
use futures_util::future::join_all;
use futures_util::StreamExt;
use notify::{
    event::{AccessKind, AccessMode, CreateKind, ModifyKind, RemoveKind, RenameMode},
    Event, EventKind, RecommendedWatcher, RecursiveMode, Result, Watcher,
};
use std::{collections::HashMap, sync::LazyLock, vec};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use tauri::Manager;
use tokio_tungstenite::{self, connect_async};
use tungstenite::{http::Uri, ClientRequestBuilder};
mod api;
mod debouncer;
pub(crate) mod fstree;

pub static IS_CONNECTED: Mutex<bool> = Mutex::new(false);
pub static TRANSFERS: LazyLock<Mutex<HashMap<PathBuf, Transfer>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static WATCHER: Mutex<Option<RecommendedWatcher>> = Mutex::new(None);
pub fn start(app: tauri::AppHandle) {
    tokio::spawn(async move {
        let config = CONFIG.lock().unwrap().clone();
        let root_path = config.folder_path;

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Result<Event>>();
        let mut watcher = notify::recommended_watcher(move |res| {
            tx.send(res).unwrap();
        })
        .unwrap();
        let root_path = PathBuf::from(root_path);
        if !root_path.exists() {
            return;
        }
        let local_tree = Arc::new(Mutex::new(fstree::build_tree(&root_path).unwrap()));
        let _root_path = root_path.clone();
        let _local_tree = local_tree.clone();
        let _app = app.app_handle().clone();
        fstree::save_tree(&local_tree.lock().unwrap(), "tree.json").unwrap();
        let socket_task = async move {
            let local_tree = _local_tree.clone();
            let root_path = _root_path.clone();
            let app = _app.app_handle().clone();

            loop {
                let config = CONFIG.lock().unwrap().clone();
                let token = config.token.as_ref();
                if config.token.is_none() {
                    break;
                }
                let token = token.unwrap().value.clone();

                let socket_url = format!(
                    "{}/websocket",
                    config
                        .server_url
                        .replace("https://", "ws://")
                        .replace("http://", "ws://")
                );
                let uri: Uri = socket_url.parse().unwrap();

                let request = ClientRequestBuilder::new(uri).with_header("authorization", &token);
                match connect_async(request).await {
                    Ok((mut socket, _response)) => {
                        println!("Connected to server");
                        *IS_CONNECTED.lock().unwrap() = true;
                        app.emit_all("is_connected", true).unwrap();
                        while let Some(msg) = socket.next().await {
                            match msg {
                                Ok(tungstenite::Message::Text(text)) => {
                                    handle_msg(
                                        &local_tree,
                                        &root_path,
                                        &app,
                                        tungstenite::Message::Text(text),
                                    )
                                    .await;
                                }
                                Ok(_) => {}
                                Err(e) => {
                                    println!("WebSocket error: {}", e);
                                }
                            }
                        }
                        println!("Disconnected from server");
                    }
                    Err(e) => {
                        *IS_CONNECTED.lock().unwrap() = false;
                        app.emit_all("is_connected", false).unwrap();
                        println!("Failed to connect: {}", e);
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    }
                }
            }
        };

        let watcher_task = async move {
            watcher
                .watch(std::path::Path::new(&root_path), RecursiveMode::Recursive)
                .unwrap();
            WATCHER.lock().unwrap().replace(watcher);
            let debouncer = debouncer::Debouncer::new(std::time::Duration::from_millis(1000));

            while let Some(res) = rx.recv().await {
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
        };
        tokio::select! {
            _ = socket_task => {},
            _ = watcher_task => {},
        }
    });
}

async fn handle_msg(
    local_tree: &Arc<Mutex<fstree::Node>>,
    root_path: &PathBuf,
    app: &tauri::AppHandle,
    msg: tungstenite::Message,
) {
    let text = msg.to_string();
    println!("Received new tree");
    let mut changes: Vec<fstree::Change> = Vec::new();
    let resp: SocketResponse = serde_json::from_str(&text).unwrap();
    let mut remote_tree = resp.data;
    {
        let local = local_tree.lock().unwrap();
        fstree::diff_trees("", Some(&local), Some(&remote_tree), &mut changes);
    }
    let mut futures = vec![];
    let changes = fstree::detect_renames(changes);
    // println!("changes: {:?}", changes);
    for change in changes {
        println!(
            "remote nodeType:{:?} -> {:?}: {}",
            change.node_type, change.change_type, change.path,
        );
        match change.change_type {
            fstree::ChangeType::Added => match change.node_type {
                fstree::NodeType::File => {
                    futures.push(api::download(
                        app.app_handle().clone(),
                        root_path,
                        change.path,
                        change.id,
                        change.hash.unwrap(),
                        local_tree.clone(),
                    ));
                }
                fstree::NodeType::Folder => {
                    std::fs::create_dir_all(Path::new(root_path).join(&change.path)).unwrap();
                    let node =
                        fstree::build_node(&Path::new(root_path), &root_path.join(&change.path));
                    local_tree.lock().unwrap().add_node(node.unwrap()).unwrap();
                }
            },
            fstree::ChangeType::Deleted => match change.node_type {
                fstree::NodeType::File => {
                    let _ = std::fs::remove_file(Path::new(root_path).join(&change.path));

                    local_tree
                        .lock()
                        .unwrap()
                        .delete_node(root_path.join(&change.path).to_str().unwrap())
                        .unwrap();
                }
                fstree::NodeType::Folder => {
                    let _ = std::fs::remove_dir_all(Path::new(root_path).join(&change.path))
                        .map_err(|_| {
                            println!("failed to delete {:#?}", change.path);
                        });
                    local_tree
                        .lock()
                        .unwrap()
                        .delete_node(root_path.join(&change.path).to_str().unwrap())
                        .unwrap();
                }
            },
            fstree::ChangeType::Renamed { from } => {
                std::fs::rename(
                    Path::new(root_path).join(&from),
                    Path::new(root_path).join(&change.path),
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
            fstree::ChangeType::Modified => futures.push(api::download(
                app.app_handle().clone(),
                root_path,
                change.path,
                change.id,
                change.hash.unwrap(),
                local_tree.clone(),
            )),
        }
    }
    join_all(futures).await;
    let mut changes: Vec<fstree::Change> = Vec::new();
    fstree::diff_trees(
        "",
        Some(&local_tree.lock().unwrap()),
        Some(&remote_tree),
        &mut changes,
    );
    remote_tree.path = Some(root_path.to_str().unwrap().to_string());
    println!("changes: {:?}", changes);
    if changes.is_empty() {
        *local_tree.lock().unwrap() = remote_tree;

        fstree::save_tree(&local_tree.lock().unwrap(), "tree.json").unwrap();
    }
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
    match event.kind {
        EventKind::Remove(RemoveKind::File)
        | EventKind::Remove(RemoveKind::Folder)
        | EventKind::Remove(RemoveKind::Any) => {
            let _ = tree
                .lock()
                .unwrap()
                .delete_node(&event.paths[0].to_str().unwrap())
                .map_err(|_| println!("failed to delete {:#?}", event.paths[0].to_str().unwrap()));
        }
        EventKind::Modify(ModifyKind::Name(RenameMode::From)) => {
            tree.lock()
                .unwrap()
                .delete_node(&event.paths[0].to_str().unwrap())
                .unwrap();
        }
        EventKind::Modify(ModifyKind::Name(RenameMode::To))
        | EventKind::Modify(ModifyKind::Any)
        | EventKind::Access(AccessKind::Close(AccessMode::Write))
        | EventKind::Create(CreateKind::File)
        | EventKind::Create(CreateKind::Folder)
        | EventKind::Create(CreateKind::Any) => {
            std::thread::sleep(std::time::Duration::from_millis(100));
            let path = std::path::Path::new(event.paths[0].to_str().unwrap());
            if path.is_dir() {
                match event.kind {
                    EventKind::Create(CreateKind::Folder)
                    | EventKind::Create(CreateKind::Any)
                    | EventKind::Modify(ModifyKind::Name(RenameMode::To)) => {}
                    _ => return,
                }
            }
            let node = fstree::build_node(&root_path, path);

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

        let changes = fstree::detect_renames(changes);
        // println!("changes: {:?}", changes);
        for change in changes.clone() {
            println!(
                "local nodeType:{:?} -> {:?}: {}",
                change.node_type, change.change_type, change.path,
            );

            match change.change_type {
                fstree::ChangeType::Added => match change.node_type {
                    fstree::NodeType::File => api::upload(
                        app.clone(),
                        change.id,
                        change.parent_id,
                        &root_path,
                        &change.path,
                        tree.clone(),
                    ),
                    fstree::NodeType::Folder => {
                        api::create_folder(&change.path, change.id, change.parent_id)
                    }
                },
                fstree::ChangeType::Deleted => api::delete(change.id, &change.path),
                fstree::ChangeType::Renamed { from } => {
                    let node =
                        fstree::build_node(&Path::new(&root_path), &root_path.join(&change.path))
                            .unwrap();

                    api::rename(change.id.clone(), change.parent_id, &from, &change.path);
                    *node.id.lock().unwrap() = change.id.lock().unwrap().clone();
                    tree.lock().unwrap().add_node(node).unwrap();
                }
                fstree::ChangeType::Modified => api::upload(
                    app.clone(),
                    change.id,
                    change.parent_id,
                    &root_path,
                    &change.path,
                    tree.clone(),
                ),
            }
        }
        if !changes.is_empty() {
            fstree::save_tree(&tree.lock().unwrap(), "tree.json").unwrap();
        }
    })
}
