use anyhow::Result;
use uuid::Uuid;

use super::layer::LayerPath;
use super::store::HistoryStore;
use crate::ai::{AgentBlock, AgentBlockKind, AgentSession};

impl HistoryStore {
    /// Sync a terminal command+output pair into i/o tables.
    /// Timestamp is generated if not provided.
    /// Returns the dialogue_id used.
    pub fn sync_terminal_dialogue(
        &self,
        workspace: &str,
        session_id: &str,
        command: &str,
        prompt: &str,
        output: &str,
    ) -> Result<String> {
        let dialogue_id = Uuid::new_v4().to_string();
        let timestamp = chrono::Utc::now().to_rfc3339();
        let path = LayerPath {
            workspace: workspace.to_string(),
            source: "terminal".to_string(),
            session_id: session_id.to_string(),
            dialogue_id: dialogue_id.clone(),
        };

        let input_text = if prompt.is_empty() {
            command.to_string()
        } else if prompt.ends_with('\n') {
            format!("{prompt}{command}")
        } else {
            format!("{prompt} {command}")
        };

        self.insert_input(&path, &input_text, "command", &timestamp)?;

        if !output.trim().is_empty() {
            self.insert_output(&path, output, "text", &timestamp)?;
        }

        Ok(dialogue_id)
    }

    /// Sync an agent session's blocks into i/o tables.
    /// Groups blocks into dialogues (user+assistant turns).
    /// Returns the number of dialogues synced.
    pub fn sync_agent_session(
        &self,
        workspace: &str,
        source: &str,
        session_id: &str,
        session: &AgentSession,
    ) -> Result<usize> {
        let dialogues = split_agent_dialogues(&session.blocks);
        let mut count = 0;

        for (dlg_idx, dlg) in dialogues.iter().enumerate() {
            let dialogue_id = format!("{session_id}-{dlg_idx}");
            let path = LayerPath {
                workspace: workspace.to_string(),
                source: source.to_string(),
                session_id: session_id.to_string(),
                dialogue_id: dialogue_id.clone(),
            };

            // Skip if already synced (idempotent)
            let existing = self.list_dialogue_inputs(&path)?;
            if !existing.is_empty() {
                count += 1;
                continue;
            }

            let timestamp = dlg
                .first()
                .and_then(|b| b.timestamp.clone())
                .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

            let mut has_content = false;
            for block in dlg {
                let (content_type, is_input) = classify_agent_block(block);
                if is_input {
                    self.insert_input(&path, &block.text, content_type, &timestamp)?;
                    has_content = true;
                } else {
                    self.insert_output(&path, &block.text, content_type, &timestamp)?;
                    has_content = true;
                }
            }

            if has_content {
                count += 1;
            }
        }

        Ok(count)
    }

    /// Check if a session has been synced (has at least one dialogue in i/o tables).
    pub fn is_session_synced(&self, workspace: &str, source: &str, session_id: &str) -> Result<bool> {
        let sessions = self.list_sessions(workspace, source)?;
        Ok(sessions.iter().any(|s| s.id == session_id && s.dialogue_count > 0))
    }

    /// Count total dialogues across all synced sessions for a workspace+source.
    pub fn count_dialogues(&self, workspace: &str, source: &str) -> Result<i64> {
        let mut stmt = self.conn.prepare(
            "SELECT count(distinct dialogue_id) FROM input WHERE workspace = ?1 AND source = ?2",
        )?;
        Ok(stmt.query_row(rusqlite::params![workspace, source], |row| row.get(0))?)
    }
}

/// Split agent blocks into dialogue groups (one user+assistant turn each).
fn split_agent_dialogues(blocks: &[AgentBlock]) -> Vec<Vec<&AgentBlock>> {
    let mut dialogues: Vec<Vec<&AgentBlock>> = Vec::new();
    let mut current: Vec<&AgentBlock> = Vec::new();
    let mut seen_assistant = false;

    for block in blocks {
        if block.kind == AgentBlockKind::User && seen_assistant && !current.is_empty() {
            dialogues.push(std::mem::take(&mut current));
            seen_assistant = false;
        }
        if block.kind == AgentBlockKind::Assistant {
            seen_assistant = true;
        }
        current.push(block);
    }

    if !current.is_empty() {
        dialogues.push(current);
    }

    dialogues
}

/// Classify an agent block into (content_type, is_input).
fn classify_agent_block(block: &AgentBlock) -> (&'static str, bool) {
    match block.kind {
        AgentBlockKind::User => ("message", true),
        AgentBlockKind::Assistant => ("message", false),
        AgentBlockKind::ToolCall => ("tool_call", true),
        AgentBlockKind::ToolOutput => ("tool_output", false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::{AgentBlock, AgentBlockKind};
    use crate::history::HistoryStore;

    fn make_block(kind: AgentBlockKind, text: &str) -> AgentBlock {
        AgentBlock {
            kind,
            timestamp: Some("2026-05-22T00:00:00Z".to_string()),
            label: None,
            text: text.to_string(),
        }
    }

    #[test]
    fn syncs_terminal_dialogue_to_i_and_o_tables() {
        let store = HistoryStore::open_memory().unwrap();
        let dialogue_id = store
            .sync_terminal_dialogue(
                "/home/user/project",
                "sess-1",
                "cargo build",
                "❯ ",
                "Compiling sivtr v0.1.0\nFinished dev",
            )
            .unwrap();

        let path = crate::history::layer::LayerPath {
            workspace: "/home/user/project".to_string(),
            source: "terminal".to_string(),
            session_id: "sess-1".to_string(),
            dialogue_id,
        };
        let inputs = store.list_dialogue_inputs(&path).unwrap();
        let outputs = store.list_dialogue_outputs(&path).unwrap();

        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0].content, "❯ cargo build");
        assert_eq!(inputs[0].content_type, "command");
        assert_eq!(outputs.len(), 1);
        assert!(outputs[0].content.contains("Compiling"));
    }

    #[test]
    fn syncs_agent_session_blocks_into_dialogues() {
        let store = HistoryStore::open_memory().unwrap();
        let session = crate::ai::AgentSession {
            path: "test.jsonl".into(),
            id: Some("abc".to_string()),
            cwd: Some("/repo".to_string()),
            title: None,
            blocks: vec![
                make_block(AgentBlockKind::User, "hello"),
                make_block(AgentBlockKind::Assistant, "hi there"),
                make_block(AgentBlockKind::User, "do X"),
                make_block(AgentBlockKind::ToolCall, r#"{"command":"ls"}"#),
                make_block(AgentBlockKind::ToolOutput, "file.txt"),
                make_block(AgentBlockKind::Assistant, "found file.txt"),
            ],
        };

        let count = store
            .sync_agent_session("/repo", "codex", "abc", &session)
            .unwrap();

        assert_eq!(count, 2);

        // Dialogue 0: "hello" → "hi there"
        let path0 = crate::history::layer::LayerPath {
            workspace: "/repo".to_string(),
            source: "codex".to_string(),
            session_id: "abc".to_string(),
            dialogue_id: "abc-0".to_string(),
        };
        let inputs0 = store.list_dialogue_inputs(&path0).unwrap();
        let outputs0 = store.list_dialogue_outputs(&path0).unwrap();
        assert_eq!(inputs0.len(), 1);
        assert_eq!(outputs0.len(), 1);
        assert_eq!(inputs0[0].content, "hello");
        assert_eq!(outputs0[0].content, "hi there");

        // Dialogue 1: "do X" + tool → "found file.txt"
        let path1 = crate::history::layer::LayerPath {
            workspace: "/repo".to_string(),
            source: "codex".to_string(),
            session_id: "abc".to_string(),
            dialogue_id: "abc-1".to_string(),
        };
        let inputs1 = store.list_dialogue_inputs(&path1).unwrap();
        let outputs1 = store.list_dialogue_outputs(&path1).unwrap();
        assert_eq!(inputs1.len(), 2); // user message + tool call
        assert_eq!(outputs1.len(), 2); // tool output + assistant message
    }

    #[test]
    fn sync_is_idempotent() {
        let store = HistoryStore::open_memory().unwrap();
        let session = crate::ai::AgentSession {
            path: "test.jsonl".into(),
            id: Some("abc".to_string()),
            cwd: Some("/repo".to_string()),
            title: None,
            blocks: vec![
                make_block(AgentBlockKind::User, "q"),
                make_block(AgentBlockKind::Assistant, "a"),
            ],
        };

        let count1 = store
            .sync_agent_session("/repo", "claude", "abc", &session)
            .unwrap();
        assert_eq!(count1, 1);

        let count2 = store
            .sync_agent_session("/repo", "claude", "abc", &session)
            .unwrap();
        assert_eq!(count2, 1); // same count, no duplicates
    }

    #[test]
    fn is_session_synced_detects_existing_data() {
        let store = HistoryStore::open_memory().unwrap();
        assert!(!store.is_session_synced("/repo", "terminal", "s1").unwrap());

        store
            .sync_terminal_dialogue("/repo", "s1", "ls", "$ ", "file.txt")
            .unwrap();

        assert!(store.is_session_synced("/repo", "terminal", "s1").unwrap());
    }
}
