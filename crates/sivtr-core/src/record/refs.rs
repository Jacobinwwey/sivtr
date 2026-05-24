use anyhow::{bail, Result};
use serde::Serialize;
use std::fmt;
use std::str::FromStr;

use crate::ai::AgentProvider;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkRefTarget {
    Record,
    Line(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkRef {
    Terminal {
        session: String,
        record_index: usize,
        target: WorkRefTarget,
    },
    Agent {
        provider: AgentProvider,
        session: String,
        turn_index: usize,
        target: WorkRefTarget,
    },
}

impl WorkRef {
    pub fn terminal_record(session: impl Into<String>, record_index: usize) -> Self {
        Self::Terminal {
            session: session.into(),
            record_index,
            target: WorkRefTarget::Record,
        }
    }

    pub fn agent_record(
        provider: AgentProvider,
        session: impl Into<String>,
        turn_index: usize,
    ) -> Self {
        Self::Agent {
            provider,
            session: session.into(),
            turn_index,
            target: WorkRefTarget::Record,
        }
    }

    pub fn with_line(&self, line: usize) -> Self {
        match self {
            Self::Terminal {
                session,
                record_index,
                ..
            } => Self::Terminal {
                session: session.clone(),
                record_index: *record_index,
                target: WorkRefTarget::Line(line),
            },
            Self::Agent {
                provider,
                session,
                turn_index,
                ..
            } => Self::Agent {
                provider: *provider,
                session: session.clone(),
                turn_index: *turn_index,
                target: WorkRefTarget::Line(line),
            },
        }
    }

    pub fn record_ref(&self) -> Self {
        match self {
            Self::Terminal {
                session,
                record_index,
                ..
            } => Self::terminal_record(session.clone(), *record_index),
            Self::Agent {
                provider,
                session,
                turn_index,
                ..
            } => Self::agent_record(*provider, session.clone(), *turn_index),
        }
    }

    pub fn line(&self) -> Option<usize> {
        match self.target() {
            WorkRefTarget::Record => None,
            WorkRefTarget::Line(line) => Some(line),
        }
    }

    pub fn target(&self) -> WorkRefTarget {
        match self {
            Self::Terminal { target, .. } | Self::Agent { target, .. } => *target,
        }
    }

    pub fn provider(&self) -> Option<AgentProvider> {
        match self {
            Self::Terminal { .. } => None,
            Self::Agent { provider, .. } => Some(*provider),
        }
    }

    pub fn session(&self) -> &str {
        match self {
            Self::Terminal { session, .. } | Self::Agent { session, .. } => session,
        }
    }

    pub fn record_index(&self) -> usize {
        match self {
            Self::Terminal { record_index, .. } => *record_index,
            Self::Agent { turn_index, .. } => *turn_index,
        }
    }
}

impl fmt::Display for WorkRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Terminal {
                session,
                record_index,
                target,
            } => write_parts(
                f,
                &["terminal", session, &record_index.to_string()],
                *target,
            ),
            Self::Agent {
                provider,
                session,
                turn_index,
                target,
            } => write_parts(
                f,
                &[provider.command_name(), session, &turn_index.to_string()],
                *target,
            ),
        }
    }
}

fn write_parts(f: &mut fmt::Formatter<'_>, parts: &[&str], target: WorkRefTarget) -> fmt::Result {
    write!(f, "{}", parts.join("/"))?;
    if let WorkRefTarget::Line(line) = target {
        write!(f, "/{line}")?;
    }
    Ok(())
}

impl Serialize for WorkRef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl FromStr for WorkRef {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        let parts = value
            .split('/')
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>();
        if !(3..=4).contains(&parts.len()) {
            bail!("Invalid work ref `{value}`; expected terminal/<session>/<record>[/line] or <provider>/<session>/<turn>[/line]");
        }

        let target = if parts.len() == 4 {
            WorkRefTarget::Line(parse_one_based(parts[3], "line", value)?)
        } else {
            WorkRefTarget::Record
        };
        let item_index = parse_one_based(parts[2], "record", value)?;

        if parts[0].eq_ignore_ascii_case("terminal") {
            return Ok(Self::Terminal {
                session: parts[1].to_string(),
                record_index: item_index,
                target,
            });
        }

        let provider = AgentProvider::from_command_name(parts[0]).ok_or_else(|| {
            anyhow::anyhow!(
                "Invalid work ref `{value}`; unknown provider `{}`",
                parts[0]
            )
        })?;
        Ok(Self::Agent {
            provider,
            session: parts[1].to_string(),
            turn_index: item_index,
            target,
        })
    }
}

fn parse_one_based(part: &str, label: &str, reference: &str) -> Result<usize> {
    let value = part.parse::<usize>().map_err(|_| {
        anyhow::anyhow!("Invalid work ref `{reference}`; {label} index must be a positive integer")
    })?;
    if value == 0 {
        bail!("Invalid work ref `{reference}`; {label} index must be 1-based");
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_renders_terminal_refs() {
        let reference: WorkRef = "terminal/current/3/12".parse().unwrap();
        assert_eq!(
            reference,
            WorkRef::Terminal {
                session: "current".to_string(),
                record_index: 3,
                target: WorkRefTarget::Line(12),
            }
        );
        assert_eq!(reference.to_string(), "terminal/current/3/12");
        assert_eq!(reference.record_ref().to_string(), "terminal/current/3");
    }

    #[test]
    fn parses_and_renders_agent_refs() {
        let reference: WorkRef = "pi/abcdef12/2".parse().unwrap();
        assert_eq!(
            reference,
            WorkRef::Agent {
                provider: AgentProvider::Pi,
                session: "abcdef12".to_string(),
                turn_index: 2,
                target: WorkRefTarget::Record,
            }
        );
        assert_eq!(reference.with_line(7).to_string(), "pi/abcdef12/2/7");
    }

    #[test]
    fn rejects_zero_indices() {
        assert!("pi/session/0".parse::<WorkRef>().is_err());
        assert!("pi/session/1/0".parse::<WorkRef>().is_err());
    }
}
