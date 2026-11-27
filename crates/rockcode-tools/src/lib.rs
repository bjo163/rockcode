use std::{
    fs,
    io::Read,
    path::{Component, Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use rockcode_kernel::{KernelError, ToolDefinition, ToolInvoker};
use serde::Serialize;
use serde_json::{json, Value};
use thiserror::Error;

pub const READ_FILE_TOOL: &str = "workspace.read_file";
pub const LIST_FILES_TOOL: &str = "workspace.list_files";
const MAX_READ_FILE_BYTES: u64 = 512 * 1024;
const MAX_DIRECTORY_ENTRIES: usize = 200;

#[derive(Debug, Error)]
pub enum WorkspaceToolsError {
    #[error("workspace root must be an existing directory")]
    InvalidRoot,
}

#[derive(Clone)]
pub struct WorkspaceToolInvoker {
    root: Arc<PathBuf>,
}

#[derive(Debug, Serialize)]
struct DirectoryEntry {
    name: String,
    kind: &'static str,
    size_bytes: Option<u64>,
}

#[derive(Debug, Serialize)]
struct DirectoryListing {
    path: String,
    entries: Vec<DirectoryEntry>,
    truncated: bool,
}

impl WorkspaceToolInvoker {
    pub fn new(root: impl AsRef<Path>) -> Result<Self, WorkspaceToolsError> {
        let canonical_root = fs::canonicalize(root).map_err(|_| WorkspaceToolsError::InvalidRoot)?;
        if !canonical_root.is_dir() {
            return Err(WorkspaceToolsError::InvalidRoot);
        }

        Ok(Self {
            root: Arc::new(canonical_root),
        })
    }

    pub fn root(&self) -> &Path {
        self.root.as_path()
    }
}

pub fn workspace_tool_definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: READ_FILE_TOOL.to_string(),
            description: "Read a UTF-8 text file inside the selected workspace. Sensitive credential paths are blocked and files are limited to 512 KiB.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "minLength": 1,
                        "description": "Workspace-relative file path; absolute paths and parent traversal are rejected."
                    }
                },
                "required": ["path"],
                "additionalProperties": false
            }),
        },
        ToolDefinition {
            name: LIST_FILES_TOOL.to_string(),
            description: "List up to 200 immediate children of a directory inside the selected workspace. Symlinks and common credential paths are omitted.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Workspace-relative directory; omit or use . for the workspace root."
                    }
                },
                "additionalProperties": false
            }),
        },
    ]
}

#[async_trait]
impl ToolInvoker for WorkspaceToolInvoker {
    async fn invoke(&self, name: &str, args: &str) -> Result<String, KernelError> {
        let root = Arc::clone(&self.root);
        let tool_name = name.to_string();
        let arguments = args.to_string();

        tokio::task::spawn_blocking(move || execute_tool(root.as_path(), &tool_name, &arguments))
            .await
            .map_err(|_| KernelError::Tool)?
    }
}

fn execute_tool(root: &Path, name: &str, args: &str) -> Result<String, KernelError> {
    let arguments = serde_json::from_str::<Value>(args).map_err(|_| KernelError::Tool)?;
    if !arguments.is_object() {
        return Err(KernelError::Tool);
    }

    match name {
        READ_FILE_TOOL => read_file(root, &arguments),
        LIST_FILES_TOOL => list_files(root, &arguments),
        _ => Err(KernelError::Tool),
    }
}

fn read_file(root: &Path, arguments: &Value) -> Result<String, KernelError> {
    let requested_path = arguments
        .get("path")
        .and_then(Value::as_str)
        .ok_or(KernelError::Tool)?;
    let relative = validate_relative_path(requested_path, false)?;
    if is_sensitive_path(&relative) {
        return Err(KernelError::Tool);
    }

    let canonical = canonicalize_inside_root(root, &relative)?;
    let canonical_relative = canonical.strip_prefix(root).map_err(|_| KernelError::Tool)?;
    if is_sensitive_path(canonical_relative) {
        return Err(KernelError::Tool);
    }

    let file = fs::File::open(&canonical).map_err(|_| KernelError::Tool)?;
    let metadata = file.metadata().map_err(|_| KernelError::Tool)?;
    if !metadata.is_file() || metadata.len() > MAX_READ_FILE_BYTES {
        return Err(KernelError::Tool);
    }

    let mut content = String::new();
    let bytes_read = file
        .take(MAX_READ_FILE_BYTES + 1)
        .read_to_string(&mut content)
        .map_err(|_| KernelError::Tool)?;
    if bytes_read as u64 > MAX_READ_FILE_BYTES {
        return Err(KernelError::Tool);
    }
    Ok(content)
}

fn list_files(root: &Path, arguments: &Value) -> Result<String, KernelError> {
    let requested_path = match arguments.get("path") {
        None => ".",
        Some(Value::String(path)) => path.as_str(),
        Some(_) => return Err(KernelError::Tool),
    };
    let relative = validate_relative_path(requested_path, true)?;
    if is_sensitive_path(&relative) {
        return Err(KernelError::Tool);
    }

    let directory = if relative.as_os_str().is_empty() {
        root.to_path_buf()
    } else {
        canonicalize_inside_root(root, &relative)?
    };
    let canonical_relative = directory.strip_prefix(root).map_err(|_| KernelError::Tool)?;
    if is_sensitive_path(canonical_relative) || !directory.is_dir() {
        return Err(KernelError::Tool);
    }

    let display_path = if canonical_relative.as_os_str().is_empty() {
        ".".to_string()
    } else {
        canonical_relative.to_string_lossy().replace('\\', "/")
    };

    let mut entries = Vec::new();
    let mut truncated = false;
    for entry in fs::read_dir(&directory).map_err(|_| KernelError::Tool)? {
        let entry = entry.map_err(|_| KernelError::Tool)?;
        let file_type = entry.file_type().map_err(|_| KernelError::Tool)?;
        if file_type.is_symlink() {
            continue;
        }

        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if is_sensitive_name(name) {
            continue;
        }

        let (kind, size_bytes) = if file_type.is_dir() {
            ("directory", None)
        } else if file_type.is_file() {
            let size = entry.metadata().map_err(|_| KernelError::Tool)?.len();
            ("file", Some(size))
        } else {
            continue;
        };

        if entries.len() == MAX_DIRECTORY_ENTRIES {
            truncated = true;
            break;
        }

        entries.push(DirectoryEntry {
            name: name.to_string(),
            kind,
            size_bytes,
        });
    }

    entries.sort_by(|left, right| left.name.cmp(&right.name));
    let listing = DirectoryListing {
        path: display_path,
        entries,
        truncated,
    };

    serde_json::to_string(&listing).map_err(|_| KernelError::Tool)
}

fn validate_relative_path(value: &str, allow_workspace_root: bool) -> Result<PathBuf, KernelError> {
    if allow_workspace_root && (value.is_empty() || value == ".") {
        return Ok(PathBuf::new());
    }
    if value.is_empty() {
        return Err(KernelError::Tool);
    }

    let path = Path::new(value);
    if path.is_absolute() {
        return Err(KernelError::Tool);
    }

    for component in path.components() {
        match component {
            Component::Normal(_) => {}
            Component::CurDir if allow_workspace_root && path.components().count() == 1 => {
                return Ok(PathBuf::new());
            }
            _ => return Err(KernelError::Tool),
        }
    }

    Ok(path.to_path_buf())
}

fn canonicalize_inside_root(root: &Path, relative: &Path) -> Result<PathBuf, KernelError> {
    let canonical = fs::canonicalize(root.join(relative)).map_err(|_| KernelError::Tool)?;
    if !canonical.starts_with(root) {
        return Err(KernelError::Tool);
    }
    Ok(canonical)
}

fn is_sensitive_path(path: &Path) -> bool {
    path.components().any(|component| match component {
        Component::Normal(name) => name.to_str().map(is_sensitive_name).unwrap_or(true),
        _ => false,
    })
}

fn is_sensitive_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower == ".git"
        || lower == ".ssh"
        || lower == ".aws"
        || lower == ".gnupg"
        || lower == ".config"
        || lower == ".env"
        || lower.starts_with(".env.")
        || lower == ".npmrc"
        || lower == ".pypirc"
        || lower == ".netrc"
        || lower.starts_with("credentials")
        || lower.starts_with("secrets")
        || lower == "id_rsa"
        || lower == "id_ed25519"
        || [".pem", ".key", ".p12", ".pfx", ".jks", ".keystore"]
            .iter()
            .any(|extension| lower.ends_with(extension))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::time::{SystemTime, UNIX_EPOCH};
    use uuid::Uuid;

    struct TempWorkspace {
        path: PathBuf,
    }

    impl TempWorkspace {
        fn new() -> Self {
            let nonce = format!(
                "{}-{}",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
                Uuid::new_v4()
            );
            let path = std::env::temp_dir().join(format!("rockcode-workspace-{nonce}"));
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }

        fn write(&self, relative: &str, content: &str) {
            let path = self.path.join(relative);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(path, content).unwrap();
        }
    }

    impl Drop for TempWorkspace {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[tokio::test]
    async fn reads_utf8_file_inside_workspace() {
        let workspace = TempWorkspace::new();
        workspace.write("src/lib.rs", "pub fn ready() -> bool { true }");
        let invoker = WorkspaceToolInvoker::new(&workspace.path).unwrap();

        let result = invoker
            .invoke(READ_FILE_TOOL, r#"{"path":"src/lib.rs"}"#)
            .await
            .unwrap();

        assert_eq!(result, "pub fn ready() -> bool { true }");
    }

    #[tokio::test]
    async fn rejects_parent_and_absolute_path_traversal() {
        let workspace = TempWorkspace::new();
        let parent = workspace.path.parent().unwrap();
        let outside = parent.join("rockcode-outside-test.txt");
        fs::write(&outside, "outside secret").unwrap();
        let invoker = WorkspaceToolInvoker::new(&workspace.path).unwrap();

        let parent_result = invoker
            .invoke(READ_FILE_TOOL, r#"{"path":"../rockcode-outside-test.txt"}"#)
            .await;
        let absolute_args = json!({"path": outside.to_string_lossy()}).to_string();
        let absolute_result = invoker.invoke(READ_FILE_TOOL, &absolute_args).await;

        assert!(parent_result.is_err());
        assert!(absolute_result.is_err());
        let _ = fs::remove_file(outside);
    }

    #[tokio::test]
    async fn blocks_common_secret_paths() {
        let workspace = TempWorkspace::new();
        workspace.write(".env", "API_KEY=do-not-read");
        workspace.write("config/credentials.json", r#"{"token":"do-not-read"}"#);
        let invoker = WorkspaceToolInvoker::new(&workspace.path).unwrap();

        assert!(invoker
            .invoke(READ_FILE_TOOL, r#"{"path":".env"}"#)
            .await
            .is_err());
        assert!(invoker
            .invoke(READ_FILE_TOOL, r#"{"path":"config/credentials.json"}"#)
            .await
            .is_err());

        let listing = invoker.invoke(LIST_FILES_TOOL, "{}").await.unwrap();
        assert!(!listing.contains(".env"));
        assert!(!listing.contains("credentials.json"));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn rejects_symlink_that_targets_outside_workspace() {
        use std::os::unix::fs::symlink;

        let workspace = TempWorkspace::new();
        let outside = workspace.path.parent().unwrap().join("rockcode-symlink-outside.txt");
        fs::write(&outside, "outside").unwrap();
        symlink(&outside, workspace.path.join("linked.txt")).unwrap();
        let invoker = WorkspaceToolInvoker::new(&workspace.path).unwrap();

        assert!(invoker
            .invoke(READ_FILE_TOOL, r#"{"path":"linked.txt"}"#)
            .await
            .is_err());

        let _ = fs::remove_file(outside);
    }

    #[tokio::test]
    async fn lists_sorted_immediate_entries_and_marks_truncation() {
        let workspace = TempWorkspace::new();
        workspace.write("zeta.txt", "z");
        workspace.write("alpha.txt", "a");
        workspace.write("folder/nested.txt", "nested");
        let invoker = WorkspaceToolInvoker::new(&workspace.path).unwrap();

        let listing = invoker.invoke(LIST_FILES_TOOL, "{}").await.unwrap();
        let parsed: Value = serde_json::from_str(&listing).unwrap();
        let names = parsed["entries"]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| entry["name"].as_str().unwrap())
            .collect::<Vec<_>>();

        assert_eq!(names, vec!["alpha.txt", "folder", "zeta.txt"]);
        assert_eq!(parsed["path"], ".");
        assert_eq!(parsed["truncated"], false);
    }

    #[tokio::test]
    async fn truncates_large_directory_listings() {
        let workspace = TempWorkspace::new();
        for index in 0..(MAX_DIRECTORY_ENTRIES + 5) {
            workspace.write(&format!("file-{index:03}.txt"), "x");
        }
        let invoker = WorkspaceToolInvoker::new(&workspace.path).unwrap();

        let listing = invoker.invoke(LIST_FILES_TOOL, "{}").await.unwrap();
        let parsed: Value = serde_json::from_str(&listing).unwrap();

        assert_eq!(
            parsed["entries"].as_array().unwrap().len(),
            MAX_DIRECTORY_ENTRIES
        );
        assert_eq!(parsed["truncated"], true);
    }

    #[tokio::test]
    async fn rejects_oversized_and_unknown_tool_requests() {
        let workspace = TempWorkspace::new();
        let large = "x".repeat((MAX_READ_FILE_BYTES + 1) as usize);
        workspace.write("large.txt", &large);
        let invoker = WorkspaceToolInvoker::new(&workspace.path).unwrap();

        assert!(invoker
            .invoke(READ_FILE_TOOL, r#"{"path":"large.txt"}"#)
            .await
            .is_err());
        assert!(invoker.invoke("workspace.execute", "{}").await.is_err());
    }

    #[test]
    fn publishes_only_read_only_tool_definitions() {
        let definitions = workspace_tool_definitions();
        assert_eq!(definitions.len(), 2);
        assert_eq!(definitions[0].name, READ_FILE_TOOL);
        assert_eq!(definitions[1].name, LIST_FILES_TOOL);
        assert!(definitions.iter().all(|definition| {
            definition.name != "workspace.write_file"
                && definition.name != "workspace.execute"
        }));
    }
}
