//! Argument coverage (T-0077): per in-scope function and catalog parameter, which classes of
//! input the Excel-evidence cells pass it. Literal arguments are classified as written and
//! single-cell references by the referenced cell's value; nested calls and operators are not
//! classified. The expected classes of a parameter follow from its catalog mode by rule, so the
//! coverage share is indicative (SPEC-0022, `arguments` block).

use std::collections::{BTreeMap, BTreeSet};

use rustsheet_engine::{call_arguments, ArgumentShape};
use serde::{Deserialize, Serialize};

use crate::catalog::{Catalog, CatalogFunction, CatalogParam};
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

/// One argument of a call: its class, read as for a parameter of any mode but `range`, and
/// whether it is a reference (a `range` parameter counts any reference as [`ArgClass::Range`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ArgumentSeen {
    /// The argument's class.
    pub class: ArgClass,
    /// Whether the argument is a reference.
    pub reference: bool,
}

/// One call a case's Excel-evidence cells make, with its arguments' classes kept together so
/// that pairs of them can be counted (T-0083); the report maps positions to catalog parameters.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CallSeen {
    /// The en-US function name.
    pub function: String,
    /// One entry per argument position; `None` for an argument that is not classified (a nested
    /// call, an operator).
    pub args: Vec<Option<ArgumentSeen>>,
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

/// Whether text reads as a plain decimal number. A rule of thumb for the census, not the
/// engine's coercion (`parse_number_text` in crates/rustsheet-engine/src/cast.rs): Excel also
/// reads dates, times, currency and percentages, which count as text here.
fn reads_as_number(text: &str) -> bool {
    let t = text.trim();
    t.starts_with(|c: char| c.is_ascii_digit() || matches!(c, '.' | '+' | '-'))
        && t.chars()
            .all(|c| c.is_ascii_digit() || matches!(c, '.' | '+' | '-' | 'e' | 'E'))
        && t.parse::<f64>().is_ok_and(f64::is_finite)
}

/// The calls the Excel-evidence formula cells among `cells` make, their arguments classified
/// against the evaluated workbook (`sheets` in tab order). Deduplicated per case; calls without
/// any classified argument are left out.
#[must_use]
pub fn observe(
    evaluated: &dyn Evaluated,
    sheets: &[String],
    cells: &[CellResult],
) -> Vec<CallSeen> {
    let mut seen = BTreeSet::new();
    for c in cells.iter().filter(|c| c.from_excel) {
        let (Some(formula), Some(addr)) = (c.formula.as_deref(), Addr::parse(&c.cell)) else {
            continue;
        };
        let (Ok(row), Ok(column)) = (i32::try_from(addr.row), i32::try_from(addr.col)) else {
            continue;
        };
        for call in call_arguments(formula, sheets, &c.sheet, row, column) {
            let args: Vec<Option<ArgumentSeen>> = call
                .args
                .iter()
                .map(|arg| {
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
                            (
                                classify_value(&addr.and_then(|a| evaluated.value(target, a))?),
                                true,
                            )
                        }
                        ArgumentShape::Other => return None,
                    };
                    Some(ArgumentSeen { class, reference })
                })
                .collect();
            if args.iter().any(Option::is_some) {
                seen.insert(CallSeen {
                    function: call.name,
                    args,
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
    Some(start + (index - start) % group).filter(|&p| p < params.len())
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
    /// Of the pairs of two counted parameters and an expected class of each, the share one call
    /// passes together (T-0083); `None` with fewer than two counted parameters.
    #[serde(default)]
    pub pairwise: Option<f64>,
}

/// A function name, a parameter and a class one call passes it: one entry of the single-argument
/// census.
type Slot<'a> = (&'a str, usize, ArgClass);

/// The (parameter, class) slots of one call of `f`, in order; a `range` parameter counts a
/// reference as [`ArgClass::Range`].
fn call_slots(f: &CatalogFunction, call: &CallSeen) -> BTreeSet<(usize, ArgClass)> {
    let arity = call.args.len();
    call.args
        .iter()
        .enumerate()
        .filter_map(|(index, arg)| {
            let arg = (*arg)?;
            let p = param_of(&f.params, arity, index)?;
            let class = if arg.reference && f.params[p].mode == "range" {
                ArgClass::Range
            } else {
                arg.class
            };
            Some((p, class))
        })
        .collect()
}

/// One row per in-scope catalog function, by name, from the cases' [`CaseResult::arguments`].
#[must_use]
pub fn summarize(catalog: &Catalog, results: &[CaseResult]) -> Vec<FunctionArguments> {
    let mut singles: BTreeSet<Slot<'_>> = BTreeSet::new();
    // Per function, two expected slots of one call with their parameters in order.
    let mut pairs: BTreeMap<&str, BTreeSet<[(usize, ArgClass); 2]>> = BTreeMap::new();
    for call in results.iter().flat_map(|r| &r.arguments) {
        let Some(f) = catalog.function(&call.function) else {
            continue;
        };
        let name = f.name.as_str();
        let slots = call_slots(f, call);
        singles.extend(slots.iter().map(|&(p, c)| (name, p, c)));
        let expected: Vec<(usize, ArgClass)> = slots
            .into_iter()
            .filter(|(p, c)| expected_classes(&f.params[*p].mode).contains(c))
            .collect();
        for (i, &a) in expected.iter().enumerate() {
            for &b in expected[i + 1..].iter().filter(|b| b.0 != a.0) {
                pairs.entry(name).or_default().insert([a, b]);
            }
        }
    }
    catalog
        .functions()
        .filter(|f| catalog.in_scope(&f.name))
        .map(|f| {
            let name = f.name.as_str();
            let params: Vec<ParamCoverage> = f
                .params
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let expected = expected_classes(&p.mode);
                    let has = |c: &ArgClass| singles.contains(&(name, i, *c));
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
            let pairs_expected: usize = params
                .iter()
                .enumerate()
                .flat_map(|(i, p)| params[i + 1..].iter().map(|q| p.expected * q.expected))
                .sum();
            let pairs_covered = pairs.get(name).map_or(0, BTreeSet::len);
            FunctionArguments {
                name: f.name.clone(),
                params,
                single: share(expected, expected - covered),
                pairwise: share(pairs_expected, pairs_expected - pairs_covered),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests;
