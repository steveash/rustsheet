//! [`Workbook`]: the facade over the engine `Model` (open/new/save and the read path).

use std::io::{Cursor, Read};
use std::path::Path;
use std::sync::Arc;

use rustsheet_engine::clock::Clock;
use rustsheet_engine::types::DateSystem;
use rustsheet_engine::{Model, LAST_COLUMN, LAST_ROW};
use rustsheet_xlsx::error::XlsxError;
use rustsheet_xlsx::import::ImportLimits;

use crate::error::{LoadError, RefError, SaveError};
use crate::refs::{
    parse_cell, parse_range, resolve_sheet_by_name, CellRef, RangeRef, SheetId, Target,
};
use crate::value::{lone_text, Array, Value};

/// Options for [`Workbook::new`] and the limits shared by all reads.
#[derive(Clone, Debug)]
pub struct WorkbookOptions {
    /// Most cells a single [`Workbook::values`] call may return (checked against the
    /// rectangle area before anything is allocated).
    pub max_read_cells: u64,
    /// Engine locale id (`"en"` is the engine's en-US locale).
    pub locale: String,
    /// IANA timezone name used by date functions.
    pub timezone: String,
    /// Engine language id for function names and error literals.
    pub language: String,
    /// Date system of a new workbook ([`Workbook::new`]). A loaded workbook keeps the one its
    /// xlsx stores (`workbookPr@date1904`), so this field does not apply to loading.
    pub date_system: DateSystem,
    /// The clock NOW, TODAY and the other readers of the current time use: the system time
    /// (the default) or a fixed instant (SPEC-0008). Applied before the first recalc, also when
    /// loading.
    pub clock: Clock,
    /// The workbook's file name without its extension (`Budget` for `Budget.xlsx`), which
    /// `CELL("address")` of a reference on another sheet
    /// reports as `[Budget.xlsx]`. `None` (the default) is an unnamed workbook, for which that
    /// `CELL("address")` is `#N/IMPL!`: Excel's form for an unsaved workbook is unprobed (T-2042). Applies to loading
    /// too.
    pub name: Option<String>,
}

impl Default for WorkbookOptions {
    fn default() -> Self {
        Self {
            max_read_cells: 1_000_000,
            locale: "en".to_string(),
            timezone: "UTC".to_string(),
            language: "en".to_string(),
            date_system: DateSystem::default(),
            clock: Clock::system(),
            name: None,
        }
    }
}

/// Options for [`Workbook::open`] and [`Workbook::from_xlsx_bytes`].
///
/// The size limits are enforced on the bytes actually inflated, before any part is parsed.
#[derive(Clone, Debug)]
pub struct LoadOptions {
    /// Recalculate once after loading.
    pub recalc: bool,
    /// Most bytes all zip entries may declare in total.
    pub max_uncompressed_bytes: u64,
    /// Most entries the zip directory may list.
    pub max_zip_entries: u32,
    /// Options of the loaded workbook (read limit, locale, timezone, language).
    pub workbook: WorkbookOptions,
}

impl Default for LoadOptions {
    fn default() -> Self {
        let import = ImportLimits::default();
        Self {
            recalc: true,
            max_uncompressed_bytes: import.max_uncompressed_bytes,
            max_zip_entries: import.max_zip_entries,
            workbook: WorkbookOptions::default(),
        }
    }
}

/// Options for saving; reserved for future use.
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct SaveOptions {}

/// A worksheet's identity and position.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SheetMeta {
    /// Stable identifier.
    pub id: SheetId,
    /// Current name.
    pub name: String,
    /// Current 0-based position.
    pub index: u32,
}

/// A cell's value together with its stored input and number format.
#[derive(Clone, Debug, PartialEq)]
pub struct CellInfo {
    /// The computed value.
    pub value: Value,
    /// The formula (with leading `=`), if the cell holds one.
    pub formula: Option<String>,
    /// The number format code, `None` for `General`.
    pub number_format: Option<String>,
}

/// An open workbook. Owns its engine model; reads never mutate it.
pub struct Workbook {
    pub(crate) model: Model,
    max_read_cells: u64,
    /// Whether `edit` diffs formula values into [`crate::ChangeSet::values`].
    pub(crate) track_changes: bool,
}

fn engine_load_error(message: impl std::fmt::Display) -> LoadError {
    LoadError::Format(message.to_string())
}

/// All sheets of a model in workbook order.
pub(crate) fn sheet_metas(model: &Model) -> Vec<SheetMeta> {
    model
        .workbook
        .worksheets
        .iter()
        .enumerate()
        .map(|(i, ws)| SheetMeta {
            id: SheetId::new(ws.get_sheet_id()),
            name: ws.get_name(),
            index: u32::try_from(i).unwrap_or(u32::MAX),
        })
        .collect()
}

/// A sheet by case-insensitive name; `None` selects the first sheet.
pub(crate) fn lookup_sheet(model: &Model, name: Option<&str>) -> Option<SheetId> {
    let sheets = sheet_metas(model);
    let pairs: Vec<(SheetId, &str)> = sheets.iter().map(|s| (s.id, s.name.as_str())).collect();
    resolve_sheet_by_name(&pairs, name)
}

/// The positional index of a sheet id.
pub(crate) fn sheet_index(model: &Model, id: SheetId) -> Result<u32, RefError> {
    model
        .workbook
        .worksheets
        .iter()
        .position(|ws| ws.get_sheet_id() == id.get())
        .and_then(|i| u32::try_from(i).ok())
        .ok_or_else(|| RefError::UnknownSheet(id.get().to_string()))
}

/// A target as a rectangle, resolving A1 text against the model's sheets.
pub(crate) fn resolve_range(model: &Model, target: Target<'_>) -> Result<RangeRef, RefError> {
    match target {
        Target::Cell(c) => RangeRef::new(c.sheet, c.row, c.col, c.row, c.col),
        Target::Range(r) => Ok(r),
        Target::A1(text) => parse_range(text, |name| lookup_sheet(model, name)),
    }
}

/// Engine coordinates are 1-based `i32`; the facade's are 0-based `u32` already bounds-checked.
pub(crate) fn engine_rc(row: u32, col: u32) -> Option<(i32, i32)> {
    Some((
        i32::try_from(row).ok()?.checked_add(1)?,
        i32::try_from(col).ok()?.checked_add(1)?,
    ))
}

impl Workbook {
    /// A new workbook with one empty sheet.
    ///
    /// # Errors
    /// [`LoadError::Options`] when the locale, timezone or language is unknown.
    pub fn new(opts: &WorkbookOptions) -> Result<Self, LoadError> {
        let mut model = Model::new_empty_with_clock(
            opts.name.as_deref().unwrap_or_default(),
            &opts.locale,
            &opts.timezone,
            &opts.language,
            opts.clock,
        )
        .map_err(LoadError::Options)?;
        model.workbook.settings.date_system = opts.date_system;
        Ok(Self::from_model(model, opts.max_read_cells))
    }

    /// The workbook's date system: the one chosen at [`Workbook::new`], or the one a loaded
    /// xlsx stores.
    #[must_use]
    pub fn date_system(&self) -> DateSystem {
        self.model.workbook.settings.date_system
    }

    fn from_model(model: Model, max_read_cells: u64) -> Self {
        Self {
            model,
            max_read_cells,
            track_changes: true,
        }
    }

    /// Opens an xlsx file.
    ///
    /// # Errors
    /// [`LoadError::Io`] if the file cannot be read, otherwise as [`Self::from_xlsx_bytes`].
    pub fn open(path: impl AsRef<Path>, opts: &LoadOptions) -> Result<Self, LoadError> {
        // Bound the read itself (special files report length 0): a larger file cannot be a
        // workbook within `max_uncompressed_bytes` anyway.
        let mut bytes = Vec::new();
        std::fs::File::open(path)?
            .take(opts.max_uncompressed_bytes.saturating_add(1))
            .read_to_end(&mut bytes)?;
        if u64::try_from(bytes.len()).map_or(true, |n| n > opts.max_uncompressed_bytes) {
            return Err(LoadError::Options(
                "file larger than max_uncompressed_bytes".into(),
            ));
        }
        Self::from_xlsx_bytes(&bytes, opts)
    }

    /// Loads an xlsx workbook from memory. Never touches the filesystem.
    ///
    /// # Errors
    /// [`LoadError::Options`] when the zip exceeds the limits or the locale is unknown,
    /// [`LoadError::Format`] when the bytes are not a valid workbook.
    pub fn from_xlsx_bytes(bytes: &[u8], opts: &LoadOptions) -> Result<Self, LoadError> {
        let w = &opts.workbook;
        // The importer enforces the same limits on the parts it reads, so a caller raising
        // them here is not stopped by the importer's defaults.
        let limits = ImportLimits {
            max_uncompressed_bytes: opts.max_uncompressed_bytes,
            max_zip_entries: opts.max_zip_entries,
            ..ImportLimits::default()
        };
        let book = rustsheet_xlsx::import::load_from_xlsx_bytes_with_limits(
            bytes,
            w.name.as_deref().unwrap_or_default(),
            &w.locale,
            &w.timezone,
            &limits,
        )
        .map_err(|e| match e {
            XlsxError::Limit { .. } => LoadError::Options(e.to_string()),
            e => engine_load_error(e),
        })?;
        let model = Model::from_workbook_with_clock(book, &w.language, w.clock)
            .map_err(LoadError::Format)?;
        let mut workbook = Self::from_model(model, w.max_read_cells);
        if opts.recalc {
            workbook.recalculate();
        }
        Ok(workbook)
    }

    /// Recalculates every formula.
    pub fn recalculate(&mut self) {
        self.model.evaluate();
    }

    /// Pins the workbook clock (`NOW`, `TODAY`) to the timestamp in the workbook's `METADATA`
    /// sheet, as Excel-authored test workbooks carry it. Returns whether it pinned: a workbook
    /// without a timestamp, or with one outside Excel's calendar, keeps its clock. Harness only
    /// (SPEC-0008); it does not recalculate.
    #[cfg(feature = "metadata_clock")]
    pub fn pin_clock_from_metadata(&mut self) -> bool {
        rustsheet_xlsx::metadata_clock::pin_clock_from_metadata(&mut self.model)
    }

    /// Inserts `count` blank rows above the 0-based `row` of `sheet`, shifting the cells below
    /// down and rewriting the formulas that refer to them, as Excel's row insert does. Harness
    /// only (decision 0076): it is not a [`crate::Txn`] edit, so it has no undo and reports no
    /// change set; the eval replays a case's `insert_row` sheet op with it. It does not
    /// recalculate.
    ///
    /// # Errors
    /// [`EditError::Ref`] for an unknown sheet, a row past the sheet or a `count` of 0 or more
    /// rows than lie below `row`; [`EditError::Engine`] when
    /// the engine refuses the insert (it would push cells off the sheet).
    #[cfg(feature = "harness_row_insert")]
    pub fn insert_rows(
        &mut self,
        sheet: SheetId,
        row: u32,
        count: u32,
    ) -> Result<(), crate::error::EditError> {
        let index = self.sheet_index(sheet)?;
        // The engine's rows are 1-based.
        let row = row
            .checked_add(1)
            .and_then(|r| i32::try_from(r).ok())
            .filter(|r| *r <= LAST_ROW);
        // At least one row, and no more than the sheet has below `row`.
        let count = i32::try_from(count)
            .ok()
            .filter(|c| *c >= 1 && row.is_some_and(|r| *c <= LAST_ROW - r + 1));
        let (Some(row), Some(count)) = (row, count) else {
            return Err(crate::error::EditError::Ref(RefError::OutOfBounds));
        };
        self.model
            .insert_rows(index, row, count)
            .map_err(crate::error::EditError::Engine)
    }

    /// Serializes to xlsx bytes.
    ///
    /// # Errors
    /// [`SaveError::Engine`] when the engine cannot export the workbook.
    pub fn to_xlsx_bytes(&self, _opts: SaveOptions) -> Result<Vec<u8>, SaveError> {
        rustsheet_xlsx::export::save_xlsx_to_writer(&self.model, Cursor::new(Vec::new()))
            .map(Cursor::into_inner)
            .map_err(|e| SaveError::Engine(e.to_string()))
    }

    /// Writes an xlsx file, replacing an existing one.
    ///
    /// # Errors
    /// [`SaveError::Io`] if the file cannot be written, otherwise as [`Self::to_xlsx_bytes`].
    pub fn save_xlsx(&self, path: impl AsRef<Path>, opts: SaveOptions) -> Result<(), SaveError> {
        let bytes = self.to_xlsx_bytes(opts)?;
        std::fs::write(path, bytes)?;
        Ok(())
    }

    /// All sheets in workbook order.
    #[must_use]
    pub fn sheets(&self) -> Vec<SheetMeta> {
        sheet_metas(&self.model)
    }

    /// Looks a sheet up by name, case-insensitively.
    ///
    /// # Errors
    /// [`RefError::UnknownSheet`] when no sheet has that name.
    pub fn sheet_id(&self, name: &str) -> Result<SheetId, RefError> {
        self.lookup_sheet(Some(name))
            .ok_or_else(|| RefError::UnknownSheet(name.to_string()))
    }

    fn lookup_sheet(&self, name: Option<&str>) -> Option<SheetId> {
        lookup_sheet(&self.model, name)
    }

    /// Parses an A1 cell reference (`Sheet1!A1`, `'My Sheet'!$B$2`); unqualified references
    /// use the first sheet.
    ///
    /// # Errors
    /// As [`parse_cell`].
    pub fn cell_ref(&self, a1: &str) -> Result<CellRef, RefError> {
        parse_cell(a1, |name| self.lookup_sheet(name))
    }

    /// Parses an A1 range reference.
    ///
    /// # Errors
    /// As [`parse_range`].
    pub fn range_ref(&self, a1: &str) -> Result<RangeRef, RefError> {
        parse_range(a1, |name| self.lookup_sheet(name))
    }

    fn resolve_range(&self, target: Target<'_>) -> Result<RangeRef, RefError> {
        resolve_range(&self.model, target)
    }

    /// Resolves `target` to its sheet index and the top-left cell in engine (1-based) coordinates.
    fn top_left<'a>(
        &self,
        target: impl Into<Target<'a>>,
    ) -> Result<(RangeRef, u32, i32, i32), RefError> {
        let range = self.resolve_range(target.into())?;
        let sheet = self.sheet_index(range.sheet)?;
        let (row, col) = engine_rc(range.top, range.left).ok_or(RefError::OutOfBounds)?;
        Ok((range, sheet, row, col))
    }

    fn sheet_index(&self, id: SheetId) -> Result<u32, RefError> {
        sheet_index(&self.model, id)
    }

    fn cell_at(&self, sheet: u32, row: u32, col: u32) -> Result<Value, RefError> {
        let (r, c) = engine_rc(row, col).ok_or(RefError::OutOfBounds)?;
        let ws = self
            .model
            .workbook
            .worksheet(sheet)
            .map_err(|_| RefError::UnknownSheet(sheet.to_string()))?;
        Ok(ws.cell(r, c).map_or(Value::Empty, |cell| {
            Value::from_cell(cell, &self.model.workbook.shared_strings)
        }))
    }

    /// The exact UTF-16 code units of a text value that holds a lone surrogate.
    ///
    /// Excel text is a sequence of UTF-16 units and keeps an unpaired surrogate when a cut
    /// splits a pair, which a Rust `str` cannot hold: [`Value::Text`] is the lossy view (each
    /// lone unit as U+FFFD). This returns `Some` exactly when the value of the cell (the top-left
    /// cell of a range) is text holding a lone surrogate, and `None` for every other value.
    ///
    /// # Errors
    /// [`RefError`] when the target does not resolve.
    pub fn text_utf16<'a>(&self, r: impl Into<Target<'a>>) -> Result<Option<Arc<[u16]>>, RefError> {
        let (range, sheet, _, _) = self.top_left(r)?;
        let (row, col) = engine_rc(range.top, range.left).ok_or(RefError::OutOfBounds)?;
        let ws = self
            .model
            .workbook
            .worksheet(sheet)
            .map_err(|_| RefError::UnknownSheet(sheet.to_string()))?;
        Ok(ws
            .cell(row, col)
            .and_then(lone_text)
            .map(|t| t.units().collect()))
    }

    /// The value of a cell, or of the top-left cell of a range.
    ///
    /// # Errors
    /// [`RefError`] when the target does not resolve.
    pub fn value<'a>(&self, r: impl Into<Target<'a>>) -> Result<Value, RefError> {
        let (range, sheet, _, _) = self.top_left(r)?;
        self.cell_at(sheet, range.top, range.left)
    }

    /// The values of a range as a dense row-major [`Array`].
    ///
    /// # Errors
    /// [`RefError::TooLarge`] when the range area exceeds `max_read_cells` (checked before
    /// allocating), otherwise as [`Self::value`].
    pub fn values<'a>(&self, r: impl Into<Target<'a>>) -> Result<Array, RefError> {
        let range = self.resolve_range(r.into())?;
        let sheet = self.sheet_index(range.sheet)?;
        // Re-validate: `RangeRef` has public fields, so a caller can build an inverted literal.
        let range = RangeRef::new(
            range.sheet,
            range.top,
            range.left,
            range.bottom,
            range.right,
        )?;
        let rows = u64::from(range.bottom - range.top) + 1;
        let cols = u64::from(range.right - range.left) + 1;
        let area = rows.checked_mul(cols).ok_or(RefError::TooLarge)?;
        if area > self.max_read_cells {
            return Err(RefError::TooLarge);
        }
        let mut values = Vec::with_capacity(usize::try_from(area).map_err(|_| RefError::TooLarge)?);
        for row in range.top..=range.bottom {
            for col in range.left..=range.right {
                values.push(self.cell_at(sheet, row, col)?);
            }
        }
        let rows = usize::try_from(rows).map_err(|_| RefError::TooLarge)?;
        let cols = usize::try_from(cols).map_err(|_| RefError::TooLarge)?;
        Array::new(rows, cols, values).map_err(|_| RefError::TooLarge)
    }

    /// A cell's value, formula and number format (top-left cell of a range).
    ///
    /// # Errors
    /// [`RefError`] when the target does not resolve.
    pub fn cell<'a>(&self, r: impl Into<Target<'a>>) -> Result<CellInfo, RefError> {
        let (range, sheet, row, col) = self.top_left(r)?;
        let formula = self
            .model
            .get_cell_formula(sheet, row, col)
            .map_err(RefError::Invalid)?;
        let format = self
            .model
            .get_style_for_cell(sheet, row, col)
            .map_err(RefError::Invalid)?
            .num_fmt;
        Ok(CellInfo {
            value: self.cell_at(sheet, range.top, range.left)?,
            formula,
            number_format: (!format.eq_ignore_ascii_case("general")).then_some(format),
        })
    }

    /// The cell's text as Excel displays it (top-left cell of a range).
    ///
    /// # Errors
    /// [`RefError`] when the target does not resolve.
    pub fn formatted<'a>(&self, r: impl Into<Target<'a>>) -> Result<String, RefError> {
        let (_, sheet, row, col) = self.top_left(r)?;
        self.model
            .get_formatted_cell_value(sheet, row, col)
            .map_err(RefError::Invalid)
    }

    /// The bounding rectangle of stored cells, `None` for an empty sheet.
    ///
    /// # Errors
    /// [`RefError::UnknownSheet`] when `sheet` does not exist.
    pub fn used_range(&self, sheet: SheetId) -> Result<Option<RangeRef>, RefError> {
        let index = self.sheet_index(sheet)?;
        let ws = self
            .model
            .workbook
            .worksheet(index)
            .map_err(|_| RefError::UnknownSheet(sheet.get().to_string()))?;
        let d = ws.dimension();
        // `dimension` reports A1:A1 for an empty sheet.
        if (d.min_row, d.min_column, d.max_row, d.max_column) == (1, 1, 1, 1)
            && ws.cell(1, 1).is_none()
        {
            return Ok(None);
        }
        debug_assert!(d.max_row <= LAST_ROW && d.max_column <= LAST_COLUMN);
        let to_u32 = |v: i32| u32::try_from(v - 1).map_err(|_| RefError::OutOfBounds);
        RangeRef::new(
            sheet,
            to_u32(d.min_row)?,
            to_u32(d.min_column)?,
            to_u32(d.max_row)?,
            to_u32(d.max_column)?,
        )
        .map(Some)
    }

    /// Every stored cell of `sheet` (values, formulas and spill members), by row and within a
    /// row by column. Cheap for a sparse sheet whatever its [`Self::used_range`].
    ///
    /// # Errors
    /// [`RefError::UnknownSheet`] when `sheet` does not exist.
    pub fn stored_cells(&self, sheet: SheetId) -> Result<Vec<CellRef>, RefError> {
        let index = self.sheet_index(sheet)?;
        let ws = self
            .model
            .workbook
            .worksheet(index)
            .map_err(|_| RefError::UnknownSheet(sheet.get().to_string()))?;
        ws.sheet_data
            .cells()
            .map(|(row, col, _)| {
                let to_u32 = |v: i32| u32::try_from(v - 1).map_err(|_| RefError::OutOfBounds);
                CellRef::new(sheet, to_u32(row)?, to_u32(col)?)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::{ErrorKind, XlError};
    use crate::Input;

    #[test]
    fn text_utf16_is_some_only_for_text_holding_a_lone_surrogate() {
        use rustsheet_engine::types::{Cell, FormulaValue};
        use rustsheet_engine::Text;
        let mut wb = Workbook::new(&WorkbookOptions::default()).expect("new");
        wb.set("A1", Input::Formula("=\"a\"".into())).expect("a1");
        wb.set("A2", Input::Formula("=1".into())).expect("a2");
        // No engine function produces a lone surrogate yet (T-1579): plant the cached value.
        let lone = Text::from_utf16(vec![0x61, 0xDE00, 0x62]);
        let sheet = wb.model.workbook.worksheet_mut(0).expect("sheet");
        sheet.sheet_data.set_cell(
            3,
            1,
            Cell::CellFormula {
                f: 0,
                s: 0,
                v: FormulaValue::Text(lone),
            },
        );
        assert_eq!(wb.text_utf16("A1").unwrap(), None);
        assert_eq!(wb.text_utf16("A2").unwrap(), None);
        assert_eq!(wb.text_utf16("A4").unwrap(), None);
        assert_eq!(
            wb.text_utf16("A3").unwrap().as_deref(),
            Some(&[0x61, 0xDE00, 0x62][..])
        );
        assert_eq!(wb.value("A3").unwrap(), Value::from("a\u{FFFD}b"));
    }

    #[test]
    fn date_system_is_chosen_at_new_and_survives_a_round_trip() {
        let default = Workbook::new(&WorkbookOptions::default()).expect("new");
        assert_eq!(default.date_system(), DateSystem::Excel1900);
        let opts = WorkbookOptions {
            date_system: DateSystem::Excel1904,
            ..WorkbookOptions::default()
        };
        let wb = Workbook::new(&opts).expect("new");
        assert_eq!(wb.date_system(), DateSystem::Excel1904);
        let bytes = wb.to_xlsx_bytes(SaveOptions::default()).expect("save");
        let loaded = Workbook::from_xlsx_bytes(&bytes, &LoadOptions::default()).expect("load");
        assert_eq!(loaded.date_system(), DateSystem::Excel1904);
    }

    #[test]
    fn options_name_is_the_file_name_cell_reports_at_new_and_at_load() {
        let named = WorkbookOptions {
            name: Some("Budget".into()),
            ..WorkbookOptions::default()
        };
        let mut wb = Workbook::new(&named).expect("new");
        wb.edit(|tx| tx.add_sheet("Data").map(|_| ()))
            .expect("sheet");
        wb.set("A1", Input::Formula("=CELL(\"address\",Data!B2)".into()))
            .expect("a1");
        assert_eq!(
            wb.value("A1").unwrap(),
            Value::Text("[Budget.xlsx]Data!$B$2".into())
        );

        let bytes = wb.to_xlsx_bytes(SaveOptions::default()).expect("save");
        let unnamed = Workbook::from_xlsx_bytes(&bytes, &LoadOptions::default()).expect("load");
        assert!(matches!(unnamed.value("A1").unwrap(), Value::Error(_)));
        let loaded = LoadOptions {
            workbook: WorkbookOptions {
                name: Some("Other".into()),
                ..WorkbookOptions::default()
            },
            ..LoadOptions::default()
        };
        let renamed = Workbook::from_xlsx_bytes(&bytes, &loaded).expect("load");
        assert_eq!(
            renamed.value("A1").unwrap(),
            Value::Text("[Other.xlsx]Data!$B$2".into())
        );
    }

    #[test]
    fn options_clock_applies_at_new_and_at_the_load_time_recalc() {
        let opts = WorkbookOptions {
            clock: Clock::fixed_unix_millis(0).expect("clock"),
            ..WorkbookOptions::default()
        };
        let mut wb = Workbook::new(&opts).expect("new");
        wb.set("A1", Input::Formula("=NOW()".into())).expect("a1");
        wb.set("A2", Input::Formula("=TODAY()".into())).expect("a2");
        assert_eq!(wb.value("A1").unwrap(), Value::Number(25569.0));
        assert_eq!(wb.value("A2").unwrap(), Value::Number(25569.0));

        let bytes = wb.to_xlsx_bytes(SaveOptions::default()).expect("save");
        let next_day = LoadOptions {
            workbook: WorkbookOptions {
                clock: Clock::fixed_unix_millis(86_400_000).expect("clock"),
                ..WorkbookOptions::default()
            },
            ..LoadOptions::default()
        };
        let loaded = Workbook::from_xlsx_bytes(&bytes, &next_day).expect("load");
        assert_eq!(loaded.value("A1").unwrap(), Value::Number(25570.0));
        assert_eq!(loaded.value("A2").unwrap(), Value::Number(25570.0));
    }

    /// A workbook built through the engine and round-tripped through xlsx.
    fn sample() -> Workbook {
        let mut model = Model::new_empty("w", "en", "UTC", "en").expect("model");
        model.update_cell_with_number(0, 1, 1, 2.5).expect("a1");
        model.update_cell_with_text(0, 2, 1, "hi").expect("a2");
        model
            .update_cell_with_formula(0, 1, 2, "=A1*2".into())
            .expect("b1");
        model
            .update_cell_with_formula(0, 2, 2, "=1/0".into())
            .expect("b2");
        model.add_sheet("My Sheet").expect("sheet");
        model.update_cell_with_bool(1, 3, 3, true).expect("c3");
        model.evaluate();
        let bytes = rustsheet_xlsx::export::save_xlsx_to_writer(&model, Cursor::new(Vec::new()))
            .expect("save")
            .into_inner();
        Workbook::from_xlsx_bytes(&bytes, &LoadOptions::default()).expect("load")
    }

    #[test]
    fn reads_values_formulas_and_errors() {
        let wb = sample();
        assert_eq!(wb.value("A1").unwrap(), Value::Number(2.5));
        assert_eq!(wb.value("Sheet1!a2").unwrap(), Value::Text("hi".into()));
        assert_eq!(wb.value("B1").unwrap(), Value::Number(5.0));
        assert_eq!(
            wb.value("B2").unwrap(),
            Value::Error(XlError::with_detail(ErrorKind::Div0, "Divide by 0"))
        );
        assert_eq!(wb.value("'my sheet'!C3").unwrap(), Value::Bool(true));
        assert_eq!(wb.value("Z9").unwrap(), Value::Empty);
        let info = wb.cell("B1").unwrap();
        assert_eq!(info.formula.as_deref(), Some("=A1*2"));
        assert_eq!(info.number_format, None);
        assert_eq!(wb.formatted("A1").unwrap(), "2.5");
    }

    #[test]
    fn values_returns_row_major_array() {
        let wb = sample();
        let a = wb.values("A1:B2").unwrap();
        assert_eq!((a.rows(), a.cols()), (2, 2));
        assert_eq!(a.get(0, 1), Some(&Value::Number(5.0)));
        assert_eq!(a.get(1, 0), Some(&Value::Text("hi".into())));
    }

    #[test]
    fn oversized_reads_fail_before_allocating() {
        let wb = sample();
        assert!(matches!(wb.values("A:XFD"), Err(RefError::TooLarge)));
        assert!(matches!(wb.values("1:1048576"), Err(RefError::TooLarge)));
    }

    #[test]
    fn sheets_and_used_range() {
        let wb = sample();
        let sheets = wb.sheets();
        assert_eq!(sheets.len(), 2);
        assert_eq!(sheets[1].name, "My Sheet");
        assert_eq!(wb.sheet_id("MY SHEET").unwrap(), sheets[1].id);
        assert!(matches!(
            wb.sheet_id("nope"),
            Err(RefError::UnknownSheet(_))
        ));
        let used = wb.used_range(sheets[0].id).unwrap().unwrap();
        assert_eq!((used.top, used.left, used.bottom, used.right), (0, 0, 1, 1));
        let used = wb.used_range(sheets[1].id).unwrap().unwrap();
        assert_eq!((used.top, used.left), (2, 2));
        let empty = Workbook::new(&WorkbookOptions::default()).unwrap();
        let id = empty.sheets()[0].id;
        assert_eq!(empty.used_range(id).unwrap(), None);
    }

    #[test]
    fn stored_cells_list_spill_members_sparsely() {
        let mut wb = Workbook::new(&WorkbookOptions::default()).unwrap();
        let id = wb.sheets()[0].id;
        wb.set("B2", Input::Formula("=SEQUENCE(1,2)".into()))
            .unwrap();
        wb.set("XFD1048576", Input::Formula("=1".into())).unwrap();
        wb.recalculate();
        let cells: Vec<(u32, u32)> = wb
            .stored_cells(id)
            .unwrap()
            .into_iter()
            .map(|c| (c.row, c.col))
            .collect();
        assert_eq!(cells, vec![(1, 1), (1, 2), (1_048_575, 16_383)]);
    }

    #[test]
    fn xlsx_round_trip_keeps_values() {
        let wb = sample();
        let bytes = wb.to_xlsx_bytes(SaveOptions::default()).unwrap();
        let again = Workbook::from_xlsx_bytes(&bytes, &LoadOptions::default()).unwrap();
        assert_eq!(again.value("B1").unwrap(), Value::Number(5.0));
    }

    #[test]
    fn rejects_garbage_and_zip_limits() {
        assert!(matches!(
            Workbook::from_xlsx_bytes(b"not a zip", &LoadOptions::default()),
            Err(LoadError::Format(_))
        ));
        let bytes = sample().to_xlsx_bytes(SaveOptions::default()).unwrap();
        let few = LoadOptions {
            max_zip_entries: 1,
            ..LoadOptions::default()
        };
        assert!(matches!(
            Workbook::from_xlsx_bytes(&bytes, &few),
            Err(LoadError::Options(_))
        ));
        let small = LoadOptions {
            max_uncompressed_bytes: 10,
            ..LoadOptions::default()
        };
        assert!(matches!(
            Workbook::from_xlsx_bytes(&bytes, &small),
            Err(LoadError::Options(_))
        ));
    }

    /// A valid workbook whose `xl/sharedStrings.xml` (a part the importer reads) is replaced by
    /// `len` zero bytes; deflate keeps the archive tiny.
    fn workbook_with_bomb_part(len: usize) -> Vec<u8> {
        use std::io::Write;
        let good = sample().to_xlsx_bytes(SaveOptions::default()).unwrap();
        let mut src = zip::ZipArchive::new(Cursor::new(good)).unwrap();
        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for i in 0..src.len() {
            let mut entry = src.by_index(i).unwrap();
            let name = entry.name().unwrap().into_owned();
            zip.start_file(&name, zip::write::SimpleFileOptions::default())
                .unwrap();
            if name == "xl/sharedStrings.xml" {
                zip.write_all(&vec![0u8; len]).unwrap();
            } else {
                std::io::copy(&mut entry, &mut zip).unwrap();
            }
        }
        zip.finish().unwrap().into_inner()
    }

    #[test]
    fn zip_bomb_is_rejected_by_actual_inflated_size() {
        let opts = LoadOptions {
            max_uncompressed_bytes: 1 << 20,
            ..LoadOptions::default()
        };
        let bomb = workbook_with_bomb_part(20 << 20);
        assert!(bomb.len() < 100_000, "test zip must be tiny");
        assert!(matches!(
            Workbook::from_xlsx_bytes(&bomb, &opts),
            Err(LoadError::Options(_))
        ));
        // Understate the declared size in the central directory (offset 24 of its header).
        let mut lying = bomb;
        let mut at = 0;
        while let Some(p) = lying[at..].windows(4).position(|w| w == b"PK\x01\x02") {
            at += p;
            let name = &lying[at + 46..at + 46 + 20];
            if name == b"xl/sharedStrings.xml" {
                lying[at + 24..at + 28].copy_from_slice(&10u32.to_le_bytes());
            }
            at += 4;
        }
        // zip 9 stops inflating an entry at its declared size and reports the overrun itself
        // ("File is larger than its declared uncompressed size"), so the lie is a format error
        // before the inflated-bytes budget is reached (T-1668).
        assert!(matches!(
            Workbook::from_xlsx_bytes(&lying, &opts),
            Err(LoadError::Format(_))
        ));
    }

    #[cfg(feature = "metadata_clock")]
    #[test]
    fn pin_clock_reads_the_metadata_now_row() {
        let mut model = Model::new_empty("w", "en", "UTC", "en").expect("model");
        model.add_sheet("METADATA").expect("sheet");
        model.update_cell_with_text(1, 1, 1, "NOW").expect("label");
        model
            .update_cell_with_number(1, 1, 2, 45000.5)
            .expect("serial");
        let bytes = rustsheet_xlsx::export::save_xlsx_to_writer(&model, Cursor::new(Vec::new()))
            .expect("save")
            .into_inner();
        let mut pinned = Workbook::from_xlsx_bytes(&bytes, &LoadOptions::default()).unwrap();
        assert_eq!(pinned.model.clock(), Clock::system());
        assert!(pinned.pin_clock_from_metadata());
        // (45000.5 - 25569) days after the Unix epoch.
        assert_eq!(pinned.model.clock().fixed(), Some(1_678_881_600_000));
        let mut unpinned = sample();
        assert!(!unpinned.pin_clock_from_metadata());
        assert_eq!(unpinned.model.clock(), Clock::system());
    }

    #[cfg(feature = "harness_row_insert")]
    #[test]
    fn insert_rows_shifts_cells_and_rewrites_references() {
        // sample(): A1 2.5, B1 =A1*2.
        let mut wb = sample();
        let sheet = wb.sheet_id("Sheet1").unwrap();
        wb.insert_rows(sheet, 0, 1).unwrap();
        wb.recalculate();
        let moved = wb.cell("B2").unwrap();
        assert_eq!(moved.formula.as_deref(), Some("=A2*2"));
        assert_eq!(moved.value, Value::Number(5.0));
        assert_eq!(wb.value("A1").unwrap(), Value::Empty);
        for (row, count) in [(u32::MAX, 1), (0, 0), (0, u32::MAX)] {
            assert!(matches!(
                wb.insert_rows(sheet, row, count),
                Err(crate::error::EditError::Ref(RefError::OutOfBounds))
            ));
        }
    }

    #[test]
    fn unknown_locale_is_an_options_error() {
        let opts = WorkbookOptions {
            locale: "xx".into(),
            ..WorkbookOptions::default()
        };
        assert!(matches!(Workbook::new(&opts), Err(LoadError::Options(_))));
    }
}
