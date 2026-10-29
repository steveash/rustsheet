//! [`Txn::log_typed_fill`]: the cells and the table a typed formula changes beside its own cell,
//! as a calculated column (T-2958, T-3024) or by growing its table (T-2968).

use super::diff::Key;
use super::{Txn, Undo};
use crate::error::{EditError, RefError};
use crate::refs::CellRef;

impl Txn<'_> {
    /// Logs what typing `text` into `cell` (engine `key`, on sheet index `sheet`) writes besides
    /// the cell: the other rows of its table column that it fills as a calculated column, and the
    /// table as it was, which the fill marks as a calculated column or the formula grows by a
    /// column when typed just right of it. They are part of this edit and its undo.
    pub(super) fn log_typed_fill(
        &mut self,
        sheet: u32,
        key: Key,
        cell: CellRef,
        text: &str,
    ) -> Result<(), EditError> {
        if !text.starts_with('=') {
            return Ok(());
        }
        let (_, row, col) = key;
        // Typed just right of a table, it grows the table by a column first (T-2968): the
        // table, the new column's header and the rows it fills are this edit's too.
        let expansion =
            (self.model.table_expansion(sheet, row, col, text)).map_err(EditError::Engine)?;
        let targets = match expansion {
            Some(expansion) => {
                self.sheet_op = true;
                self.undo.push(Undo::TableBefore(Box::new(expansion.table)));
                expansion.cells
            }
            None => {
                match (self.model.calculated_column(sheet, row, col)).map_err(EditError::Engine)? {
                    Some(fill) => {
                        self.undo.push(Undo::TableBefore(Box::new(fill.table)));
                        fill.targets
                    }
                    None => Vec::new(),
                }
            }
        };
        // The targets are the other rows of the cell's column.
        for (r, c) in targets {
            let target = CellRef {
                row: u32::try_from(r - 1).map_err(|_| RefError::OutOfBounds)?,
                ..cell
            };
            self.log_cell(sheet, (key.0, r, c), target)?;
        }
        Ok(())
    }
}
