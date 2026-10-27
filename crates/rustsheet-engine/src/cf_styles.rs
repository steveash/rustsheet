//! Reading a cell's style with its conditional-formatting overlay. The conditional formats are
//! evaluated on the first read after a load, an `evaluate()` or `set_theme` (SPEC-0021, T-0468),
//! not by those, and only for the cell read (T-0476): the rules whose areas hold it, in priority
//! order, as the evaluation of every cell would apply them to it. A rule's statistics over its
//! area (the numbers for color scales, data bars, icons, top/bottom and average rules, the value
//! counts for unique and duplicate rules) are computed once per round, over the area cut at the
//! sheet's used area. So a read costs the rules over that cell plus, once, the statistics over
//! the used area: an expression rule over a million rows costs one formula per read, not a
//! million (corpus-bench 41978).

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::cf_types::{CfCellResult, CfDataBar, CfIcon, CfRating, CfRule, ExtendedStyle};
use crate::evaluation::CellKey;
use crate::Model;

/// The areas of a rule on a sheet: (`sheet`, its (row1, column1, row2, column2) areas).
type AreaKey = (u32, Vec<(i32, i32, i32, i32)>);

/// A sheet's rules in priority order, each with its parsed areas.
type SheetRules = Arc<Vec<(Vec<(i32, i32, i32, i32)>, CfRule)>>;

/// Values (numbers, or distinct values counted) the statistics of one round may keep, across
/// all rules (directive 5): past it a rule's statistics are computed for each read that needs
/// them, as before T-0476.
const MAX_KEPT_VALUES: usize = 4_000_000;

/// What the conditional formats evaluated so far hold, for one round: from the first style read
/// after a load, an `evaluate()` or `set_theme` (which set `stale`) to the next such event.
pub(crate) struct CfState {
    /// The conditional results of each cell evaluated, in rule order.
    pub(crate) cache: HashMap<CellKey, Vec<CfCellResult>>,
    /// Set where the conditional formats used to be evaluated eagerly; the next read starts a
    /// new round.
    pub(crate) stale: bool,
    /// The round evaluated every cell (`evaluate_conditional_formatting`).
    pub(crate) all: bool,
    /// The cells whose rules this round evaluated.
    read: HashSet<CellKey>,
    /// While one cell's rules are evaluated, that cell.
    target: Option<CellKey>,
    /// The numbers of each rule area this round (`collect_numeric_values`), keyed by the area
    /// cut at the used area.
    pub(crate) values: HashMap<AreaKey, Arc<[f64]>>,
    /// The value counts of each rule area this round (`value_counts`), keyed the same way.
    pub(crate) counts: HashMap<AreaKey, Arc<HashMap<String, u32>>>,
    /// How many values `values` and `counts` hold, against `MAX_KEPT_VALUES`.
    kept: usize,
    /// Each sheet's rules in priority order this round.
    rules: HashMap<u32, SheetRules>,
    /// How many rounds began, for the tests of SPEC-0021.
    #[cfg(test)]
    pub(crate) runs: usize,
}

impl Default for CfState {
    /// Stale: nothing is evaluated before the first read.
    fn default() -> Self {
        CfState {
            cache: HashMap::new(),
            stale: true,
            all: false,
            read: HashSet::new(),
            target: None,
            values: HashMap::new(),
            counts: HashMap::new(),
            kept: 0,
            rules: HashMap::new(),
            #[cfg(test)]
            runs: 0,
        }
    }
}

/// Whether the area (row1, column1, row2, column2) holds the cell (`row`, `column`).
fn holds((r1, c1, r2, c2): (i32, i32, i32, i32), row: i32, column: i32) -> bool {
    (r1..=r2).contains(&row) && (c1..=c2).contains(&column)
}

impl CfState {
    /// Forgets the last round's results and statistics.
    pub(crate) fn begin_round(&mut self) {
        self.cache.clear();
        self.read.clear();
        self.values.clear();
        self.counts.clear();
        self.kept = 0;
        self.rules.clear();
        self.stale = false;
        self.all = false;
        #[cfg(test)]
        {
            self.runs += 1;
        }
    }

    /// Whether statistics of `values` more values may be kept this round; charges them if so.
    pub(crate) fn charge(&mut self, values: usize) -> bool {
        let kept = self.kept.saturating_add(values);
        if kept > MAX_KEPT_VALUES {
            return false;
        }
        self.kept = kept;
        true
    }

    /// How many values the statistics of this round keep, for the tests of T-0476.
    #[cfg(test)]
    pub(crate) fn kept(&self) -> usize {
        self.kept
    }

    /// The areas of `ranges` a rule's results are written for: all of them, or while one cell is
    /// evaluated, that cell once for each area that holds it.
    pub(crate) fn visited(&self, ranges: &[(i32, i32, i32, i32)]) -> Vec<(i32, i32, i32, i32)> {
        match self.target {
            None => ranges.to_vec(),
            Some((_, row, column)) => ranges
                .iter()
                .filter(|&&area| holds(area, row, column))
                .map(|_| (row, column, row, column))
                .collect(),
        }
    }
}

impl Model {
    /// Returns the extended cell style for (`sheet`, `row`, `column`), including any
    /// conditional-formatting overlay. The conditional formats are evaluated here, on the first
    /// read after a load, an `evaluate()` or `set_theme`, not by those (SPEC-0021).
    pub fn get_extended_style_for_cell(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
    ) -> Result<ExtendedStyle, String> {
        if self.cf.stale {
            self.cf.begin_round();
        }
        if !self.cf.all && self.cf.read.insert((sheet, row, column)) {
            self.evaluate_conditional_formatting_at(sheet, row, column);
        }
        let base = self.get_style_for_cell(sheet, row, column)?;
        let extended = match self.cf.cache.get(&(sheet, row, column)) {
            Some(c) => c,
            None => {
                return Ok(ExtendedStyle {
                    style: base,
                    icon: None,
                    data_bar: None,
                    rating: None,
                })
            }
        };
        let mut style = base;
        let mut icon = None;
        let mut data_bar = None;
        let mut rating = None;
        // `extended` contains the cached conditional-formatting results for this cell in the
        // order produced by the evaluator. Dxf results accumulate as we iterate; for exclusive
        // types (ColorScale, DataBar, Icon, Rating), the last assignment in this evaluated
        // result order wins.
        for cf_result in extended {
            match cf_result {
                CfCellResult::Dxf(dxf_id) => {
                    if let Some(dxf) = self.workbook.styles.dxfs.get(*dxf_id as usize) {
                        style = dxf.apply_to(&style);
                    }
                }
                CfCellResult::ColorScale(color) => {
                    style.fill.color = color.clone();
                }
                CfCellResult::DataBar {
                    positive_color,
                    negative_color,
                    is_gradient,
                    value,
                    axis_position,
                    show_value,
                } => {
                    data_bar = Some(CfDataBar {
                        positive_color: positive_color.clone(),
                        negative_color: negative_color.clone(),
                        is_gradient: *is_gradient,
                        value: *value,
                        axis_position: *axis_position,
                        show_value: *show_value,
                    });
                }
                CfCellResult::Icon {
                    icon: cf_icon,
                    color,
                    show_value,
                } => {
                    icon = Some(CfIcon {
                        icon: cf_icon.clone(),
                        color: color.clone(),
                        show_value: *show_value,
                    });
                }
                CfCellResult::Rating {
                    icon: cf_icon,
                    count,
                    max,
                    color,
                    show_value,
                } => {
                    rating = Some(CfRating {
                        icon: cf_icon.clone(),
                        count: *count,
                        max: *max,
                        color: color.clone(),
                        show_value: *show_value,
                    });
                }
            }
        }
        Ok(ExtendedStyle {
            style,
            icon,
            data_bar,
            rating,
        })
    }

    /// Evaluates the rules of `sheet` whose areas hold (`row`, `column`), in priority order, for
    /// that cell only: what `evaluate_conditional_formatting` stores for it.
    fn evaluate_conditional_formatting_at(&mut self, sheet: u32, row: i32, column: i32) {
        let Some(rules) = self.sheet_rules(sheet) else {
            return;
        };
        self.cf.target = Some((sheet, row, column));
        for (ranges, rule) in rules.iter() {
            if ranges.iter().any(|&area| holds(area, row, column)) {
                self.apply_cf_rule(sheet, rule, ranges);
            }
        }
        self.cf.target = None;
    }

    /// `sheet`'s rules in priority order with their parsed areas, built once per round.
    fn sheet_rules(&mut self, sheet: u32) -> Option<SheetRules> {
        if let Some(rules) = self.cf.rules.get(&sheet) {
            return Some(rules.clone());
        }
        let worksheet = self.workbook.worksheets.get(sheet as usize)?;
        let mut rules = worksheet.conditional_formatting.clone();
        rules.sort_by_key(|cf| cf.priority);
        let rules: SheetRules = Arc::new(
            rules
                .into_iter()
                .map(|cf| {
                    (
                        crate::conditional_formatting::parse_sqref(&cf.range),
                        cf.cf_rule,
                    )
                })
                .collect(),
        );
        self.cf.rules.insert(sheet, rules.clone());
        Some(rules)
    }
}
