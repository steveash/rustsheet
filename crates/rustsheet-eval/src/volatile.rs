//! Decides which formula cells cannot be scored because their value depends on a volatile
//! function, directly or through (statically visible) precedents.

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use regex::Regex;

use crate::case::split_qualified;
use crate::cellref::Addr;
use crate::value::Value;
use crate::xlsx_snapshot::{CellSnapshot, DefinedName, SheetSnapshot};

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

/// A name token; whether it is a defined name is decided by lookup.
static WORD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[A-Za-z_\\][A-Za-z0-9_.\\]*").unwrap_or_else(|e| unreachable!("{e}"))
});

/// A formula cell: (sheet name, address, formula text without `=`).
pub type FormulaCell = (String, Addr, String);

/// Every formula cell of a workbook snapshot, as input for [`volatile_cells`].
///
/// Shared-formula members carry the master's text shifted to their own cell (T-0027), so their
/// precedents are the ones Excel computes with.
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
pub fn scorable_cells<'a>(
    sheets: &'a [SheetSnapshot],
    names: &[DefinedName],
    clock_pinned: bool,
) -> (Vec<(&'a str, Addr, &'a CellSnapshot)>, usize) {
    let volatile = volatile_cells(&formula_cells(sheets), names, clock_pinned, &HashSet::new());
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
/// This is IronCalc's METADATA clock convention (`rustsheet_xlsx::metadata_clock`); the engine
/// adapter pins the workbook clock to the same value, so `NOW()`/`TODAY()` cells reproduce Excel's cached values.
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

/// The formula text followed by the body of every defined name it mentions, transitively, so
/// that precedents and volatile calls reached through names are seen (T-0045). A sheet-scoped
/// name shadows a global one; each name is expanded once, which ends reference cycles.
fn with_names<'a>(sheet: &str, formula: &'a str, names: &[DefinedName]) -> Cow<'a, str> {
    if names.is_empty() {
        return Cow::Borrowed(formula);
    }
    let find = |word: &str| {
        let by = |scoped: bool| {
            names.iter().find(|n| {
                n.name.eq_ignore_ascii_case(word)
                    && match &n.sheet {
                        Some(s) => scoped && s.eq_ignore_ascii_case(sheet),
                        None => !scoped,
                    }
            })
        };
        by(true).or_else(|| by(false))
    };
    let mut out = formula.to_string();
    let mut seen: HashSet<String> = HashSet::new();
    let mut scan_from = 0;
    while scan_from < out.len() {
        let text = STRING_LITERAL
            .replace_all(&out[scan_from..], "\"\"")
            .into_owned();
        scan_from = out.len();
        let mut added = String::new();
        for m in WORD.find_iter(&text) {
            if text.as_bytes().get(m.end()) == Some(&b'(') {
                continue;
            }
            if let Some(name) = find(m.as_str()) {
                if seen.insert(m.as_str().to_lowercase()) {
                    added.push(' ');
                    added.push_str(&name.body);
                }
            }
        }
        out.push_str(&added);
    }
    Cow::Owned(out)
}

/// Static precedents of a formula as `(lowercased sheet, top-left, bottom-right)`. Names
/// are expanded beforehand by [`with_names`]; `INDIRECT`/`OFFSET`, whole-row/column references and 3-D references are not followed.
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
///
/// `invariant` lists cells, as `(lowercased sheet, addr)`, that a case declares run-invariant
/// (T-1283): they call a volatile function or read a volatile cell, but their value cannot change
/// between runs (`RANDBETWEEN(5,5)`, `ISNUMBER(NOW())`). They are never tainted, so they are scored,
/// and taint does not flow through them to their dependents.
#[must_use]
pub fn volatile_cells<S: std::hash::BuildHasher>(
    cells: &[FormulaCell],
    names: &[DefinedName],
    clock_pinned: bool,
    invariant: &HashSet<(String, Addr), S>,
) -> HashSet<(String, Addr)> {
    let expanded: Vec<Cow<'_, str>> = cells
        .iter()
        .map(|(sheet, _, formula)| with_names(sheet, formula, names))
        .collect();
    let mut single: HashMap<(String, Addr), Vec<usize>> = HashMap::new();
    let mut ranges: Vec<(String, Addr, Addr, usize)> = Vec::new();
    for (idx, ((sheet, _, _), formula)) in cells.iter().zip(&expanded).enumerate() {
        for (target, lo, hi) in precedents(sheet, formula) {
            if lo == hi {
                single.entry((target, lo)).or_default().push(idx);
            } else {
                ranges.push((target, lo, hi, idx));
            }
        }
    }
    let listed: Vec<bool> = cells
        .iter()
        .map(|(sheet, addr, _)| invariant.contains(&(sheet.to_lowercase(), *addr)))
        .collect();
    let mut tainted = vec![false; cells.len()];
    let mut queue: Vec<usize> = Vec::new();
    for (idx, formula) in expanded.iter().enumerate() {
        if !listed[idx] && is_volatile(formula, clock_pinned) {
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
            if !tainted[dependent] && !listed[dependent] {
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

/// The volatile cells of a spec case once its `invariant` cells are taken out. A listed cell that
/// is not a volatile formula cell is an error, so a stale listing cannot linger unnoticed.
pub(crate) fn spec_volatile_cells(
    listed: &[String],
    formulas: &[FormulaCell],
) -> Result<HashSet<(String, Addr)>, String> {
    let volatile = volatile_cells(formulas, &[], false, &HashSet::new());
    if listed.is_empty() {
        return Ok(volatile);
    }
    let mut invariant = HashSet::new();
    for q in listed {
        let (sheet, a1) = split_qualified(q).ok_or_else(|| format!("bad cell key {q}"))?;
        let addr = Addr::parse(&a1).ok_or_else(|| format!("bad cell key {q}"))?;
        if !volatile.contains(&(sheet.clone(), addr)) {
            return Err(format!("invariant cell {q} is not a volatile formula cell"));
        }
        invariant.insert((sheet.to_lowercase(), addr));
    }
    Ok(volatile_cells(formulas, &[], false, &invariant))
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
        let set = volatile_cells(cells, &[], pinned, &HashSet::new());
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

    fn name(name: &str, sheet: Option<&str>, body: &str) -> DefinedName {
        DefinedName {
            name: name.into(),
            sheet: sheet.map(Into::into),
            body: body.into(),
        }
    }

    fn tainted_with(cells: &[FormulaCell], names: &[DefinedName]) -> Vec<String> {
        let set = volatile_cells(cells, names, false, &HashSet::new());
        let mut keys: Vec<String> = set.iter().map(|(s, a)| format!("{s}!{a}")).collect();
        keys.sort();
        keys
    }

    #[test]
    fn names_pointing_at_volatile_cells_or_calls_taint_users() {
        let cells = vec![
            cell("S", "A1", "TODAY()"),
            cell("S", "B1", "EDATE(DATE_CELL,2)+\"DATE_CELL\""),
            cell("S", "B2", "NOWISH+1"),
            cell("S", "B3", "Plain+1"),
            cell("S", "B4", "DATE_CELL(1)"),
        ];
        let names = [
            name("date_cell", None, "S!$A$1"),
            name("NowIsh", None, "NOW()"),
            name("Plain", None, "S!$Z$9"),
        ];
        assert_eq!(
            tainted_with(&cells, &names),
            ["S!A1", "S!B1", "S!B2"],
            "a name inside a string or followed by ( is not a use"
        );
    }

    #[test]
    fn sheet_scoped_name_shadows_global() {
        let cells = vec![
            cell("S", "A1", "RAND()"),
            cell("S", "B1", "N"),
            cell("T", "B1", "N"),
        ];
        let names = [name("N", None, "S!$A$1"), name("N", Some("T"), "S!$Z$9")];
        assert_eq!(tainted_with(&cells, &names), ["S!A1", "S!B1"]);
    }

    #[test]
    fn names_referring_to_each_other_terminate() {
        let cells = vec![cell("S", "A1", "X"), cell("S", "A2", "Y")];
        let names = [name("X", None, "Y+1"), name("Y", None, "X+RAND()")];
        assert_eq!(tainted_with(&cells, &names), ["S!A1", "S!A2"]);
    }

    #[test]
    fn invariant_cells_are_scored_and_stop_the_taint() {
        let cells = vec![
            cell("S", "A1", "RANDBETWEEN(5,5)"),
            cell("S", "A2", "A1*2"),
            cell("S", "B1", "RAND()"),
            cell("S", "B2", "ISNUMBER(B1)"),
            cell("S", "B3", "B2+B1"),
            cell("S", "B4", "B2+1"),
            cell("S", "C1", "ISNUMBER(NOW())"),
            cell("S", "C2", "NOW()"),
        ];
        let invariant: HashSet<(String, Addr)> = ["A1", "B2", "C1"]
            .iter()
            .map(|a1| {
                (
                    "s".to_string(),
                    Addr::parse(a1).expect("valid test address"),
                )
            })
            .collect();
        let set = volatile_cells(&cells, &[], false, &invariant);
        let mut keys: Vec<String> = set.iter().map(|(s, a)| format!("{s}!{a}")).collect();
        keys.sort();
        assert_eq!(
            keys,
            ["S!B1", "S!B3", "S!C2"],
            "a listed cell is scored; its dependents are tainted only through unlisted cells"
        );
    }
}
