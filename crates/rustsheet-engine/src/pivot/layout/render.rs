//! A pivot's cells: the captions, item labels, values and page area its layout lines give
//! (SPEC-0032 "Rendering and the grid", T-4149). [`render`] is a pure function of the
//! definition, the cache, the [`PivotResult`] and the [`PivotLayout`]; writing the cells into the
//! grid and owning the area are T-4151's.
//!
//! Evidence: the `pivots` cells of every probe sidecar in scope (docs/excel-semantics/pivots.md
//! "Rendered pivot cells", "Caption attributes, empty and error captions...", "Page fields render
//! above the location", "A tabular pivot's first row header is the field name only when every
//! pivotField has compact="0"", "Items show in the order the pivot field lists them"):
//! - **Column header.** With an axis field on columns, the location's top row holds `Column
//!   Labels` (`colHeaderCaption`) over the first data column, or the first column field's name
//!   when every pivot field has `compact="0"` (probes/t4583-pivot-row-labels C1, C2); with only
//!   Values on columns it holds `Values` (`dataCaption`; t4137 K1, t4138 G9). Each column line
//!   labels its levels from `r` on, one header row per level below the top row.
//! - **Row header.** The last header row holds `Row Labels` (`rowHeaderCaption`) over the first
//!   label column, or the first row field's name when every pivot field has `compact="0"`, which
//!   wins over `rowHeaderCaption` too (probes/t4609-pivot-row-labels-2 L5); each
//!   further label column holds the name of the first field in it (t4123 T, T2; t4138 G3, G4).
//!   Each row line labels its levels from `r` on, each in its label column (compact form puts
//!   several levels in one column; the sidecars hold values only, so indentation is not
//!   evidenced and a label is the item's text alone).
//! - **One data field** with Values on no axis: its caption sits at the location's corner when
//!   both axes have a field (t4123 C), over the data column when the column axis is empty (t4138
//!   G1, G8), and as the row label when only the row axis is empty (G7).
//! - **Totals.** A default subtotal reads `<item> Total`, a custom one `<item> <Function>` (t4137
//!   K5, t4622 B1), the grand total `Grand Total` (`grandTotalCaption`), or `Total <data field>`
//!   with Values on its axis (t4138 G6, t4135 `A_sum`).
//! - **Classic layout** (`gridDropZones`) with no column level: the data caption sits at the
//!   location's corner and the header row heads the value column `Total` (t4129 GZ,
//!   probes/t4745-pivot-classic-subtotal-caption G1); with a column level nothing changes (GC,
//!   GV).
//! - **subtotalCaption** is the whole label of a field's default subtotal lines, on rows and
//!   columns, a `?` in it standing for the item; custom subtotal lines keep `<item> <Function>`
//!   (t4129 SC, t4745 SB, SK, SF, SCC, SQ).
//! - **Repeat item labels.** A field with the x14 `fillDownLabels` shows its label on every line
//!   below its first, inner subtotal lines included, in tabular and outline form and on columns;
//!   a cell shared with a deeper level is that level's, so compact form fills nothing
//!   (probes/t4698-pivot-repeat-labels; pivots.md "Repeat item labels").
//! - **Items** render as in pivots.md: a number as a number, a boolean as `TRUE`/`FALSE`, an
//!   error as its text and a blank as `(blank)` (t4134 I2); a Values level item is the data
//!   field's caption. A date item is its serial, in `m/d/yyyy` (with ` h:mm` for a time part)
//!   whatever the source's format (`date_item_format`, t2085 D, t4164).
//! - **Values.** An intersection no record reaches is blank, or `missingCaption` with
//!   `showMissing` (t4137 K2); an error is `errorCaption` with `showError` (K3, t4135 EC).
//!   Every value cell, subtotals' and grand totals' too, carries its data field's `numFmtId`
//!   ([`RenderedCell::number_format`]), which the grid writer gives the cell, so its displayed
//!   text is Excel's (pivots.md "Value cells show their data field's number format",
//!   probes/t4169-pivot-number-formats).
//! - **Page fields** take two cells, the field's name and `(All)`, the selected item or
//!   `(Multiple Items)` when it hides items (t4137 K4), in the `rowPageCount` rows ending one row
//!   above the location, three columns per page column (t4138 G8).
//!
//! Each cell copies its label, so the cells' text is bounded by [`MAX_PIVOT_TEXT_BYTES`]
//! (pivots.md "A pivot result is bounded in work and nodes", DEVIATION); their count is bounded
//! by the layout's cell cap.
//!
//! The engine's own readings where no probe shows Excel's (pivots.md "Rendered pivot cells",
//! UNVERIFIED):
//! - a number or date item's subtotal label (`number_to_text` of its value), a page field whose
//!   hidden items leave one item, and a selected number, boolean or blank page item (T-4657);
//! - a subtotal label with Values inside its field (`<item> <data field>`), the grand-total
//!   caption with Values on its axis, and the header over a Values level in a label column of
//!   its own (left empty) (T-4650);
//! - Repeat Item Labels beside an unlabelled deeper level in the same column, on an inner column
//!   field's subtotal, beside the Values field, and on custom subtotal lines (T-4764).
//! - classic layout in compact or outline form, with no row field, with Values on rows (the
//!   corner left empty), with no data field, beside page fields, and beside
//!   `grandTotalCaption` (always `Total`); a subtotalCaption with Values inside its field (kept
//!   `<item> <data field>`), empty, or with several `?` (each replaced) (T-4785).

use crate::expressions::token::Error;
use crate::number_format::number_to_text;

use super::super::result::{page_selection, AxisTree, PivotComputeError, PivotResult};
use super::super::{Aggregate, CacheItem, ItemType, PivotCache, PivotTable};
use super::lines::{label_columns, levels, AxisLine, Level, PivotLayout};

/// The caption above a pivot's row labels (`rowHeaderCaption` replaces it).
pub const ROW_LABELS: &str = "Row Labels";
/// The caption above a pivot's column labels (`colHeaderCaption` replaces it).
pub const COLUMN_LABELS: &str = "Column Labels";
/// The caption above the data fields' captions (`dataCaption` replaces it).
pub const VALUES: &str = "Values";
/// The label of a blank item, and the text GETPIVOTDATA takes for that item on an ungrouped
/// field (probes/t4236-pivot-date-groups).
pub const BLANK_ITEM: &str = "(blank)";
/// The label of the grand-total line (`grandTotalCaption` replaces it).
pub const GRAND_TOTAL: &str = "Grand Total";
/// The header of a classic-layout pivot's one empty column line (`gridDropZones`, t4129 GZ!C4).
pub const CLASSIC_TOTAL: &str = "Total";
/// A page field with no selection and no hidden item.
pub const ALL_ITEMS: &str = "(All)";
/// A page field that hides items.
pub const MULTIPLE_ITEMS: &str = "(Multiple Items)";

/// The value of one rendered pivot cell; an empty cell is not rendered.
#[derive(Debug, Clone, PartialEq)]
pub enum PivotCell {
    /// A caption, an item label, or a caption standing for a value.
    Text(String),
    /// A number item's label or a value.
    Number(f64),
    /// A value's error.
    Error(Error),
}

/// One rendered cell: its sheet row and column (0-based), its value and its number format.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderedCell {
    /// The cell's row.
    pub row: u32,
    /// The cell's column.
    pub column: u32,
    /// The cell's value.
    pub value: PivotCell,
    /// The number format the cell shows its value in: a value cell's data field's
    /// ([`DataField::number_format`](super::super::DataField::number_format)) or a date item's
    /// ([`date_item_format`]); `None` for any other label, a caption, and a value cell whose data
    /// field has none, whose cell format is left as it is.
    pub number_format: Option<CellFormat>,
}

/// The number format a rendered cell is given.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellFormat {
    /// A `numFmtId`, as the workbook's styles resolve it: a data field's format.
    Id(u32),
    /// A format code: a date item's label.
    Code(&'static str),
}

/// The format of a date item's label (pivots.md "Item casing when cache and source disagree, and
/// date items"): Excel shows it as `m/d/yyyy`, with ` h:mm` when it has a time part, whatever the
/// source cells' format. probes/t2085-getpivotdata-items D!A4:A6 and t4130-pivot-date-filters
/// (source `m/d/yyyy`) read `1/1/2024`; t4164-pivot-date-times (source `m/d/yyyy h:mm`) reads
/// `4/17/2024` for midnight and `4/17/2024 8:00` for 8 a.m.
fn date_item_format(serial: f64) -> CellFormat {
    CellFormat::Code(if serial.fract() == 0.0 {
        "m/d/yyyy"
    } else {
        "m/d/yyyy h:mm"
    })
}

/// How an item renders as a label (pivots.md "Items show in the order the pivot field lists
/// them"): a date is its serial, as Excel writes an ungrouped date field's label cell
/// (probes/t4236-pivot-date-groups reads `43496.9999884259`).
fn item_cell(item: &CacheItem) -> PivotCell {
    match item {
        CacheItem::Text(text) => PivotCell::Text(text.clone()),
        CacheItem::Number(n) | CacheItem::Date(n) => PivotCell::Number(*n),
        CacheItem::Boolean(b) => PivotCell::Text(if *b { "TRUE" } else { "FALSE" }.to_string()),
        CacheItem::Error(error) => PivotCell::Text(error.to_string()),
        CacheItem::Missing => PivotCell::Text(BLANK_ITEM.to_string()),
    }
}

/// An item's label as text, for a subtotal's `<item> Total` and a caption filter
/// (`pivot::filter`).
pub(crate) fn item_text(item: &CacheItem) -> String {
    match item_cell(item) {
        PivotCell::Text(text) => text,
        PivotCell::Number(n) => number_to_text(n),
        PivotCell::Error(error) => error.to_string(),
    }
}

/// One axis as the renderer reads it.
struct Axis<'a> {
    tree: &'a AxisTree,
    levels: Vec<Level<'a>>,
    lines: &'a [AxisLine],
}

impl Axis<'_> {
    fn has_field(&self) -> bool {
        self.levels.iter().any(|l| matches!(l, Level::Field { .. }))
    }

    fn has_values(&self) -> bool {
        self.levels.iter().any(|l| matches!(l, Level::Values))
    }

    /// The first field level's cache field.
    fn first_field(&self) -> Option<u32> {
        self.levels.iter().find_map(|level| match level {
            Level::Field { depth, .. } => self.tree.fields().get(*depth).copied(),
            Level::Values => None,
        })
    }

    /// The cache item of tree field `depth` on the path to `node`.
    fn item(&self, node: u32, depth: usize) -> Option<(u32, u32)> {
        let nodes = self.tree.nodes();
        let wanted = u32::try_from(depth + 1).ok()?;
        let mut at = nodes.get(node as usize)?;
        while at.depth > wanted {
            at = nodes.get(at.parent? as usize)?;
        }
        let field = *self.tree.fields().get(depth)?;
        (at.depth == wanted).then_some((field, at.item?))
    }
}

/// The cells being rendered, and what their text depends on.
struct Cells<'a> {
    table: &'a PivotTable,
    cache: &'a PivotCache,
    out: Vec<RenderedCell>,
    /// The text bytes the cells may still take ([`MAX_PIVOT_TEXT_BYTES`]).
    text_left: usize,
    /// A cell's text passed `text_left`: nothing more is kept and the render fails.
    over: bool,
}

impl Cells<'_> {
    fn put(&mut self, row: u32, column: u32, value: PivotCell) {
        self.put_formatted(row, column, value, None);
    }

    fn put_formatted(
        &mut self,
        row: u32,
        column: u32,
        value: PivotCell,
        number_format: Option<CellFormat>,
    ) {
        if let PivotCell::Text(text) = &value {
            match self.text_left.checked_sub(text.len()) {
                Some(left) if !self.over => self.text_left = left,
                _ => self.over = true,
            }
        }
        if !self.over {
            self.out.push(RenderedCell {
                row,
                column,
                value,
                number_format,
            });
        }
    }

    fn text(&mut self, row: u32, column: u32, text: &str) {
        self.put(row, column, PivotCell::Text(text.to_string()));
    }

    fn field_name(&self, field: u32) -> Option<&str> {
        self.cache
            .fields
            .get(field as usize)
            .map(|f| f.name.as_str())
    }

    fn cache_item(&self, field: u32, item: u32) -> Option<&CacheItem> {
        self.cache
            .fields
            .get(field as usize)?
            .items
            .get(item as usize)
    }

    fn data_caption(&self, data_field: u32) -> Option<&str> {
        let field = self.table.data_fields.get(data_field as usize)?;
        Some(field.caption.as_str())
    }

    /// The label of `line`'s level `level` (its `x` is `x`) on `axis`, and its number format.
    fn label(
        &self,
        axis: &Axis<'_>,
        line: &AxisLine,
        level: usize,
        x: u32,
    ) -> Option<(PivotCell, Option<CellFormat>)> {
        match axis.levels.get(level)? {
            Level::Values => Some((PivotCell::Text(self.data_caption(x)?.to_string()), None)),
            Level::Field { depth, .. } => {
                let (field, item) = axis.item(line.node, *depth)?;
                let item = self.cache_item(field, item)?;
                let format = match item {
                    CacheItem::Date(serial) => Some(date_item_format(*serial)),
                    _ => None,
                };
                Some((item_cell(item), format))
            }
        }
    }

    /// The item a subtotal `line` of `axis` totals, as text.
    fn line_item(&self, axis: &Axis<'_>, line: &AxisLine) -> Option<String> {
        match axis.levels.get(line_depth(line)?)? {
            Level::Field { depth: d, .. } => {
                let (field, item) = axis.item(line.node, *d)?;
                Some(item_text(self.cache_item(field, item)?))
            }
            Level::Values => None,
        }
    }

    /// The text of a subtotal or grand-total line of `axis`, or `None` for an item's line.
    ///
    /// # Errors
    /// [`PivotComputeError::Limit`] when a `subtotalCaption`'s copies of the item would pass
    /// the text left ([`MAX_PIVOT_TEXT_BYTES`]), checked before they are made.
    fn total_label(
        &self,
        axis: &Axis<'_>,
        line: &AxisLine,
    ) -> Result<Option<String>, PivotComputeError> {
        let options = &self.table.options;
        let depth = line_depth(line);
        let item = || self.line_item(axis, line);
        // Values inside the subtotalled field: one line per data field (T-4650).
        let values_inside = || {
            let inner = depth.map_or(0, |d| d + 1);
            axis.levels
                .iter()
                .skip(inner)
                .any(|l| matches!(l, Level::Values))
        };
        Ok(match line.kind {
            ItemType::Grand if axis.has_values() => Some(format!(
                "Total {}",
                self.data_caption(line.data_field).unwrap_or_default()
            )),
            ItemType::Grand => Some(
                options
                    .grand_total_caption
                    .clone()
                    .unwrap_or_else(|| GRAND_TOTAL.to_string()),
            ),
            ItemType::Default if values_inside() => item().map(|item| {
                format!(
                    "{item} {}",
                    self.data_caption(line.data_field).unwrap_or_default()
                )
            }),
            // `subtotalCaption` is the whole label (t4129 SC, t4745 SB, SK, SCC), a `?` in it
            // standing for the item (t4745 SQ); custom subtotal lines keep `<item> <Function>`
            // (t4745 SF).
            ItemType::Default => match self.subtotal_caption(axis, depth) {
                Some(caption) if caption.contains('?') => match item() {
                    Some(item) => Some(expand_caption(caption, &item, self.text_left)?),
                    None => None,
                },
                Some(caption) => Some(caption.to_string()),
                None => item().map(|item| format!("{item} Total")),
            },
            ItemType::Function(function) => {
                item().map(|item| format!("{item} {}", Aggregate::word(function)))
            }
            _ => None,
        })
    }

    /// The `subtotalCaption` of the field at `axis`'s level `depth`, if it sets one.
    fn subtotal_caption<'f>(&self, axis: &Axis<'f>, depth: Option<usize>) -> Option<&'f str> {
        match axis.levels.get(depth?)? {
            Level::Field {
                field: Some(field), ..
            } => field.subtotal_caption.as_deref(),
            _ => None,
        }
    }

    /// The labels of `axis`'s lines; `place(i, level)` is the cell of line `i`'s label at
    /// `level`.
    fn lines(&mut self, axis: &Axis<'_>, place: impl Fn(u32, usize) -> Option<(u32, u32)>) {
        for (i, line) in (0u32..).zip(axis.lines) {
            if self.over {
                return;
            }
            // An `insertBlankRow` line is empty (probes/t4168-pivot-layout-options BR, BR2).
            if line.kind == ItemType::Blank {
                continue;
            }
            let end = line.repeat as usize + line.items.len();
            // A label past the text limit fails the render, as [`Cells::put`] does.
            let Ok(label) = self.total_label(axis, line) else {
                self.over = true;
                return;
            };
            if let Some(text) = label {
                let level = if line.kind == ItemType::Grand {
                    0
                } else {
                    // A subtotal line repeats the labels above its own level (t4698 T3R!A6).
                    let level = end.saturating_sub(1);
                    self.fill_down(axis, line, level, |l| place(i, l));
                    level
                };
                if let Some((row, column)) = place(i, level) {
                    self.put(row, column, PivotCell::Text(text));
                }
                continue;
            }
            self.fill_down(axis, line, line.repeat as usize, |l| place(i, l));
            for (level, &x) in (line.repeat as usize..).zip(&line.items) {
                let cell = self.label(axis, line, level, x);
                // An empty-text item's label cell is left empty (pivots.md "Pivot field names
                // for duplicate and blank source headers", probes/t4627-pivot-field-name-collisions
                // P_Rec!A6, a row item; a column item's is taken to match, UNVERIFIED).
                let cell = cell.filter(|(c, _)| !matches!(c, PivotCell::Text(t) if t.is_empty()));
                if let (Some((cell, format)), Some((row, column))) = (cell, place(i, level)) {
                    self.put_formatted(row, column, cell, format);
                }
            }
        }
    }

    /// Repeat Item Labels: on `line`, the label of each level above `below` whose field has
    /// `fillDownLabels` (the levels the line shares with the line before, which it does not
    /// label itself), in that level's cell (pivots.md "Repeat item labels",
    /// probes/t4698-pivot-repeat-labels). A cell belongs to the deepest level of the line placed
    /// in it, so a level whose column it shares with the next level fills nothing: compact form
    /// fills nothing down (CF), while a tabular field in a compact field's column does (MX!A6).
    fn fill_down(
        &mut self,
        axis: &Axis<'_>,
        line: &AxisLine,
        below: usize,
        place: impl Fn(usize) -> Option<(u32, u32)>,
    ) {
        for level in 0..below {
            // Past the text limit nothing more is kept, so no more labels are made.
            if self.over {
                return;
            }
            let Some(Level::Field {
                field: Some(field), ..
            }) = axis.levels.get(level)
            else {
                continue;
            };
            if !field.fill_down_labels {
                continue;
            }
            // Label columns only grow with the level, so the next level is the one that can
            // share this one's cell.
            let at = place(level);
            if at.is_none() || at == place(level + 1) {
                continue;
            }
            let cell = self.label(axis, line, level, 0);
            // An empty-text item's label stays empty here too, as on its own line.
            let cell = cell.filter(|(c, _)| !matches!(c, PivotCell::Text(t) if t.is_empty()));
            if let (Some((cell, format)), Some((row, column))) = (cell, at) {
                self.put_formatted(row, column, cell, format);
            }
        }
    }

    /// The value cell of `row` by `column` and its data field's `numFmtId`, or `None` for a
    /// blank.
    fn value(
        &self,
        result: &PivotResult,
        [rows, columns]: [&Axis<'_>; 2],
        row: &AxisLine,
        column: &AxisLine,
    ) -> Option<(PivotCell, Option<CellFormat>)> {
        let options = &self.table.options;
        let data = if rows.has_values() {
            row.data_field
        } else if columns.has_values() {
            column.data_field
        } else {
            0
        };
        let field = self.table.data_fields.get(data as usize)?;
        let function = match (row.kind, column.kind) {
            (ItemType::Function(f), _) | (_, ItemType::Function(f)) => f,
            _ => field.function,
        };
        let cell = match result.value_as(row.node, column.node, data as usize, function) {
            None => options
                .missing_caption
                .clone()
                .filter(|_| options.show_missing)
                .map(PivotCell::Text),
            Some(Ok(n)) => Some(PivotCell::Number(n)),
            Some(Err(error)) => Some(match &options.error_caption {
                Some(caption) if options.show_error => PivotCell::Text(caption.clone()),
                _ => PivotCell::Error(error),
            }),
        };
        cell.map(|cell| (cell, field.number_format.map(CellFormat::Id)))
    }
}

/// Most bytes of text one pivot's rendered cells hold together (SPEC-0032 "Untrusted-input
/// limits"): a label is copied into every cell that shows it, so a long item or caption on many
/// lines would otherwise cost lines times its length. 128 MiB, about 32 bytes for each of
/// [`MAX_PIVOT_CELLS`](super::MAX_PIVOT_CELLS) cells.
pub const MAX_PIVOT_TEXT_BYTES: usize = 1 << 27;

/// The cells of `table`'s pivot over `cache`, from its `result` ([`compute`](super::super::compute))
/// and `layout` ([`lay_out`](super::lay_out)), in no particular order: every non-empty cell of
/// the location and of the page area above it (see the module documentation).
///
/// # Errors
/// [`PivotComputeError::Malformed`] when `result` or `layout` is not over `table` and `cache`
/// (a line's node or item is not in the result's tree or the cache), and
/// [`PivotComputeError::Limit`] when the cells' text passes [`MAX_PIVOT_TEXT_BYTES`].
pub fn render(
    table: &PivotTable,
    cache: &PivotCache,
    result: &PivotResult,
    layout: &PivotLayout,
) -> Result<Vec<RenderedCell>, PivotComputeError> {
    render_within(table, cache, result, layout, MAX_PIVOT_TEXT_BYTES)
}

/// [`render`] with at most `text_bytes` bytes of text.
fn render_within(
    table: &PivotTable,
    cache: &PivotCache,
    result: &PivotResult,
    layout: &PivotLayout,
    text_bytes: usize,
) -> Result<Vec<RenderedCell>, PivotComputeError> {
    let options = &table.options;
    let rows = Axis {
        tree: result.rows(),
        levels: levels(table, &table.row_fields),
        lines: &layout.rows,
    };
    let columns = Axis {
        tree: result.columns(),
        levels: levels(table, &table.column_fields),
        lines: &layout.columns,
    };
    check_lines(&rows)?;
    check_lines(&columns)?;
    let (top, left) = (layout.location.top, layout.location.left);
    let (data_row, data_column) = layout.data_origin();
    let header_row = data_row.saturating_sub(1).max(top);
    let mut cells = Cells {
        table,
        cache,
        out: Vec::with_capacity(
            layout
                .rows
                .len()
                .saturating_add(2)
                .saturating_mul(layout.columns.len().saturating_add(2)),
        ),
        text_left: text_bytes,
        over: false,
    };
    let named = !table.fields.is_empty() && table.fields.iter().all(|f| !f.compact);
    let name = |field: Option<u32>| {
        let field = cache.fields.get(field? as usize)?;
        Some(field.name.clone())
    };

    // The column header over the first data column.
    if !columns.levels.is_empty() {
        let caption = if columns.has_field() {
            let named = name(columns.first_field()).filter(|_| named);
            named.or_else(|| options.column_header_caption.clone())
        } else {
            options.data_caption.clone()
        };
        let fallback = if columns.has_field() {
            COLUMN_LABELS
        } else {
            VALUES
        };
        cells.text(top, data_column, caption.as_deref().unwrap_or(fallback));
    }
    // The row header and the label columns' field names.
    let label_column = label_columns(&rows.levels, options);
    if rows.has_field() {
        let named = name(rows.first_field()).filter(|_| named);
        let caption = named.or_else(|| options.row_header_caption.clone());
        cells.text(header_row, left, caption.as_deref().unwrap_or(ROW_LABELS));
        let mut seen = 0;
        for (level, &column) in rows.levels.iter().zip(&label_column) {
            if column == seen {
                continue;
            }
            seen = column;
            if let Level::Field { depth, .. } = level {
                if let Some(text) = name(rows.tree.fields().get(*depth).copied()) {
                    cells.text(header_row, left.saturating_add(column), &text);
                }
            }
        }
    }
    // Classic layout with no column level: the column header row heads the axis's one empty
    // line `Total` (t4129 GZ!C4, t4745 G1!B4).
    let classic = options.grid_drop_zones && columns.levels.is_empty();
    if classic {
        cells.text(header_row, data_column, CLASSIC_TOTAL);
    }
    // One data field with Values on no axis: its caption.
    if !rows.has_values() && !columns.has_values() {
        if let Some(caption) = cells.data_caption(0).map(str::to_string) {
            if classic {
                // At the location's corner, as with a column field (t4129 GZ!A3, t4745 G1!A3).
                cells.text(top, left, &caption);
            } else if columns.levels.is_empty() {
                cells.text(header_row, data_column, &caption);
            } else if rows.has_field() {
                cells.text(top, left, &caption);
            } else if let Some(column) = data_column.checked_sub(1).filter(|&c| c >= left) {
                cells.text(data_row, column, &caption);
            }
        }
    }
    let row_place = |i: u32, level: usize| {
        let column = *label_column.get(level)?;
        Some((data_row.checked_add(i)?, left.checked_add(column)?))
    };
    cells.lines(&rows, row_place);
    let column_place = |i: u32, level: usize| {
        let row = top
            .checked_add(1)?
            .checked_add(u32::try_from(level).ok()?)?;
        Some((row, data_column.checked_add(i)?))
    };
    cells.lines(&columns, column_place);

    for (r, row) in (data_row..).zip(&layout.rows).filter(|(_, l)| l.values) {
        for (c, column) in (data_column..)
            .zip(&layout.columns)
            .filter(|(_, l)| l.values)
        {
            if let Some((value, format)) = cells.value(result, [&rows, &columns], row, column) {
                cells.put_formatted(r, c, value, format);
            }
        }
    }
    page_area(&mut cells, layout);
    if cells.over {
        return Err(PivotComputeError::Limit);
    }
    Ok(cells.out)
}

/// The index of the deepest level `line` covers (its repeated levels plus its own items), which
/// is the level a subtotal line totals; `None` for an empty line.
fn line_depth(line: &AxisLine) -> Option<usize> {
    (line.repeat as usize + line.items.len()).checked_sub(1)
}

/// A `subtotalCaption` with each `?` replaced by `item` (probes/t4745-pivot-classic-subtotal-caption
/// SQ), its size checked against `text_left` before the copies are made.
///
/// # Errors
/// [`PivotComputeError::Limit`] when the label would pass `text_left`.
fn expand_caption(
    caption: &str,
    item: &str,
    text_left: usize,
) -> Result<String, PivotComputeError> {
    let marks = caption.matches('?').count();
    // Each `?` is one byte.
    let bytes = (caption.len() - marks).saturating_add(marks.saturating_mul(item.len()));
    if bytes > text_left {
        return Err(PivotComputeError::Limit);
    }
    Ok(caption.replace('?', item))
}

/// Every line's node is in its axis's tree.
fn check_lines(axis: &Axis<'_>) -> Result<(), PivotComputeError> {
    let nodes = axis.tree.nodes().len();
    if axis.lines.iter().all(|l| (l.node as usize) < nodes) {
        Ok(())
    } else {
        Err(PivotComputeError::Malformed)
    }
}

/// The page fields' names and selections, above the location.
fn page_area(cells: &mut Cells<'_>, layout: &PivotLayout) {
    let table = cells.table;
    let pages = table.page_fields.len();
    let wrap = match table.options.page_wrap as usize {
        0 => pages,
        wrap => wrap.min(pages),
    };
    let first_row = layout
        .location
        .top
        .checked_sub(layout.geometry.row_page_count.saturating_add(1));
    for (k, page) in table.page_fields.iter().enumerate() {
        let (row, column) = if table.options.page_over_then_down {
            (k / wrap, k % wrap)
        } else {
            (k % wrap, k / wrap)
        };
        let row = first_row.and_then(|r| r.checked_add(u32::try_from(row).ok()?));
        let column = u32::try_from(column.saturating_mul(3))
            .ok()
            .and_then(|c| layout.location.left.checked_add(c));
        let (Some(row), Some(column)) = (row, column) else {
            continue;
        };
        let Some(name) = cells.field_name(page.field).map(str::to_string) else {
            continue;
        };
        let hides = table
            .fields
            .get(page.field as usize)
            .is_some_and(|f| f.hidden.iter().any(|word| *word != 0));
        let selection = match page_selection(table, page) {
            Some(item) => cells.cache_item(page.field, item).map(item_cell),
            None if hides => Some(PivotCell::Text(MULTIPLE_ITEMS.to_string())),
            None => Some(PivotCell::Text(ALL_ITEMS.to_string())),
        };
        cells.text(row, column, &name);
        if let Some(selection) = selection {
            cells.put(row, column.saturating_add(1), selection);
        }
    }
}

#[cfg(test)]
mod tests;
