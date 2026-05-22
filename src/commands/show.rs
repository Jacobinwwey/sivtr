use anyhow::{bail, Context, Result};
use serde::Serialize;

use crate::cli::ShowArgs;
use crate::commands::copy::current_workspace_sessions;
use crate::tui::workspace::WorkspaceSource;
use sivtr_core::ai::{AgentProvider, AgentSelection};

#[derive(Debug)]
struct ParsedRef {
    source: String,
    session: String,
    dialogue: Option<usize>,
    line: Option<usize>,
}

#[derive(Serialize)]
struct ShowJsonOutput {
    #[serde(rename = "ref")]
    ref_: String,
    kind: String,
    source: String,
    session_id: String,
    session: String,
    dialogue_index: Option<usize>,
    dialogue: Option<String>,
    line: Option<usize>,
    timestamp: Option<String>,
    content: String,
}

pub fn execute(args: &ShowArgs) -> Result<()> {
    let cwd = args
        .cwd
        .clone()
        .unwrap_or(std::env::current_dir().context("Failed to resolve current directory")?);
    let parsed = parse_ref(&args.reference)?;
    let providers = provider_for_source(&parsed.source)
        .map(|provider| vec![provider])
        .unwrap_or_else(|| {
            AgentProvider::all()
                .iter()
                .map(|spec| spec.provider)
                .collect()
        });
    let sessions = current_workspace_sessions(&providers, &cwd, AgentSelection::LastTurn)?;
    let session = sessions
        .iter()
        .find(|session| {
            source_name(session.source) == parsed.source && session.ref_id == parsed.session
        })
        .with_context(|| format!("No workspace session found for ref `{}`", args.reference))?;

    let (dialogue_index, dialogue_title, timestamp, content) = match parsed.dialogue {
        Some(dialogue) => {
            if dialogue == 0 {
                bail!("Dialogue index in ref must be 1-based");
            }
            let idx = dialogue - 1;
            let unit = session
                .units
                .get(idx)
                .with_context(|| format!("No dialogue {dialogue} in ref `{}`", args.reference))?;
            let title = session.dialogue_titles.get(idx).cloned();
            let timestamp = session.unit_timestamps.get(idx).cloned().flatten();
            let content = match parsed.line {
                Some(line) => {
                    if line == 0 {
                        bail!("Line index in ref must be 1-based");
                    }
                    unit.plain
                        .lines()
                        .nth(line - 1)
                        .with_context(|| format!("No line {line} in ref `{}`", args.reference))?
                        .to_string()
                }
                None => unit.plain.clone(),
            };
            (Some(dialogue), title, timestamp, content)
        }
        None => {
            let content = session
                .units
                .iter()
                .map(|unit| unit.plain.as_str())
                .collect::<Vec<_>>()
                .join("\n\n");
            (None, None, None, content)
        }
    };

    if args.json {
        let output = ShowJsonOutput {
            ref_: args.reference.clone(),
            kind: kind_name(session.source).to_string(),
            source: source_name(session.source).to_string(),
            session_id: session.ref_id.clone(),
            session: session.title.clone(),
            dialogue_index,
            dialogue: dialogue_title,
            line: parsed.line,
            timestamp,
            content,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    print!("{content}");
    if !content.ends_with('\n') {
        println!();
    }
    Ok(())
}

fn parse_ref(reference: &str) -> Result<ParsedRef> {
    let parts = reference
        .split('/')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if !(2..=4).contains(&parts.len()) {
        bail!("Invalid ref `{reference}`; expected source/session[/dialogue[/line]]");
    }

    let dialogue = parts
        .get(2)
        .map(|value| parse_one_based_index(value, "dialogue"))
        .transpose()?;
    let line = parts
        .get(3)
        .map(|value| parse_one_based_index(value, "line"))
        .transpose()?;

    Ok(ParsedRef {
        source: parts[0].to_string(),
        session: parts[1].to_string(),
        dialogue,
        line,
    })
}

fn parse_one_based_index(value: &str, name: &str) -> Result<usize> {
    let index = value
        .parse::<usize>()
        .with_context(|| format!("Invalid {name} index `{value}`"))?;
    if index == 0 {
        bail!("{name} index must be 1-based");
    }
    Ok(index)
}

fn provider_for_source(source: &str) -> Option<AgentProvider> {
    AgentProvider::from_command_name(source)
}

fn source_name(source: WorkspaceSource) -> &'static str {
    match source {
        WorkspaceSource::Terminal => "terminal",
        WorkspaceSource::Agent(provider) => provider.command_name(),
    }
}

fn kind_name(source: WorkspaceSource) -> &'static str {
    match source {
        WorkspaceSource::Terminal => "shell",
        WorkspaceSource::Agent(_) => "ai",
    }
}
