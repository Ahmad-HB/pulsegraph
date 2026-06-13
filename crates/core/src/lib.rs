pub mod model;
pub mod pricing;
pub mod parse;
pub mod discovery;
pub mod aggregate;
pub mod stats;

#[cfg(test)]
mod smoke {
    #[test]
    fn workspace_builds() {
        assert_eq!(2 + 2, 4);
    }
}
