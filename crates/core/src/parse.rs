use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::path::Path;
use crate::model::{TokenCounts, UsageEvent};

#[derive(Deserialize)]
struct Line {
    timestamp: Option<DateTime<Utc>>,
    #[serde(rename = "sessionId")]
    session_id: Option<String>,
    cwd: Option<String>,
    message: Option<Message>,
}

#[derive(Deserialize)]
struct Message {
    model: Option<String>,
    usage: Option<Usage>,
}

#[derive(Deserialize)]
struct Usage {
    #[serde(default)]
    input_tokens: u64,
    #[serde(default)]
    output_tokens: u64,
    #[serde(default)]
    cache_read_input_tokens: u64,
    #[serde(default)]
    cache_creation: Option<CacheCreation>,
    #[serde(default)]
    cache_creation_input_tokens: u64,
}

#[derive(Deserialize)]
struct CacheCreation {
    #[serde(default)]
    ephemeral_5m_input_tokens: u64,
    #[serde(default)]
    ephemeral_1h_input_tokens: u64,
}

/// Parse one JSONL line. `Ok(None)` for lines without usage (user turns, summaries,
/// blank lines); `Err` only for malformed JSON.
pub fn parse_line(line: &str) -> Result<Option<UsageEvent>, serde_json::Error> {
    let line = line.trim();
    if line.is_empty() {
        return Ok(None);
    }
    let parsed: Line = serde_json::from_str(line)?;
    let (Some(message), Some(timestamp)) = (parsed.message, parsed.timestamp) else {
        return Ok(None);
    };
    let Some(usage) = message.usage else {
        return Ok(None);
    };

    let (cw5m, cw1h) = match usage.cache_creation {
        Some(c) => (c.ephemeral_5m_input_tokens, c.ephemeral_1h_input_tokens),
        // No split available: attribute the lump-sum total to the 5m bucket.
        None => (usage.cache_creation_input_tokens, 0),
    };

    let project = parsed
        .cwd
        .as_deref()
        .map(project_name)
        .unwrap_or_else(|| "unknown".to_string());

    Ok(Some(UsageEvent {
        source: "claude-code".to_string(),
        timestamp,
        model: message.model.unwrap_or_else(|| "unknown".to_string()),
        project,
        session_id: parsed.session_id.unwrap_or_default(),
        tokens: TokenCounts {
            input: usage.input_tokens,
            output: usage.output_tokens,
            cache_write_5m: cw5m,
            cache_write_1h: cw1h,
            cache_read: usage.cache_read_input_tokens,
        },
    }))
}

/// Display name for a project = last path component of its cwd.
pub fn project_name(cwd: &str) -> String {
    Path::new(cwd)
        .file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| cwd.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const ASSISTANT_LINE: &str = r#"{"timestamp":"2026-06-13T05:28:05.205Z","type":"assistant","sessionId":"abc","cwd":"/Users/ahmad/RiderProjects/TraderVolt/API","message":{"model":"claude-opus-4-8","usage":{"input_tokens":8259,"output_tokens":341,"cache_read_input_tokens":16280,"cache_creation_input_tokens":13315,"cache_creation":{"ephemeral_5m_input_tokens":0,"ephemeral_1h_input_tokens":13315}}}}"#;

    #[test]
    fn parses_assistant_usage_line() {
        let ev = parse_line(ASSISTANT_LINE).unwrap().unwrap();
        assert_eq!(ev.model, "claude-opus-4-8");
        assert_eq!(ev.session_id, "abc");
        assert_eq!(ev.project, "API");
        assert_eq!(ev.source, "claude-code");
        assert_eq!(ev.tokens.input, 8259);
        assert_eq!(ev.tokens.output, 341);
        assert_eq!(ev.tokens.cache_read, 16280);
        assert_eq!(ev.tokens.cache_write_1h, 13315);
        assert_eq!(ev.tokens.cache_write_5m, 0);
    }

    #[test]
    fn line_without_usage_returns_none() {
        let user_line = r#"{"timestamp":"2026-06-13T05:28:05.205Z","type":"user","sessionId":"abc","cwd":"/x/y","message":{"role":"user"}}"#;
        assert!(parse_line(user_line).unwrap().is_none());
    }

    #[test]
    fn blank_line_returns_none() {
        assert!(parse_line("   ").unwrap().is_none());
    }

    #[test]
    fn malformed_json_is_err() {
        assert!(parse_line("{not json").is_err());
    }

    #[test]
    fn falls_back_to_total_cache_creation_when_split_absent() {
        let line = r#"{"timestamp":"2026-06-13T05:28:05.205Z","sessionId":"s","cwd":"/a/b","message":{"model":"m","usage":{"input_tokens":1,"output_tokens":2,"cache_creation_input_tokens":99}}}"#;
        let ev = parse_line(line).unwrap().unwrap();
        // No ephemeral split present → attribute the total to 5m (Claude Code's default bucket).
        assert_eq!(ev.tokens.cache_write_5m, 99);
        assert_eq!(ev.tokens.cache_write_1h, 0);
    }

    #[test]
    fn project_name_is_basename_of_cwd() {
        assert_eq!(project_name("/Users/ahmad/RiderProjects/TraderVolt/API"), "API");
        assert_eq!(project_name("/"), "/");
    }
}
