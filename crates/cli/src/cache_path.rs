use std::path::PathBuf;

/// Where the CLI/app stores its incremental cache DB.
pub fn cache_db_path() -> PathBuf {
    if let Ok(dir) = std::env::var("PULSEGRAPH_CACHE_DIR") {
        return PathBuf::from(dir).join("cache.sqlite");
    }
    directories::ProjectDirs::from("dev", "pulsegraph", "PulseGraph")
        .map(|d| d.cache_dir().join("cache.sqlite"))
        .unwrap_or_else(|| PathBuf::from("pulsegraph-cache.sqlite"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn honors_env_override() {
        std::env::set_var("PULSEGRAPH_CACHE_DIR", "/tmp/tp-test");
        let p = cache_db_path();
        assert_eq!(p, PathBuf::from("/tmp/tp-test/cache.sqlite"));
        std::env::remove_var("PULSEGRAPH_CACHE_DIR");
    }
}
