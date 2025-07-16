use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum NodeType {
    File,
    Folder,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Node {
    #[serde(rename = "type")]
    node_type: NodeType,
    hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<BTreeMap<String, Node>>,
    pub path: Option<String>, // relative from root
    pub id: Arc<Mutex<Option<String>>>,
    pub parent_id: Arc<Mutex<Option<String>>>,
}

pub fn hash_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

pub fn build_tree(path: &Path) -> std::io::Result<Node> {
    let relative = std::path::Path::new("");
    let mut root = _build_tree(path, relative)?;
    root.path = Some(path.to_string_lossy().to_string());
    Ok(root)
}
pub fn build_node(base_path: &Path, file_path: &Path) -> std::io::Result<Node> {
    let relative = file_path
        .strip_prefix(base_path)
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "File not under base path"))?;
    _build_tree(file_path, relative)
}

fn _build_tree(path: &Path, relative: &Path) -> std::io::Result<Node> {
    if path.is_file() {
        let data = match fs::read(path) {
            Ok(data) => data,
            Err(_) => {
                return Ok(Node {
                    node_type: NodeType::File,
                    hash: "".to_string(),
                    content: None,
                    path: Some(relative.to_string_lossy().to_string()),
                    id: Arc::new(Mutex::new(None)),
                    parent_id: Arc::new(Mutex::new(None)),
                })
            }
        };
        let hash = hash_bytes(&data);

        Ok(Node {
            node_type: NodeType::File,
            hash,
            content: None,
            path: Some(relative.to_string_lossy().to_string()),
            id: Arc::new(Mutex::new(None)),
            parent_id: Arc::new(Mutex::new(None)),
        })
    } else if path.is_dir() {
        let mut children: BTreeMap<String, Node> = BTreeMap::new();

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let name = entry.file_name();
            let full_path = entry.path();
            let rel_path = relative.join(&name);

            let node = _build_tree(&full_path, &rel_path)?;
            children.insert(name.to_string_lossy().to_string(), node);
        }

        let mut hash_input = String::new();
        for (_, child) in &children {
            hash_input.push_str(&child.hash);
        }

        let folder_hash = hash_bytes(hash_input.as_bytes());

        Ok(Node {
            node_type: NodeType::Folder,
            hash: folder_hash,
            content: Some(children),
            path: Some(relative.to_string_lossy().to_string()),
            id: Arc::new(Mutex::new(None)),
            parent_id: Arc::new(Mutex::new(None)),
        })
    } else {
        println!("Unsupported file type: {}", path.display());
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Unsupported file type",
        ))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ChangeType {
    Added,
    Deleted,
    Modified,
    Renamed { from: String },
}

#[derive(Debug, Clone)]
pub struct Change {
    pub id: Arc<Mutex<Option<String>>>,
    pub parent_id: Arc<Mutex<Option<String>>>,
    pub node_type: NodeType,
    pub path: String, // 'to' path if renamed
    pub change_type: ChangeType,
    pub hash: Option<String>, // for comparing during rename detection
}

pub fn diff_trees(
    path: &str,
    node_1: Option<&Node>,
    node_2: Option<&Node>,
    changes: &mut Vec<Change>,
) {
    match (node_1, node_2) {
        (Some(node_1), Some(node_2)) => {
            if node_1.node_type != node_2.node_type {
                changes.push(Change {
                    id: node_1.id.clone(),
                    parent_id: node_1.parent_id.clone(),
                    node_type: node_2.node_type.clone(),
                    path: path.to_string(),
                    change_type: ChangeType::Modified,
                    hash: Some(node_2.hash.clone()),
                });
            }
            if node_1.hash == node_2.hash {}
            if node_1.hash != node_2.hash
                && node_1.node_type == NodeType::File
                && node_2.node_type == NodeType::File
            {
                *node_2.id.lock().unwrap() = node_1.id.lock().unwrap().clone();
                changes.push(Change {
                    id: node_2.id.clone(),
                    parent_id: node_1.parent_id.clone(),
                    node_type: node_2.node_type.clone(),
                    path: path.to_string(),
                    change_type: ChangeType::Modified,
                    hash: Some(node_2.hash.clone()),
                });
            }

            if let (Some(old_children), Some(new_children)) = (&node_1.content, &node_2.content) {
                let all_keys: std::collections::BTreeSet<_> =
                    old_children.keys().chain(new_children.keys()).collect();

                for key in all_keys {
                    let old_child = old_children.get(key);
                    let new_child = new_children.get(key);
                    let full_path = if path.is_empty() {
                        key.clone()
                    } else {
                        Path::new(path).join(key).to_str().unwrap().to_string()
                    };
                    diff_trees(&full_path, old_child, new_child, changes);
                }
            }
        }
        (None, Some(new)) => {
            changes.push(Change {
                id: new.id.clone(),
                parent_id: new.parent_id.clone(),
                node_type: new.node_type.clone(),
                path: path.to_string(),
                change_type: ChangeType::Added,
                hash: Some(new.hash.clone()),
            });
            if let Some(children) = &new.content {
                for (child_name, child_node) in children {
                    let child_path = if path.is_empty() {
                        child_name.clone()
                    } else {
                        Path::new(path)
                            .join(child_name)
                            .to_str()
                            .unwrap()
                            .to_string()
                    };
                    diff_trees(&child_path, None, Some(child_node), changes);
                }
            }
        }
        (Some(old), None) => {
            changes.push(Change {
                id: old.id.clone(),
                parent_id: old.parent_id.clone(),
                node_type: old.node_type.clone(),
                path: path.to_string(),
                change_type: ChangeType::Deleted,
                hash: Some(old.hash.clone()),
            });
        }

        (None, None) => {}
    }
    sort_changes(changes);
}

pub fn detect_renames(mut changes: Vec<Change>) -> Vec<Change> {
    let mut final_changes = Vec::new();
    let mut added = Vec::new();
    let mut deleted = Vec::new();

    // Separate out added and deleted
    changes.retain(|change| {
        match change.change_type {
            ChangeType::Added => {
                added.push(change.clone());
                false
            }
            ChangeType::Deleted => {
                deleted.push(change.clone());
                false
            }
            _ => true, // keep modified and renamed (if already present)
        }
    });

    // Match added <-> deleted by hash
    while let Some(add) = added.pop() {
        if let Some(add_hash) = &add.hash {
            if let Some(pos) = deleted.iter().position(|del| {
                del.node_type == add.node_type && del.hash == Some(add_hash.clone())
            }) {
                let del = deleted.remove(pos);
                *add.id.lock().unwrap() = del.id.lock().unwrap().clone();
                final_changes.push(Change {
                    id: add.id.clone(),
                    parent_id: add.parent_id.clone(),
                    node_type: add.node_type.clone(),
                    path: add.path,
                    change_type: ChangeType::Renamed { from: del.path },
                    hash: Some(add_hash.clone()),
                });
                continue;
            }
        }
        // no match → still an Add, real addition
        final_changes.push(add);
    }

    // Remaining unmatched deletions,real deletions
    final_changes.extend(deleted);

    // Remaining unchanged changes (e.g., Modified)
    final_changes.extend(changes);

    final_changes
}

impl Node {
    /// High-level convenience: pass in a node with a known relative path

    pub fn delete_node(&mut self, file_path: &str) -> Result<(), String> {
        let relative = Path::new(file_path)
            .strip_prefix(self.path.as_ref().unwrap())
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "File not under base path"))
            .unwrap();
        let parts: Vec<String> = Path::new(relative)
            .components()
            .map(|c| c.as_os_str().to_string_lossy().to_string())
            .collect();

        self._delete_path(&parts)
    }

    fn _delete_path(&mut self, parts: &[String]) -> Result<(), String> {
        if parts.is_empty() {
            return Err("Cannot delete root node".to_string());
        }

        match &mut self.content {
            Some(children) => {
                let key = &parts[0];

                if parts.len() == 1 {
                    // Delete the target child
                    let _ = children.remove(key);
                } else {
                    // Recurse into subfolder
                    let child = children
                        .get_mut(key)
                        .ok_or_else(|| format!("Folder '{}' not found", key))?;

                    if child.node_type != NodeType::Folder {
                        return Err(format!("Expected folder at '{}'", key));
                    }

                    child._delete_path(&parts[1..])?;
                }

                // Recalculate hash of this folder after deletion
                self.recalculate_hash().unwrap();

                Ok(())
            }
            None => Err("Cannot delete inside a file node".to_string()),
        }
    }

    pub fn add_node(&mut self, new_node: Node) -> Result<(), String> {
        let rel_path = new_node.path.as_ref().ok_or("Node is missing path")?;
        let parts: Vec<String> = Path::new(rel_path)
            .components()
            .map(|c| c.as_os_str().to_string_lossy().to_string())
            .collect();

        self._add_path(&parts, new_node)
    }

    fn _add_path(&mut self, parts: &[String], mut new_node: Node) -> Result<(), String> {
        if parts.is_empty() {
            return Err("Cannot add node at empty path".into());
        }
        match &mut self.content {
            Some(children) => {
                let key = &parts[0];

                if parts.len() == 1 {
                    // Insert the new node
                    new_node.parent_id = self.id.clone();
                    new_node.id = children
                        .get(key)
                        .map_or_else(|| new_node.id, |n| n.id.clone());
                    children.insert(key.clone(), new_node);
                } else {
                    // Recurse into subfolder
                    let child = children.entry(key.to_string()).or_insert_with(|| Node {
                        node_type: NodeType::Folder,
                        hash: "".to_string(),
                        content: Some(BTreeMap::new()),
                        path: Some(
                            self.path
                                .as_ref()
                                .map(|p| Path::new(p).join(key).to_str().unwrap().to_string())
                                .unwrap_or_else(|| key.clone()),
                        ),
                        id: Arc::new(Mutex::new(None)),
                        parent_id: Arc::new(Mutex::new(None)),
                    });

                    if child.node_type != NodeType::Folder {
                        return Err(format!("Expected folder at '{}'", key));
                    }

                    child._add_path(&parts[1..], new_node)?;
                }

                // Recalculate hash after inserting
                self.recalculate_hash().unwrap();
                Ok(())
            }
            None => Err("Cannot add node inside a file".into()),
        }
    }

    pub fn rename_node(&mut self, old_path: &str, new_path: &str) -> Result<(), String> {
        self.delete_node(old_path)?;
        let new_node = build_node(
            std::path::Path::new(self.path.as_ref().unwrap()),
            std::path::Path::new(new_path),
        )
        .unwrap();
        self.add_node(new_node)?;
        Ok(())
    }
    fn recalculate_hash(&mut self) -> std::io::Result<()> {
        if self.node_type != NodeType::Folder || self.content.is_none() {
            return Ok(());
        }
        let mut hash_input = String::new();
        for (_, child) in self.content.as_mut().unwrap() {
            hash_input.push_str(&child.hash);
        }
        self.hash = hash_bytes(hash_input.as_bytes());
        Ok(())
    }
}

pub fn save_tree(node: &Node, path: &str) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(node)?;
    fs::write(path, json)?;
    Ok(())
}

fn sort_changes(changes: &mut Vec<Change>) {
    changes.sort_by(|a, b| {
        // 1. Folder before file
        let node_type_cmp = match (&a.node_type, &b.node_type) {
            (NodeType::Folder, NodeType::File) => std::cmp::Ordering::Greater,
            (NodeType::File, NodeType::Folder) => std::cmp::Ordering::Less,
            _ => std::cmp::Ordering::Equal,
        };

        if node_type_cmp != std::cmp::Ordering::Equal {
            return node_type_cmp;
        }

        // 2. Depth (shorter path first) — assumes UNIX-style paths
        let a_depth = a.path.matches(&['/', '\\'][..]).count();
        let b_depth = b.path.matches(&['/', '\\'][..]).count();

        b_depth.cmp(&a_depth)
    });
}
