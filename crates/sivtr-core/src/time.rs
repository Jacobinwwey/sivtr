use chrono::{DateTime, Duration, Local, LocalResult, NaiveDate, NaiveDateTime, TimeZone, Utc};

pub fn parse_timestamp(value: &str) -> Option<DateTime<Utc>> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Ok(timestamp) = DateTime::parse_from_rfc3339(trimmed) {
        return Some(timestamp.with_timezone(&Utc));
    }
    if let Some(timestamp) = parse_human_local_timestamp(trimmed) {
        return Some(timestamp);
    }
    if let Some(timestamp) = parse_unix_timestamp(trimmed) {
        return Some(timestamp);
    }
    if let Ok(date) = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        let local = date.and_hms_opt(0, 0, 0)?;
        return local_to_utc(local);
    }
    if let Ok(datetime) = NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%d %H:%M:%S") {
        return local_to_utc(datetime);
    }
    if let Ok(datetime) = NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%dT%H:%M:%S") {
        return local_to_utc(datetime);
    }

    None
}

pub fn format_local_timestamp(value: DateTime<Utc>) -> String {
    value
        .with_timezone(&Local)
        .format("%Y-%m-%dT%H:%M:%S%.3f%:z")
        .to_string()
}

pub fn normalize_timestamp(value: &str) -> Option<String> {
    Some(format_local_timestamp(parse_timestamp(value)?))
}

pub fn derive_started_at(ended_at: Option<&str>, duration_ms: Option<u64>) -> Option<String> {
    let ended_at = ended_at.and_then(parse_timestamp)?;
    let duration = Duration::milliseconds(i64::try_from(duration_ms?).ok()?);
    Some(format_local_timestamp(ended_at - duration))
}

pub fn derive_ended_at(started_at: Option<&str>, duration_ms: Option<u64>) -> Option<String> {
    let started_at = started_at.and_then(parse_timestamp)?;
    let duration = Duration::milliseconds(i64::try_from(duration_ms?).ok()?);
    Some(format_local_timestamp(started_at + duration))
}

pub fn duration_between_ms(started_at: Option<&str>, ended_at: Option<&str>) -> Option<u64> {
    let started_at = started_at.and_then(parse_timestamp)?;
    let ended_at = ended_at.and_then(parse_timestamp)?;
    (ended_at - started_at).num_milliseconds().try_into().ok()
}

fn parse_human_local_timestamp(value: &str) -> Option<DateTime<Utc>> {
    let formats = [
        "%a %b %e %H:%M:%S %Y",
        "%a %b %d %H:%M:%S %Y",
        "%A %B %e %H:%M:%S %Y",
        "%A %B %d %H:%M:%S %Y",
    ];
    formats
        .iter()
        .find_map(|format| NaiveDateTime::parse_from_str(value, format).ok())
        .and_then(local_to_utc)
}

fn parse_unix_timestamp(value: &str) -> Option<DateTime<Utc>> {
    let number = value.parse::<i64>().ok()?;
    let seconds = if value.len() >= 13 {
        number / 1000
    } else {
        number
    };
    let nanos = if value.len() >= 13 {
        (number % 1000).unsigned_abs() as u32 * 1_000_000
    } else {
        0
    };
    Utc.timestamp_opt(seconds, nanos).single()
}

fn local_to_utc(datetime: NaiveDateTime) -> Option<DateTime<Utc>> {
    match Local.from_local_datetime(&datetime) {
        LocalResult::Single(value) => Some(value.with_timezone(&Utc)),
        LocalResult::Ambiguous(earliest, _) => Some(earliest.with_timezone(&Utc)),
        LocalResult::None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_supported_timestamp_shapes() {
        assert_eq!(
            parse_timestamp("2026-05-23T12:00:00Z").unwrap(),
            DateTime::parse_from_rfc3339("2026-05-23T12:00:00Z")
                .unwrap()
                .with_timezone(&Utc)
        );
        assert_eq!(
            parse_timestamp("1779537600000").unwrap(),
            DateTime::parse_from_rfc3339("2026-05-23T12:00:00Z")
                .unwrap()
                .with_timezone(&Utc)
        );
    }

    #[test]
    fn parses_human_local_timestamps_from_shell_history() {
        assert!(parse_timestamp("Mon May 25 00:35:02 2026").is_some());
        assert!(parse_timestamp("Sun May 24 23:57:35 2026").is_some());
    }

    #[test]
    fn normalizes_timestamps_to_local_rfc3339_with_offset() {
        let normalized = normalize_timestamp("2026-05-23T12:00:00Z").unwrap();
        assert!(normalized.contains('T'));
        assert!(!normalized.ends_with('Z'));
        assert!(parse_timestamp(&normalized).is_some());
    }

    #[test]
    fn derives_missing_time_component_from_other_two() {
        assert_eq!(
            parse_timestamp(&derive_started_at(Some("2026-05-23T12:00:01Z"), Some(1_000)).unwrap()),
            parse_timestamp("2026-05-23T12:00:00Z")
        );
        assert_eq!(
            parse_timestamp(&derive_ended_at(Some("2026-05-23T12:00:00Z"), Some(1_000)).unwrap()),
            parse_timestamp("2026-05-23T12:00:01Z")
        );
        assert_eq!(
            duration_between_ms(Some("2026-05-23T12:00:00Z"), Some("2026-05-23T12:00:01Z")),
            Some(1_000)
        );
    }
}
