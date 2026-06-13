use std::collections::HashMap;
use serde::Deserialize;
use crate::model::TokenCounts;

/// USD per 1,000,000 tokens for base input and output. Cache rates are derived
/// from `input` via the documented multipliers (write 5m = 1.25x, write 1h = 2x,
/// read = 0.1x), so the table only needs two numbers per model.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct ModelPrice {
    pub input: f64,
    pub output: f64,
}

const CACHE_WRITE_5M_MULT: f64 = 1.25;
const CACHE_WRITE_1H_MULT: f64 = 2.0;
const CACHE_READ_MULT: f64 = 0.1;

#[derive(Debug, Clone, Default)]
pub struct Pricing {
    table: HashMap<String, ModelPrice>,
}

impl Pricing {
    pub fn from_map(table: HashMap<String, ModelPrice>) -> Self {
        Self { table }
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let table: HashMap<String, ModelPrice> = serde_json::from_str(json)?;
        Ok(Self { table })
    }

    /// The pricing.json shipped with the binary.
    pub fn bundled() -> Self {
        // include_str! embeds the file at compile time; it cannot fail at runtime,
        // and from_json on known-valid JSON cannot fail either.
        Self::from_json(include_str!("../pricing.json")).expect("bundled pricing.json is valid")
    }

    /// Estimated USD cost. Returns None for an unknown model so callers can show "—".
    pub fn cost(&self, model: &str, t: &TokenCounts) -> Option<f64> {
        let p = self.table.get(model)?;
        let per = |toks: u64, price: f64| toks as f64 / 1_000_000.0 * price;
        Some(
            per(t.input, p.input)
                + per(t.output, p.output)
                + per(t.cache_write_5m, p.input * CACHE_WRITE_5M_MULT)
                + per(t.cache_write_1h, p.input * CACHE_WRITE_1H_MULT)
                + per(t.cache_read, p.input * CACHE_READ_MULT),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Pricing {
        let mut table = HashMap::new();
        table.insert("claude-opus-4-8".to_string(), ModelPrice { input: 5.0, output: 25.0 });
        Pricing::from_map(table)
    }

    #[test]
    fn cost_sums_all_token_classes() {
        let p = fixture();
        let t = TokenCounts {
            input: 1_000_000,
            output: 1_000_000,
            cache_write_5m: 1_000_000,
            cache_write_1h: 1_000_000,
            cache_read: 1_000_000,
        };
        // 5 + 25 + (5*1.25) + (5*2.0) + (5*0.1) = 5 + 25 + 6.25 + 10 + 0.5 = 46.75
        let c = p.cost("claude-opus-4-8", &t).unwrap();
        assert!((c - 46.75).abs() < 1e-9, "got {c}");
    }

    #[test]
    fn unknown_model_returns_none() {
        let p = fixture();
        let t = TokenCounts { input: 100, ..Default::default() };
        assert!(p.cost("nonexistent-model", &t).is_none());
    }

    #[test]
    fn bundled_table_loads_and_has_opus() {
        let p = Pricing::bundled();
        assert!(p.cost("claude-opus-4-8", &TokenCounts { input: 1_000_000, ..Default::default() }).is_some());
    }
}
