use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Default Claude Code transcripts directory: `$CLAUDE_CONFIG_DIR/projects`,
/// else `~/.claude/projects`.
pub fn default_projects_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("CLAUDE_CONFIG_DIR") {
        return Some(PathBuf::from(dir).join("projects"));
    }
    directories::BaseDirs::new().map(|b| b.home_dir().join(".claude").join("projects"))
}

/// All `*.jsonl` files under `projects_dir` (recursive). Missing dir → empty Vec.
pub fn find_transcripts(projects_dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    if !projects_dir.exists() {
        return Ok(out);
    }
    for entry in WalkDir::new(projects_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
            out.push(path.to_path_buf());
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn finds_jsonl_files_recursively() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let proj = root.join("-Users-ahmad-Proj");
        fs::create_dir_all(&proj).unwrap();
        fs::write(proj.join("session1.jsonl"), "{}").unwrap();
        fs::write(proj.join("session2.jsonl"), "{}").unwrap();
        fs::write(proj.join("notes.txt"), "ignore me").unwrap();

        let mut found = find_transcripts(root).unwrap();
        found.sort();
        assert_eq!(found.len(), 2);
        assert!(found.iter().all(|p| p.extension().unwrap() == "jsonl"));
    }

    #[test]
    fn missing_dir_returns_empty() {
        let found = find_transcripts(Path::new("/no/such/dir/anywhere")).unwrap();
        assert!(found.is_empty());
    }
}
