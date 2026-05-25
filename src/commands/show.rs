use anyhow::{Context, Result};
use serde::Serialize;
use sivtr_core::record::{RecordTextMode, WorkRef, WorkRefSelector};

use crate::cli::ShowArgs;
use crate::commands::records::current_work_record_index;

#[derive(Serialize)]
struct ShowJsonItem {
    #[serde(rename = "ref")]
    ref_: String,
    kind: String,
    timestamp: Option<String>,
    title: ShowJsonTitle,
    content: String,
}

#[derive(Serialize)]
struct ShowJsonTitle {
    session: String,
    dialogue: Option<String>,
}

struct ShowItem {
    ref_: WorkRef,
    content: String,
    kind: String,
    timestamp: Option<String>,
    session: String,
    dialogue: Option<String>,
}

pub fn execute(args: &ShowArgs) -> Result<()> {
    let cwd = args
        .cwd
        .clone()
        .unwrap_or(std::env::current_dir().context("Failed to resolve current directory")?);
    let selector: WorkRefSelector = args.reference.parse()?;
    let providers = selector.providers();
    let records = current_work_record_index(&providers, &cwd, None)?;
    let mut items = Vec::new();
    for record in records
        .records()
        .iter()
        .filter(|record| selector.matches_work_ref(&record.work_ref))
    {
        let line_refs = match selector.selected_lines() {
            Some(lines) => lines
                .iter()
                .map(|line| record.work_ref.with_line(*line))
                .collect::<Vec<_>>(),
            None => vec![record.work_ref.clone()],
        };

        for work_ref in line_refs {
            let content = content_for_ref(record, &work_ref)
                .with_context(|| format!("No content found for ref `{work_ref}`"))?;
            items.push(ShowItem {
                ref_: work_ref,
                content,
                kind: record.kind_label().to_string(),
                timestamp: record.time.primary_at().map(str::to_string),
                session: record.work_ref.session().to_string(),
                dialogue: Some(record.title.clone()),
            });
        }
    }

    if items.is_empty() {
        anyhow::bail!("No record found for ref selector `{}`", args.reference);
    }

    if args.json {
        let output = items.into_iter().map(show_json_item).collect::<Vec<_>>();
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

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
    Ok(())
}

fn content_for_ref(record: &sivtr_core::record::WorkRecord, work_ref: &WorkRef) -> Option<String> {
    match work_ref.line() {
        Some(line) => record
            .copy_text(RecordTextMode::Combined, false)
            .plain
            .lines()
            .nth(line - 1)
            .map(str::to_string),
        None => Some(record.copy_text(RecordTextMode::Combined, false).plain),
    }
}

fn show_json_item(item: ShowItem) -> ShowJsonItem {
    ShowJsonItem {
        ref_: item.ref_.to_string(),
        kind: item.kind,
        timestamp: item.timestamp,
        title: ShowJsonTitle {
            session: item.session,
            dialogue: item.dialogue,
        },
        content: item.content,
    }
}
