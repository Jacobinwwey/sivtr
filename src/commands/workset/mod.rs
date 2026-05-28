mod source;

pub use source::load_source;

use anyhow::{bail, Context, Result};
use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use sivtr_core::record::WorkRecord;
use std::fs;
use std::path::PathBuf;

pub const WORKSET_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkSet {
    pub schema_version: u32,
    pub created_at: String,
    pub cwd: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub records: Vec<WorkRecord>,
}

fn apply_selection(mut set: WorkSet, selection: WorkSetSelection) -> WorkSet {
    let WorkSetSelection::Indices(indices) = selection else {
        return set;
    };
    set.records = indices
        .into_iter()
        .map(|index| set.records[index - 1].clone())
        .collect();
    set
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkSetSelection {
    All,
    Indices(Vec<usize>),
}

impl WorkSet {
    pub fn new(cwd: impl Into<String>, records: Vec<WorkRecord>) -> Self {
        Self {
            schema_version: WORKSET_SCHEMA_VERSION,
            created_at: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
            cwd: cwd.into(),
            name: None,
            records,
        }
    }

    pub fn save_as(&mut self, name: &str) -> Result<()> {
        validate_name(name)?;
        self.name = Some(name.to_string());
        save_named(name, self)
    }

    pub fn save_last(&self) -> Result<()> {
        save_named("last", self)
    }
}

pub fn load_reference(reference: &str) -> Result<WorkSet> {
    let parsed = parse_reference(reference)?;
    let path = set_path(parsed.name)?;
    let content = fs::read_to_string(&path).with_context(|| {
        format!(
            "Failed to read WorkSet @{} from {}",
            parsed.name,
            path.display()
        )
    })?;
    let set: WorkSet = serde_json::from_str(&content).with_context(|| {
        format!(
            "Failed to parse WorkSet @{} from {}",
            parsed.name,
            path.display()
        )
    })?;
    validate_selection(reference, &set, &parsed.selection)?;
    Ok(apply_selection(set, parsed.selection))
}

struct ParsedWorkSetReference<'a> {
    name: &'a str,
    selection: WorkSetSelection,
}

fn parse_reference(reference: &str) -> Result<ParsedWorkSetReference<'_>> {
    let body = reference
        .strip_prefix('@')
        .ok_or_else(|| anyhow::anyhow!("WorkSet reference must start with @"))?;
    if let Some(open) = body.find('[') {
        if !body.ends_with(']') {
            bail!("Invalid WorkSet reference `{reference}`; missing closing ]");
        }
        let name = &body[..open];
        validate_name(name)?;
        let selector = &body[open + 1..body.len() - 1];
        if selector.is_empty() {
            bail!("Invalid WorkSet reference `{reference}`");
        }
        let selection = parse_selector(selector, reference)?;
        Ok(ParsedWorkSetReference { name, selection })
    } else {
        validate_name(body)?;
        Ok(ParsedWorkSetReference {
            name: body,
            selection: WorkSetSelection::All,
        })
    }
}

fn parse_selector(selector: &str, reference: &str) -> Result<WorkSetSelection> {
    let mut indices = Vec::new();
    for segment in selector.split(',') {
        if segment.is_empty() {
            bail!("Invalid WorkSet reference `{reference}`; empty selector segment");
        }
        if let Some((start, end)) = segment.split_once("..") {
            let start = parse_index(start, reference)?;
            let end = parse_index(end, reference)?;
            if start > end {
                bail!("Invalid WorkSet reference `{reference}`; range start must be <= end");
            }
            indices.extend(start..=end);
        } else {
            indices.push(parse_index(segment, reference)?);
        }
    }
    Ok(WorkSetSelection::Indices(indices))
}

fn parse_index(value: &str, reference: &str) -> Result<usize> {
    let index = value.parse::<usize>().with_context(|| {
        format!("Invalid WorkSet reference `{reference}`; index must be a positive integer")
    })?;
    if index == 0 {
        bail!("Invalid WorkSet reference `{reference}`; index must be 1-based");
    }
    Ok(index)
}

fn validate_selection(reference: &str, set: &WorkSet, selection: &WorkSetSelection) -> Result<()> {
    match selection {
        WorkSetSelection::All => Ok(()),
        WorkSetSelection::Indices(indices) => {
            for index in indices {
                if *index > set.records.len() {
                    bail!(
                        "Invalid WorkSet reference `{reference}`; index {index} exceeds WorkSet length {}",
                        set.records.len()
                    );
                }
            }
            Ok(())
        }
    }
}

fn save_named(name: &str, set: &WorkSet) -> Result<()> {
    let path = set_path(name)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create WorkSet directory {}", parent.display()))?;
    }
    fs::write(&path, serde_json::to_string_pretty(set)?)
        .with_context(|| format!("Failed to write WorkSet @{} to {}", name, path.display()))
}

fn set_path(name: &str) -> Result<PathBuf> {
    Ok(sets_dir()?.join(format!("{name}.json")))
}

fn sets_dir() -> Result<PathBuf> {
    let state_dir = dirs::state_dir()
        .or_else(dirs::data_local_dir)
        .or_else(dirs::config_dir)
        .ok_or_else(|| anyhow::anyhow!("Cannot determine state directory"))?;
    Ok(state_dir.join("sivtr").join("sets"))
}

fn validate_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("WorkSet name cannot be empty");
    }
    if !name
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
    {
        bail!("Invalid WorkSet name `{name}`; use letters, numbers, '-' or '_'");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sivtr_core::record::{
        WorkChannel, WorkPart, WorkPartIo, WorkPartKind, WorkRecord, WorkRecordKind,
        WorkSessionRef, WorkSource, WorkTime,
    };

    fn record(index: usize) -> WorkRecord {
        WorkRecord {
            schema_version: sivtr_core::record::RECORD_SCHEMA_VERSION,
            work_ref: format!("terminal/session_1/{index}")
                .parse()
                .expect("valid work ref"),
            kind: WorkRecordKind::TerminalCommand,
            source: WorkSource {
                channel: WorkChannel::Terminal,
                provider: None,
            },
            session: WorkSessionRef {
                id: "session_1".to_string(),
                canonical_id: Some("session_1".to_string()),
                path: None,
            },
            cwd: None,
            time: WorkTime::default(),
            status: None,
            title: format!("record {index}"),
            parts: vec![WorkPart {
                io: WorkPartIo::Output,
                kind: WorkPartKind::Text,
                index: 1,
                occurred_at: None,
                label: None,
                text: format!("record {index}"),
                ansi: None,
            }],
        }
    }

    #[test]
    fn parses_discrete_and_range_selectors_in_order() {
        let selection = parse_selector("1,3..5,2", "@hits[1,3..5,2]").expect("selector parses");
        assert_eq!(selection, WorkSetSelection::Indices(vec![1, 3, 4, 5, 2]));
    }

    #[test]
    fn selected_keeps_discrete_selector_order() {
        let set = WorkSet::new(".", (1..=5).map(record).collect());
        let selected = apply_selection(set, WorkSetSelection::Indices(vec![3, 1, 5]));

        let refs = selected
            .records
            .iter()
            .map(|record| record.work_ref.to_string())
            .collect::<Vec<_>>();
        assert_eq!(
            refs,
            vec![
                "terminal/session_1/3",
                "terminal/session_1/1",
                "terminal/session_1/5"
            ]
        );
    }

    #[test]
    fn rejects_empty_discrete_selector_segment() {
        let error = parse_selector("1,,3", "@hits[1,,3]").expect_err("selector rejected");
        assert!(error.to_string().contains("empty selector segment"));
    }
}
