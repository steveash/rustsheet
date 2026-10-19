//! The committed copy of the engine's SPEC-0011 argument-kind table, `eval/catalog/lift-kinds.json`
//! (T-1776). tools/ checks the catalog's parameter modes and `lift: arrays` keys against it, so the
//! Python side never builds the engine; [`render`] is what `rustsheet-eval lift-kinds --write`
//! writes, and a unit test fails when the committed file differs.

use rustsheet_engine::lift_kinds_table;

/// Path of the committed table, relative to the repository root.
pub const LIFT_KINDS_PATH: &str = "eval/catalog/lift-kinds.json";

/// The table as the committed file holds it: a JSON array with one function per line, so a
/// table change diffs as the rows it touches.
///
/// # Errors
/// Fails only if serialization fails, which plain strings and numbers never do.
pub fn render() -> Result<String, serde_json::Error> {
    let rows = lift_kinds_table()
        .iter()
        .map(|row| serde_json::to_string(row).map(|line| format!("  {line}")))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(format!("[\n{}\n]\n", rows.join(",\n")))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The committed file matches the engine's table: run `rustsheet-eval lift-kinds --write`
    /// after changing `functions/lift/kinds/`.
    #[test]
    fn committed_table_is_current() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(LIFT_KINDS_PATH);
        let committed = std::fs::read_to_string(&path).expect("read eval/catalog/lift-kinds.json");
        assert!(
            committed == render().unwrap(),
            "{LIFT_KINDS_PATH} is stale: run `rustsheet-eval lift-kinds --write`"
        );
    }
}
