//! Decides which formula cells cannot be scored because their value depends on a volatile
//! function, directly or through (statically visible) precedents.

use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use regex::Regex;

use crate::cellref::Addr;
use crate::value::Value;
use crate::xlsx_snapshot::{CellSnapshot, SheetSnapshot};

/// Functions that read the wall clock. Scorable when the workbook pins the clock (see
/// [`pinned_clock_serial`]).
static CLOCK: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(?:_xlfn\.)?(NOW|TODAY)\s*\(").unwrap_or_else(|e| unreachable!("{e}"))
});

/// Functions whose results legitimately differ between runs; never scored.
static RANDOM: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(?:_xlfn\.)?(RAND|RANDBETWEEN|RANDARRAY)\s*\(")
        .unwrap_or_else(|e| unreachable!("{e}"))
});

static STRING_LITERAL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#""(?:[^"]|"")*""#).unwrap_or_else(|e| unreachable!("{e}")));

/// `[Sheet!]A1[:B2]` with an optionally quoted sheet prefix.
static REFERENCE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?:(?:'((?:[^']|'')+)'|([A-Za-z_][A-Za-z0-9_.]*))!)?(\$?[A-Za-z]{1,3}\$?[0-9]+)(?::(\$?[A-Za-z]{1,3}\$?[0-9]+))?",
    )
    .unwrap_or_else(|e| unreachable!("{e}"))
});

/// A formula cell: (sheet name, address, formula text without `=`).
pub type FormulaCell = (String, Addr, String);

/// Every formula cell of a workbook snapshot, as input for [`volatile_cells`].
///
/// Shared-formula members carry the master's text unshifted, so their references may be off
/// (a missed or extra dependency, never a wrong score).
#[must_use]
pub fn formula_cells(sheets: &[SheetSnapshot]) -> Vec<FormulaCell> {
    sheets
        .iter()
        .flat_map(|sheet| {
            sheet.cells.iter().filter_map(|(addr, cell)| {
                let formula = cell.formula.clone().filter(|_| cell.is_formula())?;
                Some((sheet.name.clone(), *addr, formula))
            })
        })
        .collect()
}

/// The formula cells that can be scored: every formula cell not tainted by a volatile function
/// (directly or through precedents), with the number of formula cells skipped as volatile.
#[must_use]
pub fn scorable_cells(
    sheets: &[SheetSnapshot],
    clock_pinned: bool,
) -> (Vec<(&str, Addr, &CellSnapshot)>, usize) {
    let volatile = volatile_cells(&formula_cells(sheets), clock_pinned);
    let mut cells = Vec::new();
    let mut skipped = 0;
    for sheet in sheets {
        for (addr, cell) in &sheet.cells {
            if !cell.is_formula() {
                continue;
            }
            if volatile.contains(&(sheet.name.clone(), *addr)) {
                skipped += 1;
            } else {
                cells.push((sheet.name.as_str(), *addr, cell));
            }
        }
    }
    (cells, skipped)
}

/// Whether the formula itself calls a volatile function.
#[must_use]
pub fn is_volatile(formula: &str, clock_pinned: bool) -> bool {
    RANDOM.is_match(formula) || (!clock_pinned && CLOCK.is_match(formula))
}

/// The Excel serial stored next to the label `NOW` in the first rows of a `METADATA` sheet.
///
/// This is IronCalc's mock-time convention (`rustsheet_xlsx::mock_time`); the engine adapter
/// pins its clock to the same value, so `NOW()`/`TODAY()` cells reproduce Excel's cached values.
#[must_use]
pub fn pinned_clock_serial(sheets: &[SheetSnapshot]) -> Option<f64> {
    let sheet = sheets
        .iter()
        .find(|s| s.name.eq_ignore_ascii_case("METADATA"))?;
    (1..=32).find_map(|row| {
        let label = sheet.cells.get(&Addr { row, col: 1 })?;
        let serial = sheet.cells.get(&Addr { row, col: 2 })?;
        match (&label.value, &serial.value) {
            (Value::Text(t), Value::Number(n)) if t == "NOW" => Some(*n),
            _ => None,
        }
    })
}

/// Static precedents of a formula as `(lowercased sheet, top-left, bottom-right)`. Names,
/// `INDIRECT`/`OFFSET`, whole-row/column references and 3-D references are not followed.
fn precedents(sheet: &str, formula: &str) -> Vec<(String, Addr, Addr)> {
    let text = STRING_LITERAL.replace_all(formula, "\"\"");
    let bytes = text.as_bytes();
    let is_word = |b: u8| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b'$');
    let mut out = Vec::new();
    for caps in REFERENCE.captures_iter(&text) {
        let Some(whole) = caps.get(0) else { continue };
        let qualified = caps.get(1).or_else(|| caps.get(2));
        let boundary_before = whole.start() == 0 || !is_word(bytes[whole.start() - 1]);
        let followed_by_call = bytes.get(whole.end()).is_some_and(|&b| b == b'(');
        if qualified.is_none() && (!boundary_before || followed_by_call) {
            continue;
        }
        let (Some(first), second) = (caps.get(3), caps.get(4)) else {
            continue;
        };
        let (Some(lo), hi) = (
            Addr::parse(first.as_str()),
            second.map_or_else(|| Addr::parse(first.as_str()), |m| Addr::parse(m.as_str())),
        ) else {
            continue;
        };
        let Some(hi) = hi else { continue };
        let target = qualified.map_or_else(|| sheet.to_lowercase(), |m| unquote(m.as_str()));
        out.push((
            target,
            Addr {
                row: lo.row.min(hi.row),
                col: lo.col.min(hi.col),
            },
            Addr {
                row: lo.row.max(hi.row),
                col: lo.col.max(hi.col),
            },
        ));
    }
    out
}

fn unquote(name: &str) -> String {
    name.replace("''", "'").to_lowercase()
}

/// Returns every formula cell (as `(sheet, addr)`) that is volatile or transitively reads a
/// volatile cell through static references.
#[must_use]
pub fn volatile_cells(cells: &[FormulaCell], clock_pinned: bool) -> HashSet<(String, Addr)> {
    let mut single: HashMap<(String, Addr), Vec<usize>> = HashMap::new();
    let mut ranges: Vec<(String, Addr, Addr, usize)> = Vec::new();
    for (idx, (sheet, _, formula)) in cells.iter().enumerate() {
        for (target, lo, hi) in precedents(sheet, formula) {
            if lo == hi {
                single.entry((target, lo)).or_default().push(idx);
            } else {
                ranges.push((target, lo, hi, idx));
            }
        }
    }
    let mut tainted = vec![false; cells.len()];
    let mut queue: Vec<usize> = Vec::new();
    for (idx, (_, _, formula)) in cells.iter().enumerate() {
        if is_volatile(formula, clock_pinned) {
            tainted[idx] = true;
            queue.push(idx);
        }
    }
    while let Some(idx) = queue.pop() {
        let (sheet, addr, _) = &cells[idx];
        let key = sheet.to_lowercase();
        let by_cell = single.get(&(key.clone(), *addr)).into_iter().flatten();
        let by_range = ranges
            .iter()
            .filter(|(s, lo, hi, _)| {
                *s == key
                    && (lo.row..=hi.row).contains(&addr.row)
                    && (lo.col..=hi.col).contains(&addr.col)
            })
            .map(|(.., dependent)| dependent);
        for &dependent in by_cell.chain(by_range) {
            if !tainted[dependent] {
                tainted[dependent] = true;
                queue.push(dependent);
            }
        }
    }
    cells
        .iter()
        .zip(tainted)
        .filter(|(_, t)| *t)
        .map(|((sheet, addr, _), _)| (sheet.clone(), *addr))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cell(sheet: &str, a1: &str, formula: &str) -> FormulaCell {
        (
            sheet.into(),
            Addr::parse(a1).expect("valid test address"),
            formula.into(),
        )
    }

    fn tainted(cells: &[FormulaCell], pinned: bool) -> Vec<String> {
        let set = volatile_cells(cells, pinned);
        let mut keys: Vec<String> = set.iter().map(|(s, a)| format!("{s}!{a}")).collect();
        keys.sort();
        keys
    }

    #[test]
    fn propagates_through_chains_ranges_and_sheets() {
        let cells = vec![
            cell("S", "A1", "RAND()"),
            cell("S", "A2", "A1+1"),
            cell("S", "A3", "SUM(A1:A2)"),
            cell("T", "B1", "'S'!A3*2"),
            cell("S", "C1", "LOG10(100)+\"A1\""),
            cell("S", "C2", "SUM(B5:B9)"),
        ];
        assert_eq!(
            tainted(&cells, false),
            ["S!A1", "S!A2", "S!A3", "T!B1"],
            "LOG10( and the string \"A1\" are not references"
        );
    }

    #[test]
    fn pinned_clock_scores_now_but_never_rand() {
        let cells = vec![
            cell("S", "A1", "TODAY()+1"),
            cell("S", "A2", "A1*2"),
            cell("S", "B1", "RANDBETWEEN(1,2)"),
            cell("S", "B2", "B1"),
        ];
        assert_eq!(tainted(&cells, true), ["S!B1", "S!B2"]);
        assert_eq!(tainted(&cells, false).len(), 4);
    }

    #[test]
    fn reads_the_metadata_now_row() {
        use crate::xlsx_snapshot::CellSnapshot;
        let constant = |value| CellSnapshot {
            formula: None,
            formula_kind: None,
            value,
        };
        let sheet = |name: &str, label: &str| SheetSnapshot {
            name: name.into(),
            cells: [
                (Addr { row: 3, col: 1 }, constant(Value::Text(label.into()))),
                (Addr { row: 3, col: 2 }, constant(Value::Number(45000.5))),
            ]
            .into(),
        };
        assert_eq!(
            pinned_clock_serial(&[sheet("metadata", "NOW")]),
            Some(45000.5)
        );
        assert_eq!(pinned_clock_serial(&[sheet("METADATA", "EPS")]), None);
        assert_eq!(pinned_clock_serial(&[sheet("Other", "NOW")]), None);
    }
}
