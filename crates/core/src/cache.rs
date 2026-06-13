use std::path::{Path, PathBuf};
use rusqlite::Connection;
use crate::model::UsageEvent;

/// Per-file cache of parsed events keyed by (path, mtime). Stored as a SQLite DB.
pub struct Cache {
    conn: Connection,
}

impl Cache {
    /// Open (creating if needed) the cache DB. If the existing file is not a valid
    /// SQLite DB or lacks our schema, it is deleted and recreated.
    pub fn open(db_path: &Path) -> rusqlite::Result<Self> {
        if let Some(parent) = db_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match Self::try_open(db_path) {
            Ok(c) => Ok(c),
            Err(_) => {
                let _ = std::fs::remove_file(db_path);
                Self::try_open(db_path)
            }
        }
    }

    fn try_open(db_path: &Path) -> rusqlite::Result<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS files (
                 path  TEXT PRIMARY KEY,
                 mtime INTEGER NOT NULL,
                 events TEXT NOT NULL
             );",
        )?;
        // Touch the table to force a read; a corrupt file errors here and triggers rebuild.
        conn.query_row("SELECT count(*) FROM files", [], |r| r.get::<_, i64>(0))?;
        Ok(Self { conn })
    }

    /// Cached events for `path` iff the stored mtime matches.
    pub fn get(&self, path: &Path, mtime: i64) -> rusqlite::Result<Option<Vec<UsageEvent>>> {
        let key = path.to_string_lossy();
        let row: Option<(i64, String)> = self
            .conn
            .query_row(
                "SELECT mtime, events FROM files WHERE path = ?1",
                [key.as_ref()],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .ok();
        match row {
            Some((stored, json)) if stored == mtime => {
                let evs = serde_json::from_str(&json).unwrap_or_default();
                Ok(Some(evs))
            }
            _ => Ok(None),
        }
    }

    /// Upsert the parsed events for `path` at `mtime`.
    pub fn put(&mut self, path: &Path, mtime: i64, events: &[UsageEvent]) -> rusqlite::Result<()> {
        let key = path.to_string_lossy();
        let json = serde_json::to_string(events).unwrap_or_else(|_| "[]".to_string());
        self.conn.execute(
            "INSERT INTO files (path, mtime, events) VALUES (?1, ?2, ?3)
             ON CONFLICT(path) DO UPDATE SET mtime = excluded.mtime, events = excluded.events",
            rusqlite::params![key.as_ref(), mtime, json],
        )?;
        Ok(())
    }

    /// Drop cache rows for files no longer present.
    pub fn prune(&mut self, keep: &[PathBuf]) -> rusqlite::Result<()> {
        let keep_set: std::collections::HashSet<String> =
            keep.iter().map(|p| p.to_string_lossy().into_owned()).collect();
        let existing: Vec<String> = {
            let mut stmt = self.conn.prepare("SELECT path FROM files")?;
            let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
            rows.filter_map(|r| r.ok()).collect()
        };
        for path in existing {
            if !keep_set.contains(&path) {
                self.conn.execute("DELETE FROM files WHERE path = ?1", [path])?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::TokenCounts;
    use chrono::Utc;
    use tempfile::tempdir;

    fn sample_events() -> Vec<UsageEvent> {
        vec![UsageEvent {
            source: "claude-code".into(),
            timestamp: Utc::now(),
            model: "claude-opus-4-8".into(),
            project: "API".into(),
            session_id: "s1".into(),
            tokens: TokenCounts { input: 5, output: 6, cache_write_5m: 1, cache_write_1h: 2, cache_read: 3 },
        }]
    }

    #[test]
    fn put_then_get_roundtrips_events() {
        let dir = tempdir().unwrap();
        let mut cache = Cache::open(&dir.path().join("c.sqlite")).unwrap();
        let path = Path::new("/x/session.jsonl");
        cache.put(path, 1000, &sample_events()).unwrap();

        let hit = cache.get(path, 1000).unwrap();
        assert!(hit.is_some());
        let evs = hit.unwrap();
        assert_eq!(evs.len(), 1);
        assert_eq!(evs[0].tokens.input, 5);
        assert_eq!(evs[0].project, "API");
    }

    #[test]
    fn get_miss_on_changed_mtime() {
        let dir = tempdir().unwrap();
        let mut cache = Cache::open(&dir.path().join("c.sqlite")).unwrap();
        let path = Path::new("/x/session.jsonl");
        cache.put(path, 1000, &sample_events()).unwrap();
        assert!(cache.get(path, 2000).unwrap().is_none()); // mtime changed → miss
    }

    #[test]
    fn put_replaces_prior_entry_for_same_path() {
        let dir = tempdir().unwrap();
        let mut cache = Cache::open(&dir.path().join("c.sqlite")).unwrap();
        let path = Path::new("/x/session.jsonl");
        cache.put(path, 1000, &sample_events()).unwrap();
        cache.put(path, 2000, &sample_events()).unwrap();
        assert!(cache.get(path, 1000).unwrap().is_none());
        assert!(cache.get(path, 2000).unwrap().is_some());
    }

    #[test]
    fn prune_removes_paths_not_in_keep_set() {
        let dir = tempdir().unwrap();
        let mut cache = Cache::open(&dir.path().join("c.sqlite")).unwrap();
        cache.put(Path::new("/x/a.jsonl"), 1, &sample_events()).unwrap();
        cache.put(Path::new("/x/b.jsonl"), 1, &sample_events()).unwrap();
        let keep: Vec<PathBuf> = vec![PathBuf::from("/x/a.jsonl")];
        cache.prune(&keep).unwrap();
        assert!(cache.get(Path::new("/x/a.jsonl"), 1).unwrap().is_some());
        assert!(cache.get(Path::new("/x/b.jsonl"), 1).unwrap().is_none());
    }

    #[test]
    fn corrupt_db_file_is_rebuilt() {
        let dir = tempdir().unwrap();
        let dbpath = dir.path().join("c.sqlite");
        std::fs::write(&dbpath, b"this is not a sqlite database").unwrap();
        // open must recover by recreating the file rather than erroring.
        let mut cache = Cache::open(&dbpath).unwrap();
        cache.put(Path::new("/x/a.jsonl"), 1, &sample_events()).unwrap();
        assert!(cache.get(Path::new("/x/a.jsonl"), 1).unwrap().is_some());
    }
}
