use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use sivtr_core::capture::scrollback;
use sivtr_core::workspace;

/// Clear the current session's log and state files.
pub fn execute(clear_all: bool) -> Result<()> {
    if clear_all {
        let removed = clear_all_sessions()?;
        if removed > 0 {
            eprintln!("sivtr: cleared {removed} session file(s)");
        } else {
            eprintln!("sivtr: no session files to clear");
        }
        return Ok(());
    }

    let log = scrollback::session_log_path();
    let state = scrollback::flush_state_path();
    let capture = scrollback::capture_file_path();

    let mut cleared = false;

    if log.exists() {
        fs::remove_file(&log)?;
        cleared = true;
    }
    for f in [&state, &capture] {
        if f.exists() {
            let _ = fs::remove_file(f);
        }
    }

    if cleared {
        eprintln!("sivtr: session cleared ({})", log.display());
    } else {
        eprintln!("sivtr: no session to clear");
    }

    Ok(())
}

fn clear_all_sessions() -> Result<usize> {
    let mut removed = 0usize;
    let mut seen = Vec::<PathBuf>::new();

    for path in legacy_session_artifacts() {
        if path.is_file() && !seen.iter().any(|seen_path| seen_path == &path) {
            fs::remove_file(&path)?;
            removed += 1;
            seen.push(path);
        }
    }

    let workspaces_dir = workspace::data_dir().join("workspaces");
    if workspaces_dir.exists() {
        fs::remove_dir_all(&workspaces_dir).context("Failed to clear workspace session files")?;
        removed += 1;
    }

    Ok(removed)
}

fn legacy_session_artifacts() -> Vec<PathBuf> {
    let mut artifacts = Vec::new();
    for dir in candidate_session_dirs() {
        if !dir.exists() {
            continue;
        }
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if is_session_artifact(&path) {
                    artifacts.push(path);
                }
            }
        }
    }
    artifacts
}

fn candidate_session_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(config_dir) = dirs::config_dir() {
        dirs.push(config_dir.join("sivtr"));
        dirs.push(config_dir.join("sift"));
    }
    dirs
}

fn is_session_artifact(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };

    (name == "session.log" || name == "session.state")
        || (name.starts_with("session_")
            && (name.ends_with(".log") || name.ends_with(".state") || name.ends_with(".capture")))
}

#[cfg(test)]
mod tests {
    use super::is_session_artifact;
    use std::path::Path;

    #[test]
    fn detects_session_artifacts_for_clear_all() {
        assert!(is_session_artifact(Path::new("session.log")));
        assert!(is_session_artifact(Path::new("session.state")));
        assert!(is_session_artifact(Path::new("session_1234.log")));
        assert!(is_session_artifact(Path::new("session_1234.state")));
        assert!(is_session_artifact(Path::new("session_1234.capture")));
        assert!(!is_session_artifact(Path::new("config.toml")));
        assert!(!is_session_artifact(Path::new("history.db")));
    }
}
