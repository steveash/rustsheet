//! References into linked workbooks, read from what Excel cached for them
//! ([`Workbook::external_books`](crate::types::Workbook)). The other workbook is never opened
//! (directive 5); what the cache does not hold reads `#BLOCKED!`, as a reference Excel holds no
//! cached data for does (docs/excel-semantics/references.md, "External reference with no cached
//! data is `#BLOCKED!`").

use crate::{
    calc_result::CalcResult,
    cast::calc_result_to_array_node,
    cell::spill_value_to_calc_result,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    external_reference::{external_area, ExternalArea},
    limits::check_array_size,
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

/// A sheet of a linked workbook that the externalLink cache holds: the index of its book in
/// [`Workbook::external_books`](crate::types::Workbook) and its own index in that book.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CachedSheet {
    book: usize,
    sheet: u32,
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

/// The index in [`Workbook::external_books`](crate::types::Workbook) of `[book]`: `[1]` is
/// the first `externalReference` of the workbook part. A book written by file name
/// (`[other.xlsx]`) is not looked up.
fn external_book_index(book: &str) -> Option<usize> {
    book.parse::<usize>().ok()?.checked_sub(1)
}

/// The reference a defined name of `book` refers to (`='Defines'!$B$2`), its sheet an index in
/// `book`'s cache; `None` when it is not a reference to one of those sheets.
fn parse_cached_reference(
    book: &ExternalBook,
    formula: &str,
    model: &Model,
) -> Option<ParsedReference> {
    let formula = formula.strip_prefix('=').unwrap_or(formula);
    ParsedReference::parse_reference_formula(None, formula, model.locale, |sheet| {
        sheet_index(book, sheet)
    })
    .ok()
}

impl Model {
    /// The linked workbook formulas call `[book]`: `[1]` is the first `externalReference` of the
    /// workbook part. A book written by file name (`[other.xlsx]`) is not looked up.
    fn external_book(&self, book: &str) -> Option<&ExternalBook> {
        self.workbook.external_books.get(external_book_index(book)?)
    }

    /// The cache of the sheet `[book]Sheet` names, `None` when the workbook holds none.
    fn cached_sheet(&self, sheet_name: &str) -> Option<CachedSheet> {
        let (book, sheet) = split_book_prefix(sheet_name)?;
        let book_index = external_book_index(book)?;
        let sheet = sheet_index(self.workbook.external_books.get(book_index)?, sheet)?;
        Some(CachedSheet {
            book: book_index,
            sheet,
        })
    }

    /// [`external_area`] with the cache of the sheet it names, if the workbook holds one.
    pub(crate) fn cached_external_area(
        &self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Option<ExternalArea> {
        let mut area = external_area(node, cell)?;
        if let Node::WrongReferenceKind {
            sheet_name: Some(sheet_name),
            ..
        }
        | Node::WrongRangeKind {
            sheet_name: Some(sheet_name),
            ..
        } = node.ungrouped()
        {
            area.cache = self.cached_sheet(sheet_name);
        }
        Some(area)
    }

    /// The cached value of the cell (`row`, `column`) of `sheet`; a cell the cache does not
    /// hold is empty, as is a book the workbook no longer holds.
    fn cached_value(
        &self,
        sheet: CachedSheet,
        row: i32,
        column: i32,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        match self.workbook.external_books.get(sheet.book) {
            Some(book) => self.cached_cell(book, sheet.sheet, row, column, cell),
            None => CalcResult::EmptyCell,
        }
    }

    /// The cells of `area` on `sheet`, row by row; the caller bounds the area's size.
    fn cached_rows(
        &self,
        sheet: CachedSheet,
        area: ExternalArea,
        cell: CellReferenceIndex,
    ) -> Vec<Vec<ArrayNode>> {
        (area.first_row..=area.last_row)
            .map(|row| {
                (area.first_column..=area.last_column)
                    .map(|column| {
                        calc_result_to_array_node(self.cached_value(sheet, row, column, cell))
                    })
                    .collect()
            })
            .collect()
    }

    /// The written area's cells in a value context: what the cache holds for them, one cell a
    /// scalar and several an array (`VLOOKUP(5,[1]Uses!$A$1:$B$3,2,FALSE)`,
    /// `SUMPRODUCT(--ISTEXT([1]Uses!$A$1:$B$3))`; fixtures/external-link-cached-ranges, T-1460).
    /// An area the cache does not hold is `#BLOCKED!` ([`ExternalArea::blocked_value`]).
    pub(crate) fn external_area_value(
        &self,
        area: ExternalArea,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let Some(sheet) = area.cache else {
            return area.blocked_value(cell);
        };
        let (rows, columns) = area.extent();
        if (rows, columns) == (1, 1) {
            return self.cached_value(sheet, area.first_row, area.first_column, cell);
        }
        if let Some((error, message)) = check_array_size(rows, columns) {
            return CalcResult::new_error(error, cell, message);
        }
        CalcResult::Array(self.cached_rows(sheet, area, cell))
    }

    /// [`ExternalArea::picked_value`] for an area INDEX or an intersection picked: its cached
    /// cells when the cache holds its sheet (`INDEX([1]Uses!$A$1:$B$3,2,2)` is the cached "7";
    /// fixtures/external-link-cached-ranges).
    pub(crate) fn external_picked_value(
        &self,
        area: ExternalArea,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if area.cache.is_some() {
            self.external_area_value(area, cell)
        } else {
            area.picked_value(cell)
        }
    }

    /// The rows and columns of `sheet` its cached cells span, `None` when it holds none.
    fn cached_extent(&self, sheet: CachedSheet) -> Option<ExternalArea> {
        let book = self.workbook.external_books.get(sheet.book)?;
        let mut cells = book
            .cells
            .keys()
            .filter(|(index, ..)| *index == sheet.sheet);
        let &(_, row, column) = cells.next()?;
        let first = ExternalArea {
            first_row: row,
            first_column: column,
            last_row: row,
            last_column: column,
            cache: Some(sheet),
        };
        Some(cells.fold(first, |area, &(_, row, column)| ExternalArea {
            first_row: area.first_row.min(row),
            first_column: area.first_column.min(column),
            last_row: area.last_row.max(row),
            last_column: area.last_column.max(column),
            cache: area.cache,
        }))
    }

    /// The cells of `areas` an aggregate reads, as reference cells: an array that skips text,
    /// booleans and blanks where SUM, COUNT, AVERAGE, MAX and the like skip them in a range
    /// (`SUM([1]Uses!$A$1)` over cached text is 0, `SUM([1]Uses!$A$1:$B$3)` 15), and the
    /// *A functions coerce them as in a range (`AVERAGEA` 3.2;
    /// fixtures/external-link-cached-ranges). Each area is clipped to the cells its cache
    /// holds, as a whole column of this workbook is clipped to the used area, so
    /// `SUM([1]Uses!$B:$B)` reads three rows; several areas are one row of their cells in
    /// formula order. `#BLOCKED!` when the cache does not hold an area's sheet.
    pub(crate) fn external_aggregate_value(
        &self,
        areas: &[ExternalArea],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let mut rows = Vec::new();
        for area in areas {
            let Some(sheet) = area.cache else {
                return CalcResult::new_error(Error::BLOCKED, cell, "External reference".into());
            };
            let Some(clipped) = self
                .cached_extent(sheet)
                .and_then(|extent| area.intersect(extent))
            else {
                continue;
            };
            let (height, width) = clipped.extent();
            if let Some((error, message)) = check_array_size(height, width) {
                return CalcResult::new_error(error, cell, message);
            }
            rows.extend(self.cached_rows(sheet, clipped, cell));
        }
        if areas.len() > 1 {
            rows = vec![rows.into_iter().flatten().collect()];
        }
        CalcResult::Array(rows)
    }

    /// How many cells of `areas` the cache holds a value for, as COUNTA counts the non-empty
    /// cells of a range (`COUNTA([1]Uses!$A$1:$B$3)` is 5; fixtures/external-link-cached-ranges).
    /// `None` when the cache does not hold an area's sheet.
    pub(crate) fn external_non_empty_count(&self, areas: &[ExternalArea]) -> Option<usize> {
        let mut count = 0;
        for area in areas {
            let sheet = area.cache?;
            let book = self.workbook.external_books.get(sheet.book)?;
            count += book
                .cells
                .keys()
                .filter(|&&(index, row, column)| {
                    index == sheet.sheet
                        && (area.first_row..=area.last_row).contains(&row)
                        && (area.first_column..=area.last_column).contains(&column)
                })
                .count();
        }
        Some(count)
    }

    /// The area a workbook-scoped name of a linked workbook, `[1]!Name`, stands for, read from
    /// the cache: `ISREF([1]!NR_Range)` is TRUE, `ROWS` 3 and `SUM` 15 for a name of
    /// `'Uses'!$A$1:$B$3` (fixtures/external-link-cached-ranges). A sheet-scoped name
    /// (`[1]Defines!Name`) of several cells is `#BLOCKED!` there, so only workbook names are
    /// areas; `None` for any other name.
    pub(crate) fn external_name_area(&self, prefixed_name: &str) -> Option<ExternalArea> {
        let (book, rest) = prefixed_name.strip_prefix('[')?.split_once(']')?;
        let name = rest.strip_prefix('!')?;
        let book_index = external_book_index(book)?;
        let (book, defined_name) = self.cached_name(book, None, name)?;
        let (first, last) = match parse_cached_reference(book, &defined_name.formula, self)? {
            ParsedReference::CellReference(target) => (target, target),
            ParsedReference::Range(first, last) => (first, last),
        };
        Some(ExternalArea {
            first_row: first.row.min(last.row),
            first_column: first.column.min(last.column),
            last_row: first.row.max(last.row),
            last_column: first.column.max(last.column),
            cache: Some(CachedSheet {
                book: book_index,
                sheet: first.sheet,
            }),
        })
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
        if !matches!(node.ungrouped(), Node::WrongReferenceKind { .. }) {
            return None;
        }
        let area = self.cached_external_area(node, cell)?;
        let sheet = area.cache?;
        Some(self.cached_value(sheet, area.first_row, area.first_column, cell))
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

    /// The defined name `name` of `book` with the scope `sheet` (a workbook name when `None`).
    fn cached_name(
        &self,
        book: &str,
        sheet: Option<&str>,
        name: &str,
    ) -> Option<(&ExternalBook, &DefinedName)> {
        let book = self.external_book(book)?;
        let scope = match sheet {
            Some(sheet) => Some(sheet_index(book, sheet)?),
            None => None,
        };
        let name = name.to_uppercase();
        let defined_name = book.defined_names.iter().find(|defined_name| {
            defined_name.sheet_id == scope && defined_name.name.to_uppercase() == name
        })?;
        Some((book, defined_name))
    }

    fn cached_name_value(
        &self,
        book: &str,
        sheet: Option<&str>,
        name: &str,
        cell: CellReferenceIndex,
    ) -> Option<CalcResult> {
        if sheet.is_none() {
            let area = self.external_name_area(&format!("[{book}]!{name}"))?;
            return Some(self.external_area_value(area, cell));
        }
        let (cached_book, defined_name) = self.cached_name(book, sheet, name)?;
        match parse_cached_reference(cached_book, &defined_name.formula, self)? {
            ParsedReference::CellReference(target) => {
                Some(self.cached_cell(cached_book, target.sheet, target.row, target.column, cell))
            }
            ParsedReference::Range(..) => None,
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
