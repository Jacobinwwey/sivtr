use serde::Serialize;
use sivtr_core::record::{WorkChannel, WorkPartIo, WorkPartKind, WorkRecord, WorkRefTarget};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkJsonSessionMeta {
    #[serde(rename = "ref")]
    pub ref_: String,
    pub channel: WorkChannel,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    pub display_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub canonical_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkJsonTargetMeta {
    #[serde(rename = "type")]
    pub type_: &'static str,
    pub record_ref: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part: Option<WorkJsonPartMeta>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkJsonPartMeta {
    #[serde(rename = "ref")]
    pub ref_: String,
    pub io: WorkPartIo,
    pub kind: WorkPartKind,
    pub index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

fn session_ref(record: &WorkRecord) -> String {
    match &record.work_ref {
        sivtr_core::record::WorkRef::Terminal { session, .. } => format!("terminal/{session}"),
        sivtr_core::record::WorkRef::Agent {
            provider, session, ..
        } => {
            format!("{}/{session}", provider.command_name())
        }
    }
}

pub fn session_meta(record: &WorkRecord) -> WorkJsonSessionMeta {
    WorkJsonSessionMeta {
        ref_: session_ref(record),
        channel: record.source.channel,
        provider: record
            .work_ref
            .provider()
            .map(|provider| provider.command_name().to_string()),
        display_id: record.session.id.clone(),
        canonical_id: record.session.canonical_id.clone(),
        path: record.session.path.clone(),
    }
}

pub fn target_meta(record: &WorkRecord, target: WorkRefTarget) -> WorkJsonTargetMeta {
    match target {
        WorkRefTarget::Record => WorkJsonTargetMeta {
            type_: "record",
            record_ref: record.work_ref.to_string(),
            line: None,
            part: None,
        },
        WorkRefTarget::Line(line) => WorkJsonTargetMeta {
            type_: "line",
            record_ref: record.work_ref.to_string(),
            line: Some(line),
            part: None,
        },
        WorkRefTarget::Part { io, index } => WorkJsonTargetMeta {
            type_: "part",
            record_ref: record.work_ref.to_string(),
            line: None,
            part: record.part_for_target(target).map(|part| WorkJsonPartMeta {
                ref_: record.work_ref.with_part(io, index).to_string(),
                io,
                kind: part.kind,
                index,
                timestamp: part.occurred_at.clone(),
                label: part.label.clone(),
            }),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sivtr_core::ai::AgentProvider;
    use sivtr_core::record::{
        WorkPart, WorkPartIo, WorkPartKind, WorkRecord, WorkRecordKind, WorkRef, WorkSessionRef,
        WorkSource, WorkTime,
    };

    #[test]
    fn builds_session_metadata_with_canonical_id() {
        let record = test_record();
        let metadata = session_meta(&record);

        assert_eq!(metadata.ref_, "codex/shortid");
        assert_eq!(metadata.channel, WorkChannel::Chat);
        assert_eq!(metadata.provider.as_deref(), Some("codex"));
        assert_eq!(metadata.display_id, "shortid");
        assert_eq!(
            metadata.canonical_id.as_deref(),
            Some("session-0123456789abcdef")
        );
    }

    #[test]
    fn builds_part_target_metadata() {
        let record = test_record();
        let metadata = target_meta(
            &record,
            WorkRefTarget::Part {
                io: WorkPartIo::Output,
                index: 1,
            },
        );

        assert_eq!(metadata.type_, "part");
        assert_eq!(metadata.record_ref, "codex/shortid/1");
        let part = metadata.part.expect("part metadata");
        assert_eq!(part.ref_, "codex/shortid/1/o/1");
        assert_eq!(part.kind, WorkPartKind::AssistantMessage);
        assert_eq!(part.label.as_deref(), Some("assistant"));
    }

    fn test_record() -> WorkRecord {
        WorkRecord {
            schema_version: 1,
            work_ref: WorkRef::agent_record(AgentProvider::Codex, "shortid", 1),
            kind: WorkRecordKind::ChatTurn,
            source: WorkSource {
                channel: WorkChannel::Chat,
                provider: Some("codex".to_string()),
            },
            session: WorkSessionRef {
                id: "shortid".to_string(),
                canonical_id: Some("session-0123456789abcdef".to_string()),
                path: Some("/tmp/session.jsonl".to_string()),
            },
            cwd: None,
            time: WorkTime::default(),
            status: None,
            title: "title".to_string(),
            parts: vec![
                WorkPart {
                    io: WorkPartIo::Input,
                    kind: WorkPartKind::UserMessage,
                    index: 1,
                    occurred_at: None,
                    label: Some("user".to_string()),
                    text: "user".to_string(),
                    ansi: None,
                },
                WorkPart {
                    io: WorkPartIo::Output,
                    kind: WorkPartKind::AssistantMessage,
                    index: 1,
                    occurred_at: Some("2026-05-24T12:00:00Z".to_string()),
                    label: Some("assistant".to_string()),
                    text: "assistant".to_string(),
                    ansi: None,
                },
            ],
        }
    }
}
