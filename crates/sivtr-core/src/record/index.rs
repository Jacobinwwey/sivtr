use super::model::{WorkRecord, WorkRecordKind};
use super::refs::WorkRef;

#[derive(Debug, Clone)]
pub struct WorkRecordIndex {
    records: Vec<WorkRecord>,
}

impl WorkRecordIndex {
    pub fn new(records: Vec<WorkRecord>) -> Self {
        Self { records }
    }

    pub fn records(&self) -> &[WorkRecord] {
        &self.records
    }

    pub fn resolve(&self, reference: &WorkRef) -> Option<&WorkRecord> {
        let record_ref = reference.record_ref();
        self.records
            .iter()
            .find(|record| record.work_ref == record_ref)
    }
}

impl WorkRecord {
    pub fn kind_label(&self) -> &'static str {
        match self.kind {
            WorkRecordKind::TerminalCommand => "shell",
            WorkRecordKind::ChatTurn => "ai",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::AgentProvider;
    use crate::record::model::{
        WorkOutcome, WorkPayload, WorkRecordKind, WorkStatus, WorkText, WorkTime,
    };

    #[test]
    fn resolves_records_by_typed_ref() {
        let records = vec![test_record("pi/abcdef12/2", "abcdef12", 2, "hello\nneedle")];
        let index = WorkRecordIndex::new(records);
        let reference = WorkRef::agent_record(AgentProvider::Pi, "abcdef12", 2);

        assert_eq!(index.resolve(&reference).unwrap().title, "title");
        assert!(index
            .resolve(&WorkRef::agent_record(AgentProvider::Pi, "abcdef12", 3))
            .is_none());
    }

    fn test_record(
        ref_id: &str,
        session_id: &str,
        turn_index: usize,
        combined: &str,
    ) -> WorkRecord {
        WorkRecord {
            schema_version: 1,
            work_ref: WorkRef::agent_record(AgentProvider::Pi, session_id, turn_index),
            kind: WorkRecordKind::ChatTurn,
            session_path: None,
            cwd: None,
            time: WorkTime::default(),
            status: WorkStatus {
                outcome: WorkOutcome::Unknown,
                exit_code: None,
            },
            title: "title".to_string(),
            text: WorkText {
                input: None,
                output: Some(combined.to_string()),
            },
            payload: WorkPayload::ChatTurn {
                user: String::new(),
                assistant: combined.to_string(),
                messages: Vec::new(),
            },
        }
    }
}
