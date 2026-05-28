use std::io::IsTerminal;
use std::str::FromStr;

use anyhow::{Context, Result};
use sivtr_core::record::{WorkRecord, WorkRef, WorkRefTarget};

use crate::cli::ShowArgs;
use crate::commands::workset;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkSetOutputFormat {
    Full,
    Compact,
    Timeline,
    Md,
    Refs,
    WorkSet,
}

impl FromStr for WorkSetOutputFormat {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "full" => Ok(Self::Full),
            "compact" => Ok(Self::Compact),
            "timeline" => Ok(Self::Timeline),
            "md" | "markdown" => Ok(Self::Md),
            "refs" => Ok(Self::Refs),
            "workset" | "json" => Ok(Self::WorkSet),
            _ => Err(format!(
                "unknown show output format `{value}`; expected full, timeline, compact, md, refs, or workset"
            )),
        }
    }
}

impl std::fmt::Display for WorkSetOutputFormat {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Full => "full",
            Self::Compact => "compact",
            Self::Timeline => "timeline",
            Self::Md => "md",
            Self::Refs => "refs",
            Self::WorkSet => "workset",
        })
    }
}

struct ShowItem {
    ref_: WorkRef,
    content: String,
}

pub fn default_output_format() -> WorkSetOutputFormat {
    if std::io::stdout().is_terminal() {
        WorkSetOutputFormat::Full
    } else {
        WorkSetOutputFormat::WorkSet
    }
}

pub fn resolve_output_format(
    format: Option<WorkSetOutputFormat>,
    full: bool,
    refs: bool,
    json: bool,
) -> WorkSetOutputFormat {
    if full {
        WorkSetOutputFormat::Full
    } else if refs {
        WorkSetOutputFormat::Refs
    } else if json {
        WorkSetOutputFormat::WorkSet
    } else {
        format.unwrap_or_else(default_output_format)
    }
}

pub fn execute(args: &ShowArgs) -> Result<()> {
    let set = resolve_target(args)?;
    print_workset(
        &set,
        resolve_output_format(args.format, args.full, args.refs, args.json),
    )
}

fn resolve_target(args: &ShowArgs) -> Result<workset::WorkSet> {
    Ok(workset::load_source(&args.source, args.cwd.as_deref())?.into_workset())
}

pub fn print_workset(set: &workset::WorkSet, format: WorkSetOutputFormat) -> Result<()> {
    match format {
        WorkSetOutputFormat::Full => print_full(set)?,
        WorkSetOutputFormat::WorkSet => {
            println!("{}", serde_json::to_string_pretty(set)?);
        }
        WorkSetOutputFormat::Compact => print_compact(set),
        WorkSetOutputFormat::Timeline => print_timeline(set),
        WorkSetOutputFormat::Md => print_markdown(set),
        WorkSetOutputFormat::Refs => print_refs(set),
    }
    Ok(())
}

fn print_full(set: &workset::WorkSet) -> Result<()> {
    let items = set
        .records
        .iter()
        .map(|record| {
            let work_ref = record.work_ref.record_ref();
            let content = record
                .content_for_target(WorkRefTarget::Record)
                .with_context(|| format!("No content found for ref `{work_ref}`"))?;
            Ok(ShowItem {
                ref_: work_ref,
                content,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    print_show_items(items);
    Ok(())
}

fn print_show_items(items: Vec<ShowItem>) {
    let multi = items.len() > 1;
    for item in items {
        if multi {
            println!("--- {} ---", item.ref_);
        }
        print!("{}", item.content);
        if !item.content.ends_with('\n') {
            println!();
        }
    }
}

fn print_compact(set: &workset::WorkSet) {
    for record in &set.records {
        println!(
            "{}  {:<8}  {}",
            short_time(record),
            source_label(record),
            record.title
        );
    }
}

fn print_timeline(set: &workset::WorkSet) {
    let mut previous_timestamp: Option<chrono::DateTime<chrono::Utc>> = None;
    for record in &set.records {
        let timestamp = record
            .time
            .primary_at()
            .and_then(sivtr_core::time::parse_timestamp);
        if let (Some(previous), Some(current)) = (previous_timestamp, timestamp) {
            let gap_minutes = (current - previous).num_minutes();
            if gap_minutes >= 15 {
                println!("          -- gap {gap_minutes}m --");
            }
        }
        if timestamp.is_some() {
            previous_timestamp = timestamp;
        }

        println!(
            "{}  {:<8}  {:<28}  {}",
            short_time(record),
            source_label(record),
            record.work_ref.record_ref(),
            record.title
        );
    }
}

fn print_markdown(set: &workset::WorkSet) {
    for record in &set.records {
        println!(
            "- **{}** `{}` {}",
            short_time(record),
            record.work_ref.record_ref(),
            escape_markdown_title(&record.title)
        );
    }
}

fn print_refs(set: &workset::WorkSet) {
    for record in &set.records {
        println!("{}", record.work_ref.record_ref());
    }
}

fn short_time(record: &WorkRecord) -> String {
    record
        .time
        .primary_at()
        .and_then(sivtr_core::time::parse_timestamp)
        .map(|timestamp| {
            timestamp
                .with_timezone(&chrono::Local)
                .format("%H:%M:%S")
                .to_string()
        })
        .unwrap_or_else(|| "--:--:--".to_string())
}

fn source_label(record: &WorkRecord) -> &'static str {
    match record.kind {
        sivtr_core::record::WorkRecordKind::TerminalCommand => "terminal",
        sivtr_core::record::WorkRecordKind::ChatTurn => record
            .work_ref
            .provider()
            .map(|provider| provider.command_name())
            .unwrap_or("agent"),
    }
}

fn escape_markdown_title(title: &str) -> String {
    title.replace('[', "\\[").replace(']', "\\]")
}
