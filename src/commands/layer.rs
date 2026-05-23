use anyhow::{Context, Result};

use crate::cli::{LayerAction, LayerCommand};
use crate::commands::capture_history;
use sivtr_core::ai::{AgentProvider, AgentSessionProvider};
use sivtr_core::history::layer::LayerPath;
use sivtr_core::history::HistoryStore;

pub fn execute(cmd: LayerCommand) -> Result<()> {
    let store = HistoryStore::open_default()?;

    match cmd.action {
        LayerAction::Sync { provider, cwd } => sync_all(&store, provider.as_deref(), cwd.as_deref()),
        LayerAction::Workspaces => list_workspaces(&store),
        LayerAction::Sources { workspace } => list_sources(&store, &resolve_workspace(workspace)?),
        LayerAction::Sessions { source, workspace } => {
            list_sessions(&store, &resolve_workspace(workspace)?, &source)
        }
        LayerAction::Dialogues {
            source,
            session,
            workspace,
        } => list_dialogues(&store, &resolve_workspace(workspace)?, &source, &session),
        LayerAction::Content {
            source,
            session,
            dialogue,
            workspace,
        } => show_content(
            &store,
            &resolve_workspace(workspace)?,
            &source,
            &session,
            &dialogue,
        ),
        LayerAction::Query {
            keyword,
            scope,
            source,
            limit,
        } => query_layer(&store, &keyword, &scope, source.as_deref(), limit),
        LayerAction::Tree { workspace, depth } => {
            show_tree(&store, &resolve_workspace(workspace)?, depth)
        }
        LayerAction::Show { table, id } => show_entry(&store, &table, id),
    }
}

fn resolve_workspace(workspace: Option<String>) -> Result<String> {
    if let Some(name) = workspace {
        return Ok(name);
    }
    std::env::current_dir()
        .context("Failed to resolve current directory")?
        .to_str()
        .map(str::to_string)
        .ok_or_else(|| anyhow::anyhow!("Current directory is not valid UTF-8"))
}

// ── list commands ──

fn list_workspaces(store: &HistoryStore) -> Result<()> {
    let workspaces = store.list_workspaces()?;
    if workspaces.is_empty() {
        println!("No workspaces found. Capture some terminal output or AI sessions first.");
        return Ok(());
    }
    println!("Workspaces:");
    for ws in &workspaces {
        println!(
            "  {}  ({} sources, {} sessions, {} dialogues)  last: {}",
            ws.name,
            ws.source_count,
            ws.session_count,
            ws.dialogue_count,
            &ws.last_active[..ws.last_active.len().min(19)]
        );
    }
    Ok(())
}

fn list_sources(store: &HistoryStore, workspace: &str) -> Result<()> {
    let sources = store.list_sources(workspace)?;
    if sources.is_empty() {
        println!("No sources found for workspace `{workspace}`.");
        return Ok(());
    }
    println!("Sources in `{workspace}`:");
    for src in &sources {
        println!(
            "  {}  ({} sessions, {} dialogues)  last: {}",
            src.name,
            src.session_count,
            src.dialogue_count,
            &src.last_active[..src.last_active.len().min(19)]
        );
    }
    Ok(())
}

fn list_sessions(store: &HistoryStore, workspace: &str, source: &str) -> Result<()> {
    let sessions = store.list_sessions(workspace, source)?;
    if sessions.is_empty() {
        println!("No sessions found for `{workspace}/{source}`.");
        return Ok(());
    }
    println!("Sessions in `{workspace}/{source}`:");
    for sess in &sessions {
        println!(
            "  {}  ({} dialogues)  {} -> {}",
            sess.id,
            sess.dialogue_count,
            &sess.first_at[..sess.first_at.len().min(19)],
            &sess.last_at[..sess.last_at.len().min(19)]
        );
    }
    Ok(())
}

fn list_dialogues(
    store: &HistoryStore,
    workspace: &str,
    source: &str,
    session_id: &str,
) -> Result<()> {
    let dialogues = store.list_dialogues(workspace, source, session_id)?;
    if dialogues.is_empty() {
        println!("No dialogues found for `{workspace}/{source}/{session_id}`.");
        return Ok(());
    }
    println!("Dialogues in `{workspace}/{source}/{session_id}`:");
    for (idx, dlg) in dialogues.iter().enumerate() {
        println!(
            "  {}. {}  ({}i + {}o)  {} -> {}",
            idx + 1,
            dlg.id,
            dlg.input_count,
            dlg.output_count,
            &dlg.first_at[..dlg.first_at.len().min(19)],
            &dlg.last_at[..dlg.last_at.len().min(19)]
        );
    }
    Ok(())
}

fn show_content(
    store: &HistoryStore,
    workspace: &str,
    source: &str,
    session_id: &str,
    dialogue_id: &str,
) -> Result<()> {
    let path = LayerPath {
        workspace: workspace.to_string(),
        source: source.to_string(),
        session_id: session_id.to_string(),
        dialogue_id: dialogue_id.to_string(),
    };

    let inputs = store.list_dialogue_inputs(&path)?;
    let outputs = store.list_dialogue_outputs(&path)?;

    println!("Content for `{workspace}/{source}/{session_id}/{dialogue_id}`:");
    println!("{} input(s), {} output(s)\n", inputs.len(), outputs.len());

    for input in &inputs {
        println!(
            "── INPUT #{} [{}] {}",
            input.id,
            input.content_type,
            &input.timestamp[..input.timestamp.len().min(19)]
        );
        println!("{}", input.content);
        println!();
    }

    for output in &outputs {
        println!(
            "── OUTPUT #{} [{}] {}",
            output.id,
            output.content_type,
            &output.timestamp[..output.timestamp.len().min(19)]
        );
        println!("{}", output.content);
        println!();
    }

    Ok(())
}

// ── query ──

fn query_layer(
    store: &HistoryStore,
    keyword: &str,
    scope: &str,
    source_filter: Option<&str>,
    limit: usize,
) -> Result<()> {
    let workspace = resolve_workspace(None)?;

    let search_input = scope == "all" || scope == "input";
    let search_output = scope == "all" || scope == "output";

    if search_input {
        let results = if let Some(source) = source_filter {
            store.search_input_scoped(&workspace, source, keyword, limit)?
        } else {
            store.search_input(keyword, limit)?
        };
        if !results.is_empty() {
            println!("── Input matches ({}) ──", results.len());
            for entry in &results {
                println!(
                    "  #{id} [{typ}] {ws}/{src}/{sess}/{dlg}  {ts}",
                    id = entry.id,
                    typ = entry.content_type,
                    ws = entry.workspace,
                    src = entry.source,
                    sess = &entry.session_id[..entry.session_id.len().min(8)],
                    dlg = &entry.dialogue_id[..entry.dialogue_id.len().min(8)],
                    ts = &entry.timestamp[..entry.timestamp.len().min(19)]
                );
                let preview: String = entry.content.lines().next().unwrap_or("").chars().take(80).collect();
                println!("    {preview}");
            }
            println!();
        }
    }

    if search_output {
        let results = if let Some(source) = source_filter {
            store.search_output_scoped(&workspace, source, keyword, limit)?
        } else {
            store.search_output(keyword, limit)?
        };
        if !results.is_empty() {
            println!("── Output matches ({}) ──", results.len());
            for entry in &results {
                println!(
                    "  #{id} [{typ}] {ws}/{src}/{sess}/{dlg}  {ts}",
                    id = entry.id,
                    typ = entry.content_type,
                    ws = entry.workspace,
                    src = entry.source,
                    sess = &entry.session_id[..entry.session_id.len().min(8)],
                    dlg = &entry.dialogue_id[..entry.dialogue_id.len().min(8)],
                    ts = &entry.timestamp[..entry.timestamp.len().min(19)]
                );
                let preview: String = entry.content.lines().next().unwrap_or("").chars().take(80).collect();
                println!("    {preview}");
            }
            println!();
        }
    }

    Ok(())
}

// ── tree ──

fn show_tree(store: &HistoryStore, workspace: &str, depth: usize) -> Result<()> {
    let depth = depth.clamp(1, 5);
    println!("{workspace}/");

    if depth < 2 {
        return Ok(());
    }

    let sources = store.list_sources(workspace)?;
    for src in &sources {
        let is_last_src = src.name == sources.last().map(|s| &s.name).unwrap_or(&src.name);
        let src_prefix = if is_last_src { "└── " } else { "├── " };
        let src_cont = if is_last_src { "    " } else { "│   " };
        println!("{src_prefix}{}/", src.name);

        if depth < 3 {
            continue;
        }

        let sessions = store.list_sessions(workspace, &src.name)?;
        let session_limit = if depth < 4 { 5.min(sessions.len()) } else { sessions.len() };
        for (i, sess) in sessions.iter().take(session_limit).enumerate() {
            let is_last_sess = i == session_limit - 1;
            let sess_prefix = if is_last_sess { "└── " } else { "├── " };
            let sess_cont = if is_last_sess { "    " } else { "│   " };
            let short_id = &sess.id[..sess.id.len().min(8)];
            println!(
                "{src_cont}{sess_prefix}{short_id}  ({} dialogues)",
                sess.dialogue_count
            );

            if depth < 4 {
                continue;
            }

            let dialogues = store.list_dialogues(workspace, &src.name, &sess.id)?;
            let dlg_limit = if depth < 5 { 3.min(dialogues.len()) } else { dialogues.len() };
            for (j, dlg) in dialogues.iter().take(dlg_limit).enumerate() {
                let is_last_dlg = j == dlg_limit - 1;
                let dlg_prefix = if is_last_dlg { "└── " } else { "├── " };
                let dlg_cont = if is_last_dlg { "    " } else { "│   " };
                let short_dlg = &dlg.id[..dlg.id.len().min(8)];
                println!(
                    "{src_cont}{sess_cont}{dlg_prefix}{short_dlg}  ({}i + {}o)",
                    dlg.input_count, dlg.output_count
                );

                if depth < 5 {
                    continue;
                }

                let path = LayerPath {
                    workspace: workspace.to_string(),
                    source: src.name.clone(),
                    session_id: sess.id.clone(),
                    dialogue_id: dlg.id.clone(),
                };
                let inputs = store.list_dialogue_inputs(&path)?;
                let outputs = store.list_dialogue_outputs(&path)?;
                for input in &inputs {
                    let preview: String = input.content.lines().next().unwrap_or("").chars().take(60).collect();
                    println!(
                        "{src_cont}{sess_cont}{dlg_cont}├── i:#{} [{}] {preview}",
                        input.id, input.content_type
                    );
                }
                for output in &outputs {
                    let preview: String = output.content.lines().next().unwrap_or("").chars().take(60).collect();
                    println!(
                        "{src_cont}{sess_cont}{dlg_cont}├── o:#{} [{}] {preview}",
                        output.id, output.content_type
                    );
                }
            }

            if sessions.len() > session_limit {
                println!(
                    "{src_cont}... and {} more session(s)",
                    sessions.len() - session_limit
                );
            }
        }

        if sessions.len() > session_limit {
            println!(
                "{src_cont}... and {} more session(s)",
                sessions.len() - session_limit
            );
        }
    }

    Ok(())
}

// ── sync ──

fn sync_all(store: &HistoryStore, provider: Option<&str>, cwd_override: Option<&str>) -> Result<()> {
    let workspace = if let Some(cwd) = cwd_override {
        cwd.to_string()
    } else {
        resolve_workspace(None)?
    };

    // Sync terminal session log
    eprintln!("Syncing terminal session log...");
    match capture_history::sync_terminal_session_log(store) {
        Ok(n) => eprintln!("  terminal: {n} dialogue(s) synced"),
        Err(e) => eprintln!("  terminal: skipped ({e:#})"),
    }

    // Sync agent sessions
    let providers: Vec<AgentProvider> = if let Some(name) = provider {
        match AgentProvider::from_command_name(name) {
            Some(p) => vec![p],
            None => {
                eprintln!("Unknown provider `{name}`. Available: codex, claude, opencode, pi");
                return Ok(());
            }
        }
    } else {
        AgentProvider::all()
            .iter()
            .map(|spec| spec.provider)
            .collect()
    };

    for p in &providers {
        eprintln!("Syncing {} sessions...", p.name());
        match sync_agent_sessions(store, *p, &workspace) {
            Ok(n) => eprintln!("  {}: {n} dialogue(s) synced across all sessions", p.name()),
            Err(e) => eprintln!("  {}: error ({e:#})", p.name()),
        }
    }

    eprintln!("Sync complete. Run `sivtr layer tree` to browse.");
    Ok(())
}

fn sync_agent_sessions(
    store: &HistoryStore,
    provider: AgentProvider,
    workspace: &str,
) -> Result<usize> {
    let source = provider.session_provider();
    let source_name = provider.command_name();
    let sessions = source.list_recent_sessions(Some(std::path::Path::new(
        workspace,
    )))?;

    let mut total_dialogues = 0;
    for info in &sessions {
        let session_id = info
            .id
            .clone()
            .unwrap_or_else(|| info.path.to_string_lossy().to_string());

        // Skip if already fully synced
        if store.is_session_synced(workspace, source_name, &session_id)? {
            continue;
        }

        match source.parse_session_file(&info.path) {
            Ok(session) => {
                match store.sync_agent_session(workspace, source_name, &session_id, &session) {
                    Ok(n) => total_dialogues += n,
                    Err(e) => {
                        eprintln!("    warning: failed to sync {}: {e:#}", info.path.display())
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "    warning: failed to parse {}: {e:#}",
                    info.path.display()
                )
            }
        }
    }

    Ok(total_dialogues)
}

// ── show ──

fn show_entry(store: &HistoryStore, table: &str, id: i64) -> Result<()> {
    match table {
        "i" | "input" => match store.get_input(id)? {
            Some(entry) => {
                println!("── Input #{id} ──");
                println!("Layer:    {}/{}/{}/{}", entry.workspace, entry.source, entry.session_id, entry.dialogue_id);
                println!("Type:     {}", entry.content_type);
                println!("Timestamp: {}", entry.timestamp);
                println!("Content:");
                println!("{}", entry.content);
            }
            None => println!("Input entry #{id} not found."),
        },
        "o" | "output" => match store.get_output(id)? {
            Some(entry) => {
                println!("── Output #{id} ──");
                println!("Layer:    {}/{}/{}/{}", entry.workspace, entry.source, entry.session_id, entry.dialogue_id);
                println!("Type:     {}", entry.content_type);
                println!("Timestamp: {}", entry.timestamp);
                println!("Content:");
                println!("{}", entry.content);
            }
            None => println!("Output entry #{id} not found."),
        },
        _ => {
            anyhow::bail!("Unknown table `{table}`. Use `i` (input) or `o` (output).");
        }
    }
    Ok(())
}
