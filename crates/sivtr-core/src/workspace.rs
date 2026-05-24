use anyhow::{Context, Result};
use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const WORKSPACES_DIR: &str = "workspaces";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceMetadata {
    pub key: String,
    pub root: String,
    pub created_at: String,
    pub last_seen_at: String,
}

#[derive(Debug, Clone)]
pub struct WorkspacePaths {
    pub key: String,
    pub root: PathBuf,
    pub dir: PathBuf,
    pub terminals_dir: PathBuf,
}

pub fn resolve_current_workspace() -> Result<Option<WorkspacePaths>> {
    let cwd = std::env::current_dir().context("Failed to resolve current directory")?;
    resolve_workspace_for_dir(&cwd)
}

pub fn resolve_workspace_for_dir(cwd: &Path) -> Result<Option<WorkspacePaths>> {
    let Some(root) = git_root(cwd)? else {
        return Ok(None);
    };
    Ok(Some(paths_for_root(root)?))
}

pub fn ensure_current_workspace() -> Result<Option<WorkspacePaths>> {
    let Some(paths) = resolve_current_workspace()? else {
        return Ok(None);
    };
    ensure_workspace_metadata(&paths)?;
    fs::create_dir_all(&paths.terminals_dir)?;
    Ok(Some(paths))
}

pub fn data_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("sivtr")
}

pub fn legacy_session_dir() -> PathBuf {
    data_dir()
}

pub fn terminal_id() -> String {
    std::env::var("SIVTR_TERMINAL_ID")
        .ok()
        .filter(|id| !id.trim().is_empty())
        .unwrap_or_else(|| format!("session_{}", std::process::id()))
}

pub fn current_terminal_log_path() -> Result<Option<PathBuf>> {
    let Some(paths) = ensure_current_workspace()? else {
        return Ok(None);
    };
    Ok(Some(
        paths.terminals_dir.join(format!("{}.jsonl", terminal_id())),
    ))
}

pub fn current_terminal_state_path() -> Result<Option<PathBuf>> {
    Ok(current_terminal_log_path()?.map(|path| path.with_extension("state")))
}

pub fn current_terminal_capture_path() -> Result<Option<PathBuf>> {
    Ok(current_terminal_log_path()?.map(|path| path.with_extension("capture")))
}

pub fn terminal_log_paths_for_workspace(cwd: &Path) -> Result<Vec<PathBuf>> {
    let Some(paths) = resolve_workspace_for_dir(cwd)? else {
        return Ok(Vec::new());
    };
    if !paths.terminals_dir.exists() {
        return Ok(Vec::new());
    }

    let mut logs = Vec::new();
    for entry in fs::read_dir(&paths.terminals_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("jsonl") {
            logs.push(path);
        }
    }
    logs.sort_by_key(|path| std::cmp::Reverse(modified_time(path)));
    Ok(logs)
}

pub fn terminal_session_id_from_path(path: &Path) -> String {
    path.file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("terminal")
        .to_string()
}

pub fn terminal_log_path_for_root(root: &Path) -> Result<PathBuf> {
    let paths = paths_for_root(root.to_path_buf())?;
    Ok(paths.terminals_dir.join(format!("{}.jsonl", terminal_id())))
}

fn paths_for_root(root: PathBuf) -> Result<WorkspacePaths> {
    let root = canonicalize_lossy(&root);
    let root_text = root.to_string_lossy().to_string();
    let key = workspace_key(&root_text);
    let dir = data_dir().join(WORKSPACES_DIR).join(&key);
    Ok(WorkspacePaths {
        key,
        root,
        terminals_dir: dir.join("terminals"),
        dir,
    })
}

fn ensure_workspace_metadata(paths: &WorkspacePaths) -> Result<()> {
    fs::create_dir_all(&paths.dir)?;
    let path = paths.dir.join("workspace.json");
    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let created_at = if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|content| serde_json::from_str::<WorkspaceMetadata>(&content).ok())
            .map(|meta| meta.created_at)
            .unwrap_or_else(|| now.clone())
    } else {
        now.clone()
    };

    let metadata = WorkspaceMetadata {
        key: paths.key.clone(),
        root: paths.root.to_string_lossy().to_string(),
        created_at,
        last_seen_at: now,
    };
    fs::write(path, serde_json::to_string_pretty(&metadata)?)?;
    Ok(())
}

fn git_root(cwd: &Path) -> Result<Option<PathBuf>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(cwd)
        .arg("rev-parse")
        .arg("--show-toplevel")
        .output();

    let Ok(output) = output else {
        return Ok(None);
    };
    if !output.status.success() {
        return Ok(None);
    }

    let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if root.is_empty() {
        return Ok(None);
    }
    Ok(Some(PathBuf::from(root)))
}

fn canonicalize_lossy(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn workspace_key(root: &str) -> String {
    let normalized = root.replace('\\', "/").to_lowercase();
    let hash = fnv1a64(normalized.as_bytes());
    format!("{hash:016x}")
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn modified_time(path: &Path) -> std::time::SystemTime {
    fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
}

#[cfg(test)]
mod tests {
    use super::{terminal_session_id_from_path, workspace_key};
    use std::path::Path;

    #[test]
    fn workspace_key_normalizes_case_and_separators() {
        assert_eq!(workspace_key("D:\\sivtr"), workspace_key("d:/sivtr"));
    }

    #[test]
    fn terminal_session_id_uses_file_stem() {
        assert_eq!(
            terminal_session_id_from_path(Path::new("session_123.jsonl")),
            "session_123"
        );
    }
}
