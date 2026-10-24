//! Where the cells and the dynamic spills of the workbook are, for the evaluation driver
//! (`evaluation.rs`): every position in natural order, the dynamic anchors, and snapshots of
//! their spills that an abandoned pass is rolled back to.

use std::collections::HashSet;

use crate::expressions::types::CellReferenceIndex;
use crate::model::Model;
use crate::types::{ArrayKind, Cell};

impl Model {
    /// Every position that holds a cell, in natural `(sheet, row, column)` order.
    pub(crate) fn all_positions(&self) -> Vec<CellReferenceIndex> {
        self.get_all_cells()
            .into_iter()
            .map(|index| CellReferenceIndex {
                sheet: index.index,
                row: index.row,
                column: index.column,
            })
            .collect()
    }

    /// Every dynamic anchor and every spill cell of a dynamic anchor, as they
    /// are now, in natural order, which is the order the sheets give their
    /// cells in.
    pub(crate) fn dynamic_spills(&self) -> Vec<(CellReferenceIndex, Cell)> {
        let mut found = Vec::new();
        for (sheet, worksheet) in self.workbook.worksheets.iter().enumerate() {
            for (row, column, cell) in worksheet.sheet_data.cells() {
                if !matches!(cell, Cell::ArrayFormula { .. } | Cell::SpillCell { .. }) {
                    continue;
                }
                let position = CellReferenceIndex {
                    sheet: sheet as u32,
                    row,
                    column,
                };
                if self.belongs_to_a_dynamic_array(position, cell) {
                    found.push((position, cell.clone()));
                }
            }
        }
        found
    }

    /// Puts the dynamic anchors and their spill cells back to `snapshot`,
    /// removing the spill cells written since.
    pub(crate) fn restore_dynamic_spills(&mut self, snapshot: &[(CellReferenceIndex, Cell)]) {
        let written_since: Vec<CellReferenceIndex> = self
            .dynamic_spills()
            .into_iter()
            .filter(|(_, cell)| matches!(cell, Cell::SpillCell { .. }))
            .map(|(position, _)| position)
            .collect();
        for position in written_since {
            if let Ok(worksheet) = self.workbook.worksheet_mut(position.sheet) {
                let _ = worksheet.cell_clear_contents(position.row, position.column);
            }
        }
        for (position, cell) in snapshot {
            if let Ok(worksheet) = self.workbook.worksheet_mut(position.sheet) {
                let _ = worksheet.update_cell(position.row, position.column, cell.clone());
            }
        }
    }

    pub(crate) fn belongs_to_a_dynamic_array(
        &self,
        position: CellReferenceIndex,
        cell: &Cell,
    ) -> bool {
        match cell {
            Cell::ArrayFormula {
                kind: ArrayKind::Dynamic,
                ..
            } => true,
            Cell::SpillCell { a, .. } => matches!(
                self.fetch_cell(CellReferenceIndex {
                    sheet: position.sheet,
                    row: a.0,
                    column: a.1,
                }),
                Some(Cell::ArrayFormula {
                    kind: ArrayKind::Dynamic,
                    ..
                })
            ),
            _ => false,
        }
    }

    /// Brings `anchor_order` in line with the dynamic anchors that exist now:
    /// anchors that are gone are dropped, new ones are appended in natural
    /// order, the rest keep their relative order.
    pub(crate) fn sync_anchor_order(&mut self) {
        let current = self.dynamic_anchors_in_natural_order();
        let current_set: HashSet<CellReferenceIndex> = current.iter().copied().collect();
        let order = &mut self.evaluation.anchor_order;
        order.retain(|anchor| current_set.contains(anchor));
        let known: HashSet<CellReferenceIndex> = order.iter().copied().collect();
        order.extend(current.into_iter().filter(|a| !known.contains(a)));
    }

    pub(crate) fn dynamic_anchors_in_natural_order(&self) -> Vec<CellReferenceIndex> {
        self.positions_where(|cell| {
            matches!(
                cell,
                Cell::ArrayFormula {
                    kind: ArrayKind::Dynamic,
                    ..
                }
            )
        })
    }

    /// The positions of the cells that `keep` accepts, in natural `(sheet, row, column)` order.
    pub(crate) fn positions_where(&self, keep: impl Fn(&Cell) -> bool) -> Vec<CellReferenceIndex> {
        let mut found = Vec::new();
        for (sheet, worksheet) in self.workbook.worksheets.iter().enumerate() {
            for (row, column, cell) in worksheet.sheet_data.cells() {
                if keep(cell) {
                    found.push(CellReferenceIndex {
                        sheet: sheet as u32,
                        row,
                        column,
                    });
                }
            }
        }
        found
    }
}
