use std::io::{self, Read};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use sivtr_core::ai::AgentProvider;
use sivtr_core::record::{WorkRecord, WorkRef, WorkRefSelector};

use crate::commands::records::current_work_record_index;

use super::WorkSet;

#[derive(Debug, Clone)]
pub enum WorkSetSource {
    Reference(WorkSet),
    Records {
        cwd: PathBuf,
        records: Vec<WorkRecord>,
    },
}

impl WorkSetSource {
    pub fn cwd(&self) -> PathBuf {
        match self {
            Self::Reference(set) => PathBuf::from(&set.cwd),
            Self::Records { cwd, .. } => cwd.clone(),
        }
    }

    pub fn records(self) -> Vec<WorkRecord> {
        match self {
            Self::Reference(set) => set.records,
            Self::Records { records, .. } => records,
        }
    }

    pub fn into_workset(self) -> WorkSet {
        match self {
            Self::Reference(set) => set,
            Self::Records { cwd, records } => WorkSet::new(cwd.display().to_string(), records),
        }
    }
}

pub fn load_source(source: &str, cwd: Option<&Path>) -> Result<WorkSetSource> {
    if source == "@" {
        return Ok(WorkSetSource::Reference(read_stdin()?));
    }
    if source.starts_with('@') {
        return Ok(WorkSetSource::Reference(super::load_reference(source)?));
    }

    let cwd = cwd
        .map(Path::to_path_buf)
        .unwrap_or(std::env::current_dir().context("Failed to resolve current directory")?);
    if let Ok(work_ref) = source.parse::<WorkRef>() {
        return resolve_ref_source(source, &cwd, &work_ref);
    }

    resolve_selector_source(source, &cwd)
}

fn read_stdin() -> Result<WorkSet> {
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .context("Failed to read WorkSet from stdin")?;
    serde_json::from_str(&input).context("Failed to parse WorkSet from stdin")
}

fn resolve_ref_source(source: &str, cwd: &Path, work_ref: &WorkRef) -> Result<WorkSetSource> {
    let work_ref = work_ref.record_ref();
    let providers = work_ref
        .provider()
        .map(|provider| vec![provider])
        .unwrap_or_else(all_agent_providers);
    let index = current_work_record_index(&providers, cwd, None)?;
    let record = index
        .resolve(&work_ref)
        .with_context(|| format!("No record found for ref `{source}`"))?;
    Ok(WorkSetSource::Records {
        cwd: cwd.to_path_buf(),
        records: vec![record.clone()],
    })
}

fn resolve_selector_source(source: &str, cwd: &Path) -> Result<WorkSetSource> {
    let selector: WorkRefSelector = source.parse()?;
    let providers = selector.providers();
    let index = current_work_record_index(&providers, cwd, None)?;
    let records = index
        .records()
        .iter()
        .filter(|record| selector.matches_work_ref(&record.work_ref))
        .cloned()
        .collect::<Vec<_>>();

    if records.is_empty() {
        bail!("No record found for ref selector `{source}`");
    }

    Ok(WorkSetSource::Records {
        cwd: cwd.to_path_buf(),
        records,
    })
}

fn all_agent_providers() -> Vec<AgentProvider> {
    AgentProvider::all()
        .iter()
        .map(|spec| spec.provider)
        .collect()
}
