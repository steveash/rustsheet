//! [`Workbook`]: the facade over the engine `Model` (open/new/save and the read path).

use std::io::{Cursor, Read};
use std::path::Path;

use rustsheet_engine::{Model, LAST_COLUMN, LAST_ROW};

use crate::error::{LoadError, RefError, SaveError};
use crate::refs::{
    parse_cell, parse_range, resolve_sheet_by_name, CellRef, RangeRef, SheetId, Target,
};
use crate::value::{Array, Value};

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
}

impl Default for WorkbookOptions {
    fn default() -> Self {
        Self {
            max_read_cells: 1_000_000,
            locale: "en".to_string(),
            timezone: "UTC".to_string(),
            language: "en".to_string(),
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
        Self {
            recalc: true,
            max_uncompressed_bytes: 256 << 20,
            max_zip_entries: 5_000,
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

/// Rejects zip containers that list too many entries or inflate past the byte limit.
///
/// The declared sizes are only a fast reject (an attacker controls them); the limit is enforced
/// on the bytes actually produced by inflating every entry into a sink, so the importer later
/// decompresses at most the same bounded amount.
fn check_zip_limits(bytes: &[u8], opts: &LoadOptions) -> Result<(), LoadError> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).map_err(engine_load_error)?;
    if u64::try_from(archive.len()).map_or(true, |n| n > u64::from(opts.max_zip_entries)) {
        return Err(LoadError::Options(format!(
            "zip lists {} entries, limit is {}",
            archive.len(),
            opts.max_zip_entries
        )));
    }
    let too_big = || {
        LoadError::Options(format!(
            "zip inflates to more than {} bytes",
            opts.max_uncompressed_bytes
        ))
    };
    let mut declared: u64 = 0;
    for i in 0..archive.len() {
        declared =
            declared.saturating_add(archive.by_index_raw(i).map_err(engine_load_error)?.size());
    }
    if declared > opts.max_uncompressed_bytes {
        return Err(too_big());
    }
    let mut remaining = opts.max_uncompressed_bytes;
    for i in 0..archive.len() {
        let entry = archive.by_index(i).map_err(engine_load_error)?;
        // Read one byte past the budget so an overrun is distinguishable from an exact fit.
        let limit = remaining.saturating_add(1);
        let copied = std::io::copy(&mut entry.take(limit), &mut std::io::sink())
            .map_err(engine_load_error)?;
        if copied > remaining {
            return Err(too_big());
        }
        remaining -= copied;
    }
    Ok(())
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
        let model = Model::new_empty("workbook", &opts.locale, &opts.timezone, &opts.language)
            .map_err(LoadError::Options)?;
        Ok(Self::from_model(model, opts.max_read_cells))
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
        check_zip_limits(bytes, opts)?;
        let w = &opts.workbook;
        let book =
            rustsheet_xlsx::import::load_from_xlsx_bytes(bytes, "workbook", &w.locale, &w.timezone)
                .map_err(engine_load_error)?;
        let model = Model::from_workbook(book, &w.language).map_err(LoadError::Format)?;
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

    /// Pins the engine clock (`NOW`, `TODAY`) to the timestamp in the workbook's `METADATA`
    /// sheet, as Excel-authored test workbooks carry it. Returns whether a timestamp was found.
    #[cfg(feature = "mock_time")]
    pub fn pin_clock_from_metadata(&self) -> bool {
        let Some(millis) = rustsheet_xlsx::mock_time::get_metadata_timestamp(&self.model) else {
            return false;
        };
        rustsheet_engine::mock_time::set_mock_time(millis);
        true
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::{ErrorKind, XlError};

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

    fn zip_of_zeros(len: usize) -> Vec<u8> {
        use std::io::Write;
        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
        zip.start_file("bomb.xml", zip::write::FileOptions::default())
            .unwrap();
        zip.write_all(&vec![0u8; len]).unwrap();
        zip.finish().unwrap().into_inner()
    }

    #[test]
    fn zip_bomb_is_rejected_by_actual_inflated_size() {
        let opts = LoadOptions {
            max_uncompressed_bytes: 1 << 20,
            ..LoadOptions::default()
        };
        let bomb = zip_of_zeros(20 << 20);
        assert!(bomb.len() < 100_000, "test zip must be tiny");
        assert!(matches!(
            Workbook::from_xlsx_bytes(&bomb, &opts),
            Err(LoadError::Options(_))
        ));
        // Understate the declared size in the central directory (offset 24 of its header).
        let mut lying = bomb;
        let at = lying
            .windows(4)
            .position(|w| w == b"PK\x01\x02")
            .expect("central directory");
        lying[at + 24..at + 28].copy_from_slice(&10u32.to_le_bytes());
        assert!(matches!(
            Workbook::from_xlsx_bytes(&lying, &opts),
            Err(LoadError::Options(_))
        ));
    }

    #[cfg(feature = "mock_time")]
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
        let pinned = Workbook::from_xlsx_bytes(&bytes, &LoadOptions::default()).unwrap();
        assert!(pinned.pin_clock_from_metadata());
        assert!(!sample().pin_clock_from_metadata());
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
