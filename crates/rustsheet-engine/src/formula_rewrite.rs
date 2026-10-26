//! Rewriting the stored text of every formula and defined name for a workbook edit (a table or
//! sheet delete, a sheet move), keeping what changed so the undo can put it back.

use crate::{
    expressions::{
        lexer::LexerMode,
        parser::{stringify::to_english_string, Node},
        types::CellReferenceRC,
    },
    model::Model,
};

impl Model {
    /// Prints every formula of every sheet but `skip_sheet`, and every defined name, with
    /// `print` (given the parsed formula, and the cell it is read on for a defined name, or no
    /// cell for an R1C1 shared formula), stores the text where it changed and returns what it
    /// changed. Does not reparse.
    pub(crate) fn rewrite_formulas(
        &mut self,
        skip_sheet: Option<usize>,
        print: impl Fn(Node, Option<&CellReferenceRC>) -> String,
    ) -> Vec<RewrittenFormula> {
        let mut rewritten = Vec::new();
        self.parser.set_lexer_mode(LexerMode::R1C1);
        for (index, worksheet) in self.workbook.worksheets.iter_mut().enumerate() {
            if Some(index) == skip_sheet {
                continue;
            }
            let context = CellReferenceRC {
                sheet: worksheet.get_name(),
                row: 1,
                column: 1,
            };
            for (formula_index, formula) in worksheet.shared_formulas.iter_mut().enumerate() {
                let node = self.parser.parse(formula, &context);
                let text = print(node, None);
                if text != *formula {
                    rewritten.push(RewrittenFormula {
                        place: FormulaPlace::Shared {
                            sheet_id: worksheet.sheet_id,
                            index: formula_index,
                        },
                        original: std::mem::replace(formula, text.clone()),
                        rewritten: text,
                    });
                }
            }
        }
        self.parser.set_lexer_mode(LexerMode::A1);
        // Defined names have no cell context; any cell of a remaining sheet does.
        let remaining = self
            .workbook
            .worksheets
            .iter()
            .enumerate()
            .find(|(index, _)| Some(*index) != skip_sheet)
            .map(|(_, worksheet)| worksheet.get_name());
        let Some(sheet) = remaining else {
            return rewritten;
        };
        let context = CellReferenceRC {
            sheet,
            row: 1,
            column: 1,
        };
        for (index, defined_name) in self.workbook.defined_names.iter_mut().enumerate() {
            let stored = defined_name.formula.trim_start();
            let (eq, body) = stored.strip_prefix('=').map_or(("", stored), |b| ("=", b));
            let node = self.parser.parse(body, &context);
            let unchanged = to_english_string(&node, &context);
            let text = print(node, Some(&context));
            if text != unchanged {
                let text = format!("{eq}{text}");
                rewritten.push(RewrittenFormula {
                    place: FormulaPlace::DefinedName(index),
                    original: std::mem::replace(&mut defined_name.formula, text.clone()),
                    rewritten: text,
                });
            }
        }
        rewritten
    }

    /// Puts back the formulas [`Model::rewrite_formulas`] rewrote, where they still hold the
    /// rewritten text (the undo of the edit). Does not reparse.
    pub(crate) fn restore_rewritten_formulas(&mut self, rewritten: Vec<RewrittenFormula>) {
        for RewrittenFormula {
            place,
            original,
            rewritten,
        } in rewritten
        {
            let slot = match place {
                FormulaPlace::Shared { sheet_id, index } => self
                    .workbook
                    .worksheets
                    .iter_mut()
                    .find(|worksheet| worksheet.sheet_id == sheet_id)
                    .and_then(|worksheet| worksheet.shared_formulas.get_mut(index)),
                FormulaPlace::DefinedName(index) => self
                    .workbook
                    .defined_names
                    .get_mut(index)
                    .map(|defined_name| &mut defined_name.formula),
            };
            if let Some(slot) = slot.filter(|slot| **slot == rewritten) {
                *slot = original;
            }
        }
    }
}

/// A formula that a workbook edit rewrote, kept so the undo can put it back.
#[derive(Debug, Clone, bitcode::Encode, bitcode::Decode)]
pub(crate) struct RewrittenFormula {
    place: FormulaPlace,
    original: String,
    rewritten: String,
}

/// Where a [`RewrittenFormula`] lives.
#[derive(Debug, Clone, Copy, bitcode::Encode, bitcode::Decode)]
enum FormulaPlace {
    /// A shared formula of the sheet with this id.
    Shared { sheet_id: u32, index: usize },
    /// A defined name, by its index in the workbook's list.
    DefinedName(usize),
}
