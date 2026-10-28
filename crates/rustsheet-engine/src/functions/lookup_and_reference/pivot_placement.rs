//! Where PIVOTBY puts the column total and subtotal cells of one function over several value
//! columns, and which of them Excel computes although it drops them (T-1700, T-2967). Excel:
//! docs/excel-semantics/arrays.md "GROUPBY and PIVOTBY: column totals over several value columns"
//! and "PIVOTBY: an error column total over several value columns".

use std::collections::HashMap;

use super::grouping::{Group, Grouping, Line};

/// One column of the PIVOTBY grid right of the row keys (and the label column): output value
/// column `value` (`Functions::shape`) of column group `group`, of the subtotal of the column
/// groups `start..end` (which share their first `level` keys), or the total of it.
#[derive(Clone, Copy)]
pub(super) enum Slot {
    Group {
        group: usize,
        value: usize,
    },
    Subtotal {
        level: usize,
        start: usize,
        end: usize,
        value: usize,
    },
    Total {
        value: usize,
    },
}

/// What a grid column shows in every row: the column totals and subtotals of one function over
/// several value columns show other columns' cells or none ([`Placement`], T-1700). `Own` and
/// `Moved` compute alike; the variants record which cells moved.
#[derive(Clone, Copy)]
pub(super) enum Shown {
    /// The cell of its own slot.
    Own(Slot),
    /// The cell of a subtotal or total slot, shown in a subtotal or total column that may be
    /// another's (probes/pivotby-col-subtotals-values S!A1: C's second subtotal column shows the
    /// first Grand Total column's cells).
    Moved(Slot),
    /// `""` in every row.
    Empty,
}

/// Where Excel puts the column total and subtotal cells of one function over several value
/// columns (docs/excel-semantics/arrays.md "GROUPBY and PIVOTBY: column totals over several
/// value columns", T-1700): the column lines laid out with the subtotals after their groups, in
/// the order of the key sort before any value sort, and each group's first source row, which
/// names it across the value sort.
pub(super) struct Placement {
    lines: Vec<Line>,
    first_rows: Vec<usize>,
}

impl Placement {
    pub(super) fn new(cols: &Grouping, levels: usize) -> Self {
        Placement {
            lines: cols.lines(levels, false),
            first_rows: cols.groups.iter().map(first_row).collect(),
        }
    }

    /// What each of `slots` (the columns as laid out after the sorts) shows, with `width` value
    /// columns per line: what [`Placement::written_cells`] wrote into the subtotal or total
    /// column, which follows its line into the layout (a negative depth, a value sort), `""`
    /// where nothing was. Without subtotals the total's writes land in the group columns unless
    /// there is only one group, so the Total columns are `""` (P!Y97) or show the last value's
    /// total first (probes/pivotby-total-columns-one-group S!A1). Evidence:
    /// probes/pivotby-col-subtotals-values S!A1-A111, -values2, -values3; probes/groupby-subtotals
    /// G!A781, G!A1141, G!A1171.
    pub(super) fn shown(&self, cols: &Grouping, slots: &[Slot], width: usize) -> Vec<Shown> {
        let laid_out = self.laid_out(cols);
        let cells = self.written_cells(&laid_out, width);
        let position: HashMap<Line, usize> = laid_out
            .iter()
            .enumerate()
            .map(|(index, &line)| (line, index))
            .collect();
        let written = |line: Option<usize>, value: usize| {
            line.and_then(|line| cells.get(line.saturating_mul(width).saturating_add(value)))
                .copied()
                .flatten()
                .map_or(Shown::Empty, Shown::Moved)
        };
        slots
            .iter()
            .map(|&slot| match slot {
                Slot::Group { .. } => Shown::Own(slot),
                Slot::Subtotal {
                    level,
                    start,
                    end,
                    value,
                } => written(
                    position.get(&Line::Subtotal { level, start, end }).copied(),
                    value,
                ),
                Slot::Total { value } => written(Some(self.lines.len()), value),
            })
            .collect()
    }

    /// Each placement line as laid out after the sorts (`cols` in their final order): a value
    /// sort keeps a parent's groups together, so a subtotal covers as many groups from its first.
    fn laid_out(&self, cols: &Grouping) -> Vec<Line> {
        let now: HashMap<usize, usize> = cols
            .groups
            .iter()
            .enumerate()
            .map(|(index, group)| (first_row(group), index))
            .collect();
        let sorted = |group: usize| {
            self.first_rows
                .get(group)
                .and_then(|row| now.get(row))
                .copied()
                .unwrap_or(group)
        };
        self.lines
            .iter()
            .map(|&line| match line {
                Line::Group(group) => Line::Group(sorted(group)),
                Line::Subtotal { level, start, end } => {
                    let first = (start..end).map(sorted).min().unwrap_or(start);
                    Line::Subtotal {
                        level,
                        start: first,
                        end: first.saturating_add(end.saturating_sub(start)),
                    }
                }
            })
            .collect()
    }

    /// The slot written into each placement column (`width` per line, then the total's): the
    /// total and then each subtotal (by its first group, a parent before its own deeper
    /// subtotals) write their value v into column `f * width + k + v`, where `f` is the first line
    /// they cover and `k` the number of lines they cover (the total: all of them), Excel counting
    /// the covered lines, not their columns. A write over a group column is dropped, and a later
    /// write wins. The slots written name the lines as `laid_out`.
    fn written_cells(&self, laid_out: &[Line], width: usize) -> Vec<Option<Slot>> {
        let mut cells: Vec<Option<Slot>> = vec![None; (self.lines.len() + 1).saturating_mul(width)];
        for (column, slot) in self.writes(laid_out, width) {
            if matches!(self.lines.get(column / width), Some(Line::Group(_))) {
                continue;
            }
            if let Some(cell) = cells.get_mut(column) {
                *cell = Some(slot);
            }
        }
        cells
    }

    /// The dropped writes of [`Placement::written_cells`], each gated by the group column it
    /// fell on: Excel still computes such a cell, in the rows with a record in that group, and
    /// an error there makes PIVOTBY `#VALUE!` (`Grid::checked_totals`, pivotby.rs). Without subtotals
    /// these are the Total cells of value `v` gated by the line `(lines + v) / width` (T-2578);
    /// with subtotals, also the subtotal cells, and a Total cell that lands on a subtotal column
    /// is shown, not checked (probes/t2967-pivotby-subtotal-gate; t2606-pivotby-total-gate-layouts
    /// P!A1-B14). Excel: docs/excel-semantics/arrays.md "PIVOTBY: an error column total over
    /// several value columns".
    pub(super) fn checked_totals(&self, cols: &Grouping, width: usize) -> Vec<CheckedTotal> {
        let laid_out = self.laid_out(cols);
        self.writes(&laid_out, width)
            .filter_map(|(column, slot)| {
                let index = column / width.max(1);
                let Some(Line::Group(_)) = self.lines.get(index) else {
                    return None;
                };
                let gate = *laid_out.get(index)?;
                Some(CheckedTotal { slot, gate })
            })
            .collect()
    }

    /// Each write of the total and the subtotals, in Excel's order: the placement column
    /// `f * width + k + v` ([`Placement::written_cells`]) and the slot written there.
    fn writes<'s>(
        &'s self,
        laid_out: &'s [Line],
        width: usize,
    ) -> impl Iterator<Item = (usize, Slot)> + 's {
        let mut group_line = vec![0; self.first_rows.len()];
        let mut subtotals = Vec::new();
        for (index, line) in self.lines.iter().enumerate() {
            match *line {
                Line::Group(group) => {
                    if let Some(entry) = group_line.get_mut(group) {
                        *entry = index;
                    }
                }
                Line::Subtotal { level, start, .. } => subtotals.push((start, level, index)),
            }
        }
        subtotals.sort_unstable();
        let lines = self.lines.len();
        // (first line covered, lines covered, the line written, `None` for the total).
        let writers = std::iter::once((0, lines, None)).chain(subtotals.into_iter().map(
            move |(start, _, index)| {
                let first = group_line.get(start).copied().unwrap_or(index);
                (
                    first,
                    index.saturating_sub(first),
                    laid_out.get(index).copied(),
                )
            },
        ));
        writers.flat_map(move |(first, covered, line)| {
            (0..width).map(move |value| {
                let column = first
                    .saturating_mul(width)
                    .saturating_add(covered)
                    .saturating_add(value);
                let slot = match line {
                    Some(Line::Subtotal { level, start, end }) => Slot::Subtotal {
                        level,
                        start,
                        end,
                        value,
                    },
                    _ => Slot::Total { value },
                };
                (column, slot)
            })
        })
    }
}

/// A group's first source row, which no other group has.
fn first_row(group: &Group) -> usize {
    group.rows.first().copied().unwrap_or(usize::MAX)
}

/// A Total or subtotal column slot whose cell `Model::check_totals` (pivotby.rs) computes, and the column
/// group line `gate` under which a line needs a record for it to be computed
/// ([`Placement::checked_totals`]).
#[derive(Clone, Copy)]
pub(super) struct CheckedTotal {
    pub(super) slot: Slot,
    pub(super) gate: Line,
}
