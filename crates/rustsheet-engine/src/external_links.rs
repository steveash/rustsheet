//! References into linked workbooks, read from what Excel cached for them
//! ([`Workbook::external_books`](crate::types::Workbook)). The other workbook is never opened
//! (directive 5); what the cache does not hold reads `#BLOCKED!`, as a reference Excel holds no
//! cached data for does (docs/excel-semantics/references.md, "External reference with no cached
//! data is `#BLOCKED!`").

use crate::{
    calc_result::CalcResult,
    cell::spill_value_to_calc_result,
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    external_reference::external_area,
    types::{DefinedName, SpillValue},
    utils::ParsedReference,
    Model,
};

use bitcode::{Decode, Encode};
use std::collections::HashMap;

/// A workbook this one links to, as far as Excel cached it in `xl/externalLinks/externalLinkN.xml`
/// (`externalBook`). The other workbook is never opened (directive 5): a reference into it reads
/// these cached values (docs/excel-semantics/references.md, "External reference with no cached
/// data is `#BLOCKED!`").
#[derive(Encode, Decode, Debug, PartialEq, Clone, Default)]
pub struct ExternalBook {
    /// Its sheet names in `sheetNames` order; a sheet's index here is its `sheetId` in the cache.
    pub sheet_names: Vec<String>,
    /// Its defined names; `sheet_id` is the index in `sheet_names` of a sheet-scoped name's sheet.
    pub defined_names: Vec<DefinedName>,
    /// The cached cell values, keyed by (sheet index, row, column).
    pub cells: HashMap<(u32, i32, i32), SpillValue>,
}

/// The book index and sheet name of an external sheet prefix, `[1]Sheet` → (`1`, `Sheet`).
fn split_book_prefix(sheet_name: &str) -> Option<(&str, &str)> {
    sheet_name.strip_prefix('[')?.split_once(']')
}

/// The index in `book` of its sheet `name`; sheet names match case-insensitively, as
/// `Model::get_sheet_index_by_name` matches this workbook's.
fn sheet_index(book: &ExternalBook, name: &str) -> Option<u32> {
    let name = name.to_uppercase();
    let position = book
        .sheet_names
        .iter()
        .position(|sheet| sheet.to_uppercase() == name)?;
    u32::try_from(position).ok()
}

impl Model {
    /// The linked workbook formulas call `[book]`: `[1]` is the first `externalReference` of the
    /// workbook part. A book written by file name (`[other.xlsx]`) is not looked up.
    fn external_book(&self, book: &str) -> Option<&ExternalBook> {
        let index = book.parse::<usize>().ok()?.checked_sub(1)?;
        self.workbook.external_books.get(index)
    }

    /// The cached value of the cell (`row`, `column`) of sheet `sheet` of `book`; a cell the cache
    /// does not hold is empty.
    fn cached_cell(
        &self,
        book: &ExternalBook,
        sheet: u32,
        row: i32,
        column: i32,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        match book.cells.get(&(sheet, row, column)) {
            Some(value) => spill_value_to_calc_result(value, cell, self.language),
            None => CalcResult::EmptyCell,
        }
    }

    /// The cached value of a one-cell reference into a linked workbook, `=[1]Uses!$A$1` → "Hello!"
    /// (poi/ref-56737.xlsx Refs!E3, Excel-cached). `None` when `node` is no such reference or the
    /// workbook has no cache for that book or sheet: the caller gives `#BLOCKED!`.
    pub(crate) fn external_reference_value(
        &self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Option<CalcResult> {
        let Node::WrongReferenceKind {
            sheet_name: Some(sheet_name),
            ..
        } = node.ungrouped()
        else {
            return None;
        };
        let area = external_area(node, cell)?;
        let (book, sheet) = split_book_prefix(sheet_name)?;
        let book = self.external_book(book)?;
        let sheet = sheet_index(book, sheet)?;
        Some(self.cached_cell(book, sheet, area.first_row, area.first_column, cell))
    }

    /// `[book]!name` or `[book]Sheet!name`: a defined name of a linked workbook (the parser keeps
    /// the prefix in the `DefinedNameKind` name), read through its cache: `=[1]Defines!NR_To_A1` →
    /// "Test A1" and `=[1]!NR_Global_B2` → 142 (poi/ref-56737.xlsx Refs!G3, I3, Excel-cached).
    /// `[0]` is this workbook: `=[0]!NR_Global_B2` is the name `NR_Global_B2` (poi/56737.xlsx
    /// Uses!D3). A name the cache does not hold, or whose `refersTo` is not one cell, is
    /// `#BLOCKED!` (UNVERIFIED, references.md). `None` for a name with no such prefix.
    pub(crate) fn evaluate_external_name(
        &mut self,
        prefixed_name: &str,
        cell: CellReferenceIndex,
    ) -> Option<CalcResult> {
        let (book, rest) = prefixed_name.strip_prefix('[')?.split_once(']')?;
        let (sheet, name) = rest.split_once('!')?;
        let sheet = (!sheet.is_empty()).then_some(sheet);
        if book == "0" {
            return Some(self.evaluate_own_name(sheet, name, cell));
        }
        Some(
            self.cached_name_value(book, sheet, name, cell)
                .unwrap_or_else(|| {
                    CalcResult::new_error(
                        Error::BLOCKED,
                        cell,
                        format!("No cached value for {prefixed_name}"),
                    )
                }),
        )
    }

    fn cached_name_value(
        &self,
        book: &str,
        sheet: Option<&str>,
        name: &str,
        cell: CellReferenceIndex,
    ) -> Option<CalcResult> {
        let book = self.external_book(book)?;
        let scope = match sheet {
            Some(sheet) => Some(sheet_index(book, sheet)?),
            None => None,
        };
        let name = name.to_uppercase();
        let defined_name = book.defined_names.iter().find(|defined_name| {
            defined_name.sheet_id == scope && defined_name.name.to_uppercase() == name
        })?;
        let formula = defined_name
            .formula
            .strip_prefix('=')
            .unwrap_or(&defined_name.formula);
        match ParsedReference::parse_reference_formula(None, formula, self.locale, |sheet| {
            sheet_index(book, sheet)
        }) {
            Ok(ParsedReference::CellReference(target)) => {
                Some(self.cached_cell(book, target.sheet, target.row, target.column, cell))
            }
            _ => None,
        }
    }

    /// `[0]!name` or `[0]Sheet!name`: the workbook- or `Sheet`-scoped defined name of this
    /// workbook, evaluated as the plain name is (`#NAME?` when there is none).
    fn evaluate_own_name(
        &mut self,
        sheet: Option<&str>,
        name: &str,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let scope = match sheet {
            Some(sheet) => match self.get_sheet_index_by_name(sheet) {
                Some(index) => Some(index),
                None => return Self::wrong_sheet_error(Some(sheet), cell, "Wrong sheet"),
            },
            None => None,
        };
        let node = Node::DefinedNameKind((name.to_string(), scope, String::new()));
        self.evaluate_node_in_context(&node, cell)
    }
}
