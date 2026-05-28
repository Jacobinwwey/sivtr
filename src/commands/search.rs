use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use chrono::Utc;
use regex::Regex;
use sivtr_core::ai::{AgentProvider, AgentSessionProvider};
use sivtr_core::record::{WorkOutcome, WorkRecord, WorkRecordKind, WorkRefSelector};

use crate::cli::{SearchArgs, SearchFieldArg, SearchSortArg, SearchStatusArg};
use crate::commands::show;
use crate::commands::time_filter::build_time_range;
use crate::commands::workset::{self, WorkSet};

struct SearchResultGroup<'a> {
    record: &'a WorkRecord,
}

struct SearchMatch<'a> {
    record: &'a WorkRecord,
    ref_: String,
    snippet: String,
}

pub fn execute(args: &SearchArgs) -> Result<()> {
    let source = workset::load_source(&args.source, args.cwd.as_deref())?;
    let cwd = source.cwd();
    let selector = if args.source.starts_with('@') {
        None
    } else {
        Some(args.source.parse::<WorkRefSelector>()?)
    };
    let providers = selector
        .as_ref()
        .map(WorkRefSelector::providers)
        .unwrap_or_default();
    let now = Utc::now();
    let (time_range, _) = build_time_range(
        args.since.as_deref(),
        args.until.as_deref(),
        args.last.as_deref(),
        now,
    )?;
    let records = source.records();
    let excluded_sessions = if args.exclude_current {
        current_agent_session_paths(&providers, &cwd)?
    } else {
        HashSet::new()
    };
    let regex = args
        .match_
        .as_deref()
        .map(|query| Regex::new(&format!("(?i){query}")))
        .transpose()?;
    let exclude_regex = args
        .exclude
        .as_deref()
        .map(|query| Regex::new(&format!("(?i){query}")))
        .transpose()?;
    let min_duration_ms = parse_duration_ms_filter(args.min_duration.as_deref(), "--min-duration")?;
    let max_duration_ms = parse_duration_ms_filter(args.max_duration.as_deref(), "--max-duration")?;
    if let (Some(min), Some(max)) = (min_duration_ms, max_duration_ms) {
        if min > max {
            bail!("--min-duration must be less than or equal to --max-duration");
        }
    }
    let mut matches = records
        .iter()
        .filter(|record| {
            selector
                .as_ref()
                .is_none_or(|selector| selector_matches_record(selector, record))
                && !excluded_session_matches(record, &excluded_sessions)
                && status_matches(
                    args.status,
                    record
                        .status
                        .as_ref()
                        .map(|status| status.outcome)
                        .unwrap_or(WorkOutcome::Unknown),
                )
                && exit_code_matches(
                    args.exit_code,
                    record.status.as_ref().and_then(|status| status.exit_code),
                )
                && duration_matches(min_duration_ms, max_duration_ms, record.time.duration_ms)
                && time_range
                    .as_ref()
                    .is_none_or(|range| range.contains_record_time(record.time.primary_at()))
        })
        .flat_map(|record| matching_refs(record, selector.as_ref(), args.in_field, regex.as_ref()))
        .filter(|matched| !match_excluded(matched, args.in_field, exclude_regex.as_ref()))
        .collect::<Vec<_>>();
    sort_results(&mut matches, SearchSortArg::Newest);
    let mut results = group_results(matches);
    if let Some(latest) = args.latest {
        results.truncate(latest);
    }
    sort_group_results(&mut results, args.sort);
    if let Some(limit) = args.limit.or_else(|| args.latest.is_none().then_some(20)) {
        results.truncate(limit);
    }

    let mut workset = WorkSet::new(
        cwd.display().to_string(),
        results
            .into_iter()
            .map(|result| result.record.clone())
            .collect(),
    );
    workset.save_last()?;
    if let Some(name) = args.save.as_deref() {
        workset.save_as(name)?;
    }
    show::print_workset(
        &workset,
        show::resolve_output_format(args.format, false, args.refs, args.json),
    )?;

    Ok(())
}

fn selector_matches_record(selector: &WorkRefSelector, record: &WorkRecord) -> bool {
    match selector {
        WorkRefSelector::Terminal { .. } if record.kind != WorkRecordKind::TerminalCommand => {
            return false;
        }
        WorkRefSelector::Agent { .. } if record.kind != WorkRecordKind::ChatTurn => {
            return false;
        }
        _ => {}
    }

    selector.matches_work_ref(&record.work_ref)
}

fn status_matches(status: Option<SearchStatusArg>, outcome: WorkOutcome) -> bool {
    match status {
        Some(SearchStatusArg::Success) => outcome == WorkOutcome::Success,
        Some(SearchStatusArg::Failure) => outcome == WorkOutcome::Failure,
        Some(SearchStatusArg::Unknown) => outcome == WorkOutcome::Unknown,
        None => true,
    }
}

fn exit_code_matches(expected: Option<i32>, actual: Option<i32>) -> bool {
    expected.is_none_or(|expected| actual == Some(expected))
}

fn duration_matches(min: Option<u64>, max: Option<u64>, actual: Option<u64>) -> bool {
    if min.is_none() && max.is_none() {
        return true;
    }

    let Some(actual) = actual else {
        return false;
    };

    min.is_none_or(|min| actual >= min) && max.is_none_or(|max| actual <= max)
}

fn parse_duration_ms_filter(value: Option<&str>, label: &str) -> Result<Option<u64>> {
    value
        .map(|value| parse_duration_ms(value).with_context(|| format!("Invalid {label}: {value}")))
        .transpose()
}

fn parse_duration_ms(value: &str) -> Result<u64> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        bail!("duration is empty");
    }

    let number_end = trimmed
        .char_indices()
        .take_while(|(_, ch)| ch.is_ascii_digit())
        .map(|(idx, ch)| idx + ch.len_utf8())
        .last()
        .ok_or_else(|| anyhow::anyhow!("duration must start with a number"))?;
    let amount = trimmed[..number_end]
        .parse::<u64>()
        .context("duration amount must be an unsigned integer")?;
    let unit = trimmed[number_end..].trim().to_ascii_lowercase();
    let multiplier = match unit.as_str() {
        "" | "ms" | "msec" | "msecs" | "millisecond" | "milliseconds" => 1,
        "s" | "sec" | "secs" | "second" | "seconds" => 1_000,
        "m" | "min" | "mins" | "minute" | "minutes" => 60_000,
        "h" | "hr" | "hrs" | "hour" | "hours" => 3_600_000,
        _ => bail!("unsupported duration unit `{unit}`"),
    };
    amount
        .checked_mul(multiplier)
        .ok_or_else(|| anyhow::anyhow!("duration is too large"))
}

fn field_matches(record: &WorkRecord, field: SearchFieldArg, regex: &Regex) -> bool {
    match field {
        SearchFieldArg::Content => regex.is_match(&combined_text(record)),
        SearchFieldArg::Title => regex.is_match(&record.title),
        SearchFieldArg::Session => regex.is_match(record.work_ref.session()),
        SearchFieldArg::Input => record
            .input_text()
            .is_some_and(|text| regex.is_match(&text)),
        SearchFieldArg::Output => record
            .output_text()
            .is_some_and(|text| regex.is_match(&text)),
        SearchFieldArg::Command => record
            .input_text()
            .is_some_and(|text| regex.is_match(&text)),
        SearchFieldArg::All => {
            regex.is_match(&combined_text(record))
                || regex.is_match(&record.title)
                || regex.is_match(record.work_ref.session())
        }
    }
}

fn match_excluded(matched: &SearchMatch<'_>, field: SearchFieldArg, regex: Option<&Regex>) -> bool {
    let Some(regex) = regex else {
        return false;
    };

    if regex.is_match(&matched.snippet) {
        return true;
    }

    if line_search_field(field) {
        return matched
            .ref_
            .rsplit_once('/')
            .and_then(|(_, line)| line.parse::<usize>().ok())
            .and_then(|line| {
                combined_text(matched.record)
                    .lines()
                    .nth(line - 1)
                    .map(str::to_string)
            })
            .is_some_and(|line| regex.is_match(&line));
    }

    field_matches(matched.record, field, regex)
}

fn matching_refs<'a>(
    record: &'a WorkRecord,
    selector: Option<&WorkRefSelector>,
    field: SearchFieldArg,
    regex: Option<&Regex>,
) -> Vec<SearchMatch<'a>> {
    let text = combined_text(record);
    let target_lines = selector.and_then(WorkRefSelector::selected_lines);
    if let Some(lines) = target_lines {
        let has_selected_line = lines
            .iter()
            .any(|line| text.lines().nth(line - 1).is_some());
        if !has_selected_line {
            return Vec::new();
        }
    }

    let Some(regex) = regex else {
        return match target_lines {
            Some(lines) => lines
                .iter()
                .filter_map(|line| {
                    text.lines().nth(line - 1).map(|line_text| SearchMatch {
                        record,
                        ref_: record.work_ref.with_line(*line).to_string(),
                        snippet: snippet(line_text),
                    })
                })
                .collect(),
            None => vec![SearchMatch {
                record,
                ref_: record.work_ref.to_string(),
                snippet: record_snippet(record, field),
            }],
        };
    };

    if let Some(lines) = target_lines {
        return lines
            .iter()
            .filter_map(|line| {
                let line_text = text.lines().nth(line - 1).unwrap_or_default();
                line_matches_field(record, field, *line, line_text, regex).then(|| SearchMatch {
                    record,
                    ref_: record.work_ref.with_line(*line).to_string(),
                    snippet: snippet(line_text),
                })
            })
            .collect();
    }

    if line_search_field(field) {
        let matches = text
            .lines()
            .enumerate()
            .filter(|(_, line)| regex.is_match(line))
            .map(|(idx, line)| SearchMatch {
                record,
                ref_: record.work_ref.with_line(idx + 1).to_string(),
                snippet: snippet(line),
            })
            .collect::<Vec<_>>();
        if !matches.is_empty() {
            return matches;
        }
    }

    if field_matches(record, field, regex) {
        return vec![SearchMatch {
            record,
            ref_: record.work_ref.to_string(),
            snippet: record_snippet(record, field),
        }];
    }

    Vec::new()
}

fn line_search_field(field: SearchFieldArg) -> bool {
    matches!(
        field,
        SearchFieldArg::Content | SearchFieldArg::Output | SearchFieldArg::All
    )
}

fn line_matches_field(
    record: &WorkRecord,
    field: SearchFieldArg,
    line_index: usize,
    line: &str,
    regex: &Regex,
) -> bool {
    match field {
        SearchFieldArg::Content | SearchFieldArg::Output | SearchFieldArg::All => {
            regex.is_match(line)
        }
        _ => {
            field_matches(record, field, regex)
                && combined_text(record).lines().nth(line_index - 1).is_some()
        }
    }
}

fn sort_results(results: &mut [SearchMatch<'_>], sort: SearchSortArg) {
    match sort {
        SearchSortArg::Newest => results.sort_by(|a, b| {
            b.record
                .time
                .primary_at()
                .cmp(&a.record.time.primary_at())
                .then_with(|| a.ref_.cmp(&b.ref_))
        }),
        SearchSortArg::Oldest => results.sort_by(|a, b| {
            a.record
                .time
                .primary_at()
                .cmp(&b.record.time.primary_at())
                .then_with(|| a.ref_.cmp(&b.ref_))
        }),
        SearchSortArg::Duration => results.sort_by(|a, b| {
            b.record
                .time
                .duration_ms
                .cmp(&a.record.time.duration_ms)
                .then_with(|| b.record.time.primary_at().cmp(&a.record.time.primary_at()))
                .then_with(|| a.ref_.cmp(&b.ref_))
        }),
        SearchSortArg::DurationAsc => results.sort_by(|a, b| {
            a.record
                .time
                .duration_ms
                .cmp(&b.record.time.duration_ms)
                .then_with(|| b.record.time.primary_at().cmp(&a.record.time.primary_at()))
                .then_with(|| a.ref_.cmp(&b.ref_))
        }),
        SearchSortArg::ExitCode => results.sort_by(|a, b| {
            b.record
                .status
                .as_ref()
                .and_then(|status| status.exit_code)
                .cmp(&a.record.status.as_ref().and_then(|status| status.exit_code))
                .then_with(|| b.record.time.primary_at().cmp(&a.record.time.primary_at()))
                .then_with(|| a.ref_.cmp(&b.ref_))
        }),
        SearchSortArg::ExitCodeAsc => results.sort_by(|a, b| {
            a.record
                .status
                .as_ref()
                .and_then(|status| status.exit_code)
                .cmp(&b.record.status.as_ref().and_then(|status| status.exit_code))
                .then_with(|| b.record.time.primary_at().cmp(&a.record.time.primary_at()))
                .then_with(|| a.ref_.cmp(&b.ref_))
        }),
    }
}

fn combined_text(record: &WorkRecord) -> String {
    record.combined_text()
}

fn group_results(matches: Vec<SearchMatch<'_>>) -> Vec<SearchResultGroup<'_>> {
    let mut results: Vec<SearchResultGroup<'_>> = Vec::new();

    for matched in matches {
        let record_ref = matched.record.work_ref.record_ref();
        if !results
            .iter()
            .any(|group| group.record.work_ref.record_ref() == record_ref)
        {
            results.push(SearchResultGroup {
                record: matched.record,
            });
        }
    }

    results
}

fn record_ref_string(result: &SearchResultGroup<'_>) -> String {
    result.record.work_ref.record_ref().to_string()
}

fn sort_group_results(results: &mut [SearchResultGroup<'_>], sort: SearchSortArg) {
    match sort {
        SearchSortArg::Newest => results.sort_by(|a, b| {
            b.record
                .time
                .primary_at()
                .cmp(&a.record.time.primary_at())
                .then_with(|| record_ref_string(a).cmp(&record_ref_string(b)))
        }),
        SearchSortArg::Oldest => results.sort_by(|a, b| {
            a.record
                .time
                .primary_at()
                .cmp(&b.record.time.primary_at())
                .then_with(|| record_ref_string(a).cmp(&record_ref_string(b)))
        }),
        SearchSortArg::Duration => results.sort_by(|a, b| {
            b.record
                .time
                .duration_ms
                .cmp(&a.record.time.duration_ms)
                .then_with(|| b.record.time.primary_at().cmp(&a.record.time.primary_at()))
                .then_with(|| record_ref_string(a).cmp(&record_ref_string(b)))
        }),
        SearchSortArg::DurationAsc => results.sort_by(|a, b| {
            a.record
                .time
                .duration_ms
                .cmp(&b.record.time.duration_ms)
                .then_with(|| b.record.time.primary_at().cmp(&a.record.time.primary_at()))
                .then_with(|| record_ref_string(a).cmp(&record_ref_string(b)))
        }),
        SearchSortArg::ExitCode => results.sort_by(|a, b| {
            b.record
                .status
                .as_ref()
                .and_then(|status| status.exit_code)
                .cmp(&a.record.status.as_ref().and_then(|status| status.exit_code))
                .then_with(|| b.record.time.primary_at().cmp(&a.record.time.primary_at()))
                .then_with(|| record_ref_string(a).cmp(&record_ref_string(b)))
        }),
        SearchSortArg::ExitCodeAsc => results.sort_by(|a, b| {
            a.record
                .status
                .as_ref()
                .and_then(|status| status.exit_code)
                .cmp(&b.record.status.as_ref().and_then(|status| status.exit_code))
                .then_with(|| b.record.time.primary_at().cmp(&a.record.time.primary_at()))
                .then_with(|| record_ref_string(a).cmp(&record_ref_string(b)))
        }),
    }
}

fn current_agent_session_paths(
    providers: &[AgentProvider],
    cwd: &Path,
) -> Result<HashSet<PathBuf>> {
    let mut paths = HashSet::new();

    for provider in providers {
        let source = provider.session_provider();
        if let Some(path) = current_agent_session_path(source.as_ref(), *provider, cwd)? {
            paths.insert(comparable_path(&path));
        }
    }

    Ok(paths)
}

fn current_agent_session_path(
    source: &dyn AgentSessionProvider,
    provider: AgentProvider,
    cwd: &Path,
) -> Result<Option<PathBuf>> {
    if let Some(path) = current_agent_transcript_path(provider) {
        return Ok(Some(path));
    }

    if let Some(session_id) = current_agent_session_id(provider) {
        if let Some(path) = source.find_session_by_id(&session_id)? {
            return Ok(Some(path));
        }
    }

    source.find_current_session(cwd)
}

fn current_agent_transcript_path(provider: AgentProvider) -> Option<PathBuf> {
    let env_name = provider.current_transcript_env()?;
    std::env::var(env_name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn current_agent_session_id(provider: AgentProvider) -> Option<String> {
    let env_name = provider.current_session_id_env()?;
    std::env::var(env_name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn excluded_session_matches(record: &WorkRecord, excluded_sessions: &HashSet<PathBuf>) -> bool {
    if excluded_sessions.is_empty() || record.kind != WorkRecordKind::ChatTurn {
        return false;
    }

    record
        .session
        .path
        .as_deref()
        .map(Path::new)
        .map(comparable_path)
        .is_some_and(|path| excluded_sessions.contains(&path))
}

fn comparable_path(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn record_snippet(record: &WorkRecord, field: SearchFieldArg) -> String {
    let text = match field {
        SearchFieldArg::Title => record.title.as_str(),
        SearchFieldArg::Session => record.work_ref.session(),
        SearchFieldArg::Input | SearchFieldArg::Command => {
            return snippet(&record.input_text().unwrap_or_default());
        }
        SearchFieldArg::Output => return snippet(&record.output_text().unwrap_or_default()),
        SearchFieldArg::Content | SearchFieldArg::All => return first_content_snippet(record),
    };

    snippet(text)
}

fn first_content_snippet(record: &WorkRecord) -> String {
    combined_text(record)
        .lines()
        .find(|line| !line.trim().is_empty())
        .map(snippet)
        .unwrap_or_default()
}

fn snippet(text: &str) -> String {
    const LIMIT: usize = 160;
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut shortened = collapsed.chars().take(LIMIT).collect::<String>();
    if collapsed.chars().count() > LIMIT {
        shortened.push('…');
    }
    shortened
}

#[cfg(test)]
mod tests {
    use super::*;
    use sivtr_core::record::{
        WorkPart, WorkPartIo, WorkPartKind, WorkRecordKind, WorkRef, WorkRefSelector, WorkStatus,
        WorkTime,
    };

    #[test]
    fn parses_search_targets() {
        assert_eq!(
            "terminal/session_1/3/2".parse::<WorkRefSelector>().unwrap(),
            WorkRefSelector::Terminal {
                session: Some("session_1".to_string()),
                records: Some(vec![3]),
                lines: Some(vec![2]),
            }
        );
        assert_eq!(
            "pi/*/*".parse::<WorkRefSelector>().unwrap(),
            WorkRefSelector::Agent {
                provider: Some(AgentProvider::Pi),
                session: None,
                records: None,
                lines: None,
            }
        );
        assert!(matches!(
            "agent".parse::<WorkRefSelector>().unwrap(),
            WorkRefSelector::Agent { provider: None, .. }
        ));
    }

    #[test]
    fn rejects_invalid_targets() {
        assert!("unknown".parse::<WorkRefSelector>().is_err());
        assert!("pi/session/0".parse::<WorkRefSelector>().is_err());
        assert!("pi/session/one".parse::<WorkRefSelector>().is_err());
    }

    #[test]
    fn matching_refs_returns_line_refs_for_content_matches() {
        let record = test_terminal_record("terminal/session_1/3", "alpha\nneedle\nneedle again");
        let target = "terminal/session_1/3".parse::<WorkRefSelector>().unwrap();
        let regex = Regex::new("needle").unwrap();

        let matches = matching_refs(
            &record,
            Some(&target),
            SearchFieldArg::Content,
            Some(&regex),
        );

        assert_eq!(
            matches
                .iter()
                .map(|item| item.ref_.as_str())
                .collect::<Vec<_>>(),
            vec!["terminal/session_1/3/2", "terminal/session_1/3/3"]
        );
    }

    #[test]
    fn target_line_segment_filters_to_specific_line() {
        let record = test_terminal_record("terminal/session_1/3", "alpha\nneedle");
        let target = "terminal/session_1/3/2".parse::<WorkRefSelector>().unwrap();
        let regex = Regex::new("needle").unwrap();

        let matches = matching_refs(
            &record,
            Some(&target),
            SearchFieldArg::Content,
            Some(&regex),
        );

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].ref_, "terminal/session_1/3/2");
    }

    #[test]
    fn target_selectors_filter_to_multiple_records_and_lines() {
        let record = test_terminal_record("terminal/session_1/3", "alpha\nneedle\nneedle again");
        let target = "terminal/session_1/2-3/2-3"
            .parse::<WorkRefSelector>()
            .unwrap();
        let regex = Regex::new("needle").unwrap();

        let matches = matching_refs(
            &record,
            Some(&target),
            SearchFieldArg::Content,
            Some(&regex),
        );

        assert_eq!(
            matches
                .iter()
                .map(|item| item.ref_.as_str())
                .collect::<Vec<_>>(),
            vec!["terminal/session_1/3/2", "terminal/session_1/3/3"]
        );
    }

    #[test]
    fn match_excluded_filters_matching_snippets() {
        let record =
            test_terminal_record("terminal/session_1/3", "alpha\nneedle example\nneedle real");
        let target = "terminal/session_1/3".parse::<WorkRefSelector>().unwrap();
        let regex = Regex::new("needle").unwrap();
        let exclude = Regex::new("example").unwrap();
        let matches = matching_refs(
            &record,
            Some(&target),
            SearchFieldArg::Content,
            Some(&regex),
        )
        .into_iter()
        .filter(|matched| !match_excluded(matched, SearchFieldArg::Content, Some(&exclude)))
        .collect::<Vec<_>>();

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].ref_, "terminal/session_1/3/3");
        assert_eq!(matches[0].snippet, "needle real");
    }

    #[test]
    fn parses_duration_filters_as_milliseconds() {
        assert_eq!(parse_duration_ms("500").unwrap(), 500);
        assert_eq!(parse_duration_ms("2s").unwrap(), 2_000);
        assert_eq!(parse_duration_ms("3m").unwrap(), 180_000);
    }

    #[test]
    fn filters_exit_code_and_duration() {
        assert!(exit_code_matches(Some(101), Some(101)));
        assert!(!exit_code_matches(Some(101), Some(0)));
        assert!(duration_matches(Some(100), Some(200), Some(150)));
        assert!(!duration_matches(Some(100), Some(200), Some(250)));
        assert!(!duration_matches(Some(100), None, None));
    }

    fn test_terminal_record(_ref_id: &str, combined: &str) -> WorkRecord {
        use sivtr_core::record::{WorkChannel, WorkSessionRef, WorkSource};
        let work_ref = WorkRef::terminal_record("session_1", 3);
        WorkRecord {
            schema_version: 1,
            work_ref,
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
            time: WorkTime::from_components(
                Some("2026-05-24T00:00:00Z".to_string()),
                None,
                Some(150),
            ),
            status: Some(WorkStatus {
                outcome: WorkOutcome::Failure,
                exit_code: Some(101),
            }),
            title: "cargo test".to_string(),
            parts: vec![WorkPart {
                io: WorkPartIo::Output,
                kind: WorkPartKind::Text,
                index: 1,
                occurred_at: None,
                label: None,
                text: combined.to_string(),
                ansi: None,
            }],
        }
    }
}
