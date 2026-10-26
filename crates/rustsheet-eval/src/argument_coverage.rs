//! Argument coverage (T-0077): per in-scope function and catalog parameter, which classes of
//! input the Excel-evidence cells pass it. Literal arguments are classified as written and
//! single-cell references by the referenced cell's value; nested calls and operators are not
//! classified. The expected classes of a parameter follow from its catalog mode by rule, so the
//! coverage share is indicative (SPEC-0022, `arguments` block).

use std::collections::{BTreeMap, BTreeSet};

use rustsheet_engine::{call_arguments, ArgumentShape};
use serde::{Deserialize, Serialize};

use crate::catalog::{Catalog, CatalogParam};
use crate::cellref::Addr;
use crate::engine::Evaluated;
use crate::runner::{CaseResult, CellResult};
use crate::v15_checkpoint::share;
use crate::value::Value;

/// The class of one argument.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArgClass {
    /// A positive whole number.
    Number,
    /// Zero.
    Zero,
    /// A negative number.
    Negative,
    /// A positive number with a fractional part.
    Fraction,
    /// Text that reads as a number (`"12"`, `" -1.5e3 "`).
    NumericText,
    /// Other non-empty text.
    Text,
    /// `""`.
    EmptyText,
    /// `TRUE` or `FALSE`.
    Boolean,
    /// An error value.
    Error,
    /// A reference to an empty cell, or an argument left out (`ROUND(1,)`).
    Blank,
    /// A multi-cell range, a 3-D reference or an array constant; for a `range` parameter, any
    /// reference.
    Range,
}

/// Every class, in report order.
pub const ALL_CLASSES: [ArgClass; 11] = [
    ArgClass::Number,
    ArgClass::Zero,
    ArgClass::Negative,
    ArgClass::Fraction,
    ArgClass::NumericText,
    ArgClass::Text,
    ArgClass::EmptyText,
    ArgClass::Boolean,
    ArgClass::Error,
    ArgClass::Blank,
    ArgClass::Range,
];

/// One argument class a case's Excel-evidence cells pass a function at one position of a call
/// with `arity` arguments; the report maps the position to a catalog parameter.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ArgumentSeen {
    /// The en-US function name.
    pub function: String,
    /// How many arguments the call has.
    pub arity: usize,
    /// The 0-based argument position.
    pub index: usize,
    /// The argument's class, read as for a parameter of any mode but `range`.
    pub class: ArgClass,
    /// Whether the argument is a reference (a `range` parameter counts it as [`ArgClass::Range`]).
    pub reference: bool,
}

/// The class of a value: a referenced cell's, or a literal's.
#[must_use]
pub fn classify_value(value: &Value) -> ArgClass {
    match value {
        Value::Number(n) if *n == 0.0 => ArgClass::Zero,
        Value::Number(n) if *n < 0.0 => ArgClass::Negative,
        Value::Number(n) if n.fract() != 0.0 => ArgClass::Fraction,
        Value::Number(_) => ArgClass::Number,
        Value::Text(s) if s.is_empty() => ArgClass::EmptyText,
        Value::Text(s) if reads_as_number(s) => ArgClass::NumericText,
        Value::Text(_) => ArgClass::Text,
        Value::Bool(_) => ArgClass::Boolean,
        Value::Error(_) => ArgClass::Error,
        Value::Empty => ArgClass::Blank,
    }
}

/// Whether text reads as a plain decimal number. A rule of thumb for the census: Excel also
/// reads dates, times, currency and percentages, which count as text here.
fn reads_as_number(text: &str) -> bool {
    let t = text.trim();
    t.starts_with(|c: char| c.is_ascii_digit() || matches!(c, '.' | '+' | '-'))
        && t.chars()
            .all(|c| c.is_ascii_digit() || matches!(c, '.' | '+' | '-' | 'e' | 'E'))
        && t.parse::<f64>().is_ok_and(f64::is_finite)
}

/// The arguments the Excel-evidence formula cells among `cells` pass, classified against the
/// evaluated workbook (`sheets` in tab order). Deduplicated per case.
#[must_use]
pub fn observe(
    evaluated: &dyn Evaluated,
    sheets: &[String],
    cells: &[CellResult],
) -> Vec<ArgumentSeen> {
    let mut seen = BTreeSet::new();
    for c in cells.iter().filter(|c| c.from_excel) {
        let (Some(formula), Ok(addr)) = (c.formula.as_deref(), Addr::parse(&c.cell).ok_or(()))
        else {
            continue;
        };
        let (Ok(row), Ok(column)) = (i32::try_from(addr.row), i32::try_from(addr.col)) else {
            continue;
        };
        for call in call_arguments(formula, sheets, &c.sheet, row, column) {
            let arity = call.args.len();
            for (index, arg) in call.args.iter().enumerate() {
                let (class, reference) = match arg {
                    ArgumentShape::Number(n) => (classify_value(&Value::Number(*n)), false),
                    ArgumentShape::Text(s) => (classify_value(&Value::Text(s.clone())), false),
                    ArgumentShape::Bool(_) => (ArgClass::Boolean, false),
                    ArgumentShape::Error => (ArgClass::Error, false),
                    ArgumentShape::Omitted => (ArgClass::Blank, false),
                    ArgumentShape::Area => (ArgClass::Range, true),
                    ArgumentShape::Cell { sheet, row, column } => {
                        let addr = u32::try_from(*row)
                            .ok()
                            .zip(u32::try_from(*column).ok())
                            .map(|(row, col)| Addr { row, col });
                        let target = sheet.as_deref().unwrap_or(&c.sheet);
                        match addr.and_then(|a| evaluated.value(target, a)) {
                            Some(v) => (classify_value(&v), true),
                            None => continue,
                        }
                    }
                    _ => continue,
                };
                seen.insert(ArgumentSeen {
                    function: call.name.clone(),
                    arity,
                    index,
                    class,
                    reference,
                });
            }
        }
    }
    seen.into_iter().collect()
}

/// The catalog parameter that argument `index` of a call with `arity` arguments fills: the
/// parameters before a repeating group in order, then the group in turn, with the parameters
/// after it (LET's calculation, SWITCH's default) taking the last arguments when the group's
/// arguments come out whole. `None` past the last parameter.
#[must_use]
pub fn param_of(params: &[CatalogParam], arity: usize, index: usize) -> Option<usize> {
    let Some(start) = params.iter().position(|p| p.repeat.is_some()) else {
        return (index < params.len()).then_some(index);
    };
    if index < start {
        return Some(index);
    }
    let group = params[start].repeat.unwrap_or(1).max(1);
    let tail = params.len().saturating_sub(start + group);
    let middle = arity.checked_sub(start + tail);
    if let Some(middle) = middle.filter(|m| tail > 0 && m % group == 0) {
        if index >= start + middle {
            return Some(start + group + (index - start - middle)).filter(|&p| p < params.len());
        }
    }
    Some(start + (index - start) % group)
}

/// The classes a parameter of `mode` is expected to see, by rule: a `range` parameter takes only
/// references; `lambda` and `name` parameters take no value and are not counted; every other mode
/// takes any value.
#[must_use]
pub fn expected_classes(mode: &str) -> &'static [ArgClass] {
    match mode {
        "range" => &[ArgClass::Range],
        "lambda" | "name" => &[],
        _ => &ALL_CLASSES,
    }
}

/// One catalog parameter's coverage.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ParamCoverage {
    /// The catalog's parameter name.
    pub name: String,
    /// The catalog's mode (`value`, `range`, ...).
    pub mode: String,
    /// The classes seen, in [`ALL_CLASSES`] order.
    pub seen: Vec<ArgClass>,
    /// How many classes the mode expects ([`expected_classes`]).
    pub expected: usize,
    /// How many of those were seen.
    pub covered: usize,
}

/// One in-scope function's coverage.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FunctionArguments {
    /// Upper-case Excel name.
    pub name: String,
    /// One row per catalog parameter.
    pub params: Vec<ParamCoverage>,
    /// Covered over expected classes, summed over the parameters; `None` when no parameter
    /// expects any (PI, LAMBDA's names).
    pub single: Option<f64>,
}

/// One row per in-scope catalog function, by name, from the cases' [`CaseResult::arguments`].
#[must_use]
pub fn summarize(catalog: &Catalog, results: &[CaseResult]) -> Vec<FunctionArguments> {
    let mut seen: BTreeMap<(&str, usize), BTreeSet<ArgClass>> = BTreeMap::new();
    for a in results.iter().flat_map(|r| &r.arguments) {
        let Some(f) = catalog.function(&a.function) else {
            continue;
        };
        let Some(p) = param_of(&f.params, a.arity, a.index) else {
            continue;
        };
        let class = if a.reference && f.params[p].mode == "range" {
            ArgClass::Range
        } else {
            a.class
        };
        seen.entry((f.name.as_str(), p)).or_default().insert(class);
    }
    catalog
        .functions()
        .filter(|f| catalog.in_scope(&f.name))
        .map(|f| {
            let params: Vec<ParamCoverage> = f
                .params
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let classes = seen.get(&(f.name.as_str(), i));
                    let expected = expected_classes(&p.mode);
                    let has = |c: &ArgClass| classes.is_some_and(|s| s.contains(c));
                    ParamCoverage {
                        name: p.name.clone(),
                        mode: p.mode.clone(),
                        seen: ALL_CLASSES.iter().copied().filter(|c| has(c)).collect(),
                        expected: expected.len(),
                        covered: expected.iter().filter(|c| has(c)).count(),
                    }
                })
                .collect();
            let expected: usize = params.iter().map(|p| p.expected).sum();
            let covered: usize = params.iter().map(|p| p.covered).sum();
            let single = share(expected, expected - covered);
            FunctionArguments {
                name: f.name.clone(),
                params,
                single,
            }
        })
        .collect()
}

/// The mean of the functions' single-argument coverage, over those with a share.
#[must_use]
pub fn average_single(rows: &[FunctionArguments]) -> Option<f64> {
    let shares: Vec<f64> = rows.iter().filter_map(|r| r.single).collect();
    // A few hundred functions, so the cast is exact.
    #[allow(clippy::cast_precision_loss)]
    (!shares.is_empty()).then(|| shares.iter().sum::<f64>() / shares.len() as f64)
}

#[cfg(test)]
mod tests;
