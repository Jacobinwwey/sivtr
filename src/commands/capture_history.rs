use anyhow::Result;
use sivtr_core::config::SivtrConfig;
use sivtr_core::history::{CaptureSource, HistoryStore};
use sivtr_core::session;

pub fn maybe_save_default(
    config: &SivtrConfig,
    content: &str,
    command: Option<&str>,
    source: CaptureSource,
) -> Result<()> {
    let store = HistoryStore::open_default()?;
    maybe_save(&store, config, content, command, source)
}

fn maybe_save(
    store: &HistoryStore,
    config: &SivtrConfig,
    content: &str,
    command: Option<&str>,
    source: CaptureSource,
) -> Result<()> {
    if !config.history.auto_save || content.trim().is_empty() {
        return Ok(());
    }

    store.insert(content, command, source)?;
    store.trim_to_max_entries(config.history.max_entries)?;
    Ok(())
}

/// Sync the terminal session log entries into i/o tables.
/// Reads the session JSONL log and persists each entry's input/output split.
pub fn sync_terminal_session_log(store: &HistoryStore) -> Result<usize> {
    let log_path = sivtr_core::capture::scrollback::session_log_path();
    if !log_path.exists() {
        return Ok(0);
    }

    let entries = session::load_entries(&log_path)?;
    let workspace = resolve_cwd();
    let session_id = "terminal-current".to_string();
    let mut synced = 0;

    for entry in &entries {
        match store.sync_terminal_dialogue(
            &workspace,
            &session_id,
            &entry.command,
            &entry.prompt,
            &entry.output,
        ) {
            Ok(_) => synced += 1,
            Err(e) => eprintln!("sivtr: failed to sync terminal dialogue: {e:#}"),
        }
    }

    Ok(synced)
}

fn resolve_cwd() -> String {
    std::env::current_dir()
        .ok()
        .and_then(|p| p.to_str().map(str::to_string))
        .unwrap_or_else(|| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saves_capture_when_auto_save_is_enabled() {
        let store = HistoryStore::open_memory().unwrap();
        let config = SivtrConfig::default();

        maybe_save(
            &store,
            &config,
            "captured output",
            Some("echo captured"),
            CaptureSource::Run,
        )
        .unwrap();

        let entries = store.list_recent(10).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].command.as_deref(), Some("echo captured"));
        assert_eq!(entries[0].source, "run");
    }

    #[test]
    fn skips_save_when_auto_save_is_disabled() {
        let store = HistoryStore::open_memory().unwrap();
        let mut config = SivtrConfig::default();
        config.history.auto_save = false;

        maybe_save(
            &store,
            &config,
            "captured output",
            Some("echo captured"),
            CaptureSource::Run,
        )
        .unwrap();

        let entries = store.list_recent(10).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn trims_history_to_configured_limit() {
        let store = HistoryStore::open_memory().unwrap();
        let mut config = SivtrConfig::default();
        config.history.max_entries = 1;

        maybe_save(
            &store,
            &config,
            "first output",
            Some("echo first"),
            CaptureSource::Run,
        )
        .unwrap();
        maybe_save(
            &store,
            &config,
            "second output",
            Some("echo second"),
            CaptureSource::Pipe,
        )
        .unwrap();

        let entries = store.list_recent(10).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].content, "second output");
        assert_eq!(entries[0].source, "pipe");
    }
}
