//! Compiler frontends translating vendor routing configs into [`RuleSet`] IR.

mod xray;

pub use xray::compile_xray;

use crate::ir::RuleSet;

#[derive(Debug)]
pub struct CompileOutput {
    /// Compiled rule set.
    pub ruleset: RuleSet,
    /// `(rule_index, explanation)` pairs for tolerated deviations (unknown
    /// keys, skipped empty rules, deferred strategy semantics).
    pub warnings: Vec<(usize, String)>,
}

/// Extracts an optional JSON string-or-array-of-strings field as owned
/// strings. Errors when the value is present but neither shape.
pub(crate) fn field_strings(
    v: &serde_json::Value,
    rule_index: usize,
    field: &'static str,
) -> Result<Vec<String>, crate::RouteError> {
    match v {
        serde_json::Value::String(s) => Ok(vec![s.clone()]),
        serde_json::Value::Array(items) => items
            .iter()
            .map(|it| match it {
                serde_json::Value::String(s) => Ok(s.clone()),
                other => Err(non_string(rule_index, field, other)),
            })
            .collect(),
        other => Err(non_string(rule_index, field, other)),
    }
}

fn non_string(rule_index: usize, field: &'static str, v: &serde_json::Value) -> crate::RouteError {
    crate::RouteError::Parse {
        rule_index,
        field,
        message: format!("expected string or string array, got {v}"),
    }
}
