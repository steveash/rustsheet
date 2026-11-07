#![allow(clippy::unwrap_used, clippy::panic, reason = "tests")]
//! The probe harness the pivot stage tests share: a probe's case file (its source cells and the
//! case builder's pivot definition) read back into the cache and definition Excel refreshed, and its
//! oracle sidecar (Excel's saved `pivots`: cells, `rowItems`/`colItems` and `location`), so every
//! expectation comes from the committed probe files, never from the engine.

use std::collections::HashMap;

use serde_json::Value as Json;

use crate::expressions::token::get_error_by_english_name;
use crate::expressions::utils::number_to_column;
use crate::model::Model;
use crate::pivot::{compute_on, lay_out, render, PivotCell, MAX_PIVOT_WORK};
use crate::pivot::{
    read_source, Aggregate, Axis, AxisField, CacheBuilder, CacheField, CacheItem, CacheSource,
    CellArea, DataField, PageField, PivotCache, PivotField, PivotItem, PivotTable, RecordValue,
    ShowDataAs, SourceArea, SourceCells, MAX_RECORD_VALUES,
};

/// The pivot sheets of probes/t4634-pivot-total-order-4 that the result, line and render tests
/// check: Sum beside each other aggregate (X1 to X12) and alone (S1, O1 to O4), Average alone (A1
/// to A3), Var, Varp and `StdDevp` totals (V1 to V7), two fields per axis (T1 to T3), and Count or
/// `CountNums` beside only Max, Min, Product or a variance (C1 to C7).
pub(crate) const T4634_SHEETS: &[&str] = &[
    "S1", "X1", "X2", "X3", "X4", "X5", "X6", "X7", "X8", "X9", "X10", "X11", "X12", "A1", "A2",
    "A3", "O1", "O2", "O3", "O4", "V1", "V2", "V3", "V4", "V5", "V6", "V7", "T1", "T2", "T3", "C1",
    "C2", "C3", "C4", "C5", "C6", "C7",
];

/// The pivot sheets of probes/t4745-pivot-classic-subtotal-caption: classic layout with a
/// column field (GC), two data fields (GV) and one row field (G1); subtotalCaption in outline
/// form at the top (SO) and at the bottom (SB), compact form (SK), over custom subtotals (SF), on
/// columns (SCC) and with a `?` for the item (SQ).
pub(crate) const T4745_SHEETS: &[&str] = &["GC", "GV", "G1", "SO", "SB", "SK", "SF", "SCC", "SQ"];

pub(crate) fn read(stem: &str, kind: &str) -> Json {
    let path = format!(
        "{}/../../eval/suites/probes/cases/{stem}.{kind}.json",
        env!("CARGO_MANIFEST_DIR")
    );
    serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap()
}

pub(crate) fn sheet<'a>(case: &'a Json, name: &str) -> &'a Json {
    case["sheets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["name"] == name)
        .unwrap_or_else(|| panic!("no sheet {name}"))
}

/// `(column, row)`, both 0-based, of an A1 address.
pub(crate) fn position(address: &str) -> (u32, u32) {
    let split = address.find(|c: char| c.is_ascii_digit()).unwrap();
    let (letters, digits) = address.split_at(split);
    let column = letters
        .bytes()
        .fold(0, |acc, b| acc * 26 + u32::from(b - b'A' + 1));
    (column - 1, digits.parse::<u32>().unwrap() - 1)
}

pub(crate) fn address(column: u32, row: u32) -> String {
    let letters = number_to_column(i32::try_from(column + 1).unwrap()).unwrap();
    format!("{letters}{}", row + 1)
}

/// A case cell's value as a cache holds it.
pub(crate) fn cache_item(value: Option<&Json>) -> CacheItem {
    match value {
        None => CacheItem::Missing,
        Some(Json::String(text)) => {
            assert!(!text.starts_with('='), "a formula in a source: {text}");
            CacheItem::Text(text.clone())
        }
        Some(Json::Number(n)) => CacheItem::Number(n.as_f64().unwrap()),
        Some(Json::Bool(b)) => CacheItem::Boolean(*b),
        // A typed text cell: text that would read as a number (probes/t4134-pivot-items D5!A3).
        Some(typed) if typed["t"] == "s" => CacheItem::Text(typed["v"].as_str().unwrap().into()),
        Some(error) => {
            assert_eq!(error["t"], "e");
            CacheItem::Error(get_error_by_english_name(error["v"].as_str().unwrap()).unwrap())
        }
    }
}

/// A case sheet's cells as a pivot source.
struct CaseCells<'a> {
    cells: &'a Json,
    area: SourceArea,
    /// Whether a formula cell reads as empty, as the case builder caches it, rather than panics.
    formulas_blank: bool,
}

impl SourceCells for CaseCells<'_> {
    fn resolve(&self, _: &CacheSource) -> Option<SourceArea> {
        Some(self.area)
    }

    fn value(&self, _: u32, row: u32, column: u32) -> CacheItem {
        let cell = self.cells.get(address(column, row));
        let formula = cell
            .and_then(Json::as_str)
            .is_some_and(|t| t.starts_with('='));
        if self.formulas_blank && formula {
            return CacheItem::Missing;
        }
        cache_item(cell)
    }
}

/// The cache Excel's refresh builds from `source` (`Data!A1:H5`), read by the engine's own
/// refresh (`read_source`, with Excel's item identity): items in first-seen order, as the case
/// builder lists them (`pivot_definition.py` `_Field.items`). A formula in the source panics:
/// the case file holds no evaluated value for it.
pub(crate) fn source_cache(case: &Json, source: &str) -> PivotCache {
    case_cache(case, source, false)
}

/// The cache the case builder saves for `source` (`pivot_definition.py`): [`source_cache`] with
/// each formula cell, header or record, read as empty, and each field named after its header
/// cell's text, a formula header under its formula, or `names` when the spec gives them
/// (`cache_names`). Excel's refresh on open renames such fields and appends the formula cells'
/// items.
pub(crate) fn builder_cache(case: &Json, source: &str, names: Option<&Json>) -> PivotCache {
    let mut cache = case_cache(case, source, true);
    let (sheet_name, range) = source.split_once('!').unwrap();
    let (left, top) = position(range.split_once(':').unwrap().0);
    let cells = &sheet(case, sheet_name)["cells"];
    for (column, field) in (left..).zip(&mut cache.fields) {
        let given = names.map(|n| &n[(column - left) as usize]);
        let header = given.unwrap_or(&cells[address(column, top)]);
        field.name = header.as_str().unwrap().to_string();
    }
    cache
}

fn case_cache(case: &Json, source: &str, formulas_blank: bool) -> PivotCache {
    let (name, range) = source.split_once('!').unwrap();
    let (from, to) = range.split_once(':').unwrap();
    let ((left, top), (right, bottom)) = (position(from), position(to));
    let cells = CaseCells {
        cells: &sheet(case, name)["cells"],
        area: SourceArea {
            sheet: 0,
            area: CellArea {
                top,
                left,
                bottom,
                right,
            },
        },
        formulas_blank,
    };
    let prior = PivotCache {
        source: Some(CacheSource {
            sheet_name: name.to_string(),
            range: range.to_string(),
        }),
        ..PivotCache::default()
    };
    read_source(&prior, &cells, MAX_RECORD_VALUES)
        .unwrap()
        .finish()
}

pub(crate) fn aggregate_named(name: &str) -> Aggregate {
    Aggregate::from_consolidate_name(name).unwrap_or_else(|| panic!("unknown aggregate {name}"))
}

/// A custom subtotal line's function by its `item@t`, as the importer reads it.
pub(crate) fn subtotal_named(name: &str) -> Aggregate {
    match name {
        "countA" => Aggregate::Count,
        "count" => Aggregate::CountNums,
        "avg" => Aggregate::Average,
        "stdDevP" => Aggregate::StdDevP,
        "varP" => Aggregate::VarP,
        other => aggregate_named(other),
    }
}

/// A field's cache item equal to `value`.
pub(crate) fn item_of(cache: &PivotCache, field: u32, value: &Json) -> u32 {
    let wanted = cache_item(Some(value));
    let items = &cache.fields[field as usize].items;
    u32::try_from(items.iter().position(|i| *i == wanted).unwrap()).unwrap()
}

/// The pivot the case builder writes for the definition of sheet `name`'s pivot.
pub(crate) fn probe_pivot(case: &Json, name: &str) -> (PivotCache, PivotTable) {
    let spec = &sheet(case, name)["pivot_tables"][0];
    probe_pivot_over(
        case,
        name,
        source_cache(case, spec["source"].as_str().unwrap()),
    )
}

/// [`probe_pivot`] over `cache`, a cache of the pivot's source read some other way (the
/// engine's own refresh over a [`probe_model`], whose formulas it evaluates).
pub(crate) fn probe_pivot_over(
    case: &Json,
    name: &str,
    cache: PivotCache,
) -> (PivotCache, PivotTable) {
    let spec = &sheet(case, name)["pivot_tables"][0];
    let definition = &spec["definition"];
    // The builder writes each calculated field after the source fields, with its formula and
    // `databaseField="0"` (`pivot_definition.py` `cache_definition`).
    let mut cache = cache;
    for calculated in definition["calculated"].as_array().into_iter().flatten() {
        cache.fields.push(CacheField {
            name: calculated["name"].as_str().unwrap().to_string(),
            formula: Some(calculated["formula"].as_str().unwrap().to_string()),
            ..CacheField::default()
        });
    }
    let cache = cache;
    let field = |name: &str| {
        u32::try_from(cache.fields.iter().position(|f| f.name == name).unwrap()).unwrap()
    };
    let names = |key: &str| -> Vec<String> {
        definition[key].as_array().map_or_else(Vec::new, |a| {
            a.iter()
                .map(|v| {
                    v.as_str()
                        .unwrap_or_else(|| v["field"].as_str().unwrap())
                        .to_string()
                })
                .collect()
        })
    };
    let axis = |names: &[String]| -> Vec<AxisField> {
        names
            .iter()
            .map(|n| match n.as_str() {
                "Values" => AxisField::Values,
                n => AxisField::Field(field(n)),
            })
            .collect()
    };
    // The anchor: the builder's `location` is the body's top-left cell (page fields above it).
    let (left, top) = position(spec["location"].as_str().unwrap());
    let area = CellArea {
        top,
        left,
        bottom: top,
        right: left,
    };
    let mut table = PivotTable::new(name.to_string(), 1, area, 0);
    table.row_fields = axis(&names("rows"));
    table.column_fields = axis(&names("columns"));
    let data = definition["data"].as_array().unwrap();
    if data.len() > 1
        && !table.row_fields.contains(&AxisField::Values)
        && !table.column_fields.contains(&AxisField::Values)
    {
        table.column_fields.push(AxisField::Values);
    }
    let pages = names("pages");
    for (index, cache_field) in (0u32..).zip(&cache.fields) {
        let on = |list: &[AxisField]| list.contains(&AxisField::Field(index));
        let axis = if on(&table.row_fields) {
            Some(Axis::Row)
        } else if on(&table.column_fields) {
            Some(Axis::Column)
        } else if pages.contains(&cache_field.name) {
            Some(Axis::Page)
        } else {
            None
        };
        let mut pivot_field = PivotField::new(axis, cache_field.items.len());
        // The builder writes `showAll="0"` on every field (`pivot_definition.py`), and refuses
        // it in `attrs` today; a spec that sets it is read as given.
        let attrs = &definition["fields"][&cache_field.name]["attrs"];
        pivot_field.show_all = attrs["showAll"].as_bool().unwrap_or(false);
        let spec = &definition["fields"][&cache_field.name];
        if axis.is_some() {
            // The builder lists the field's `item_order` (the `<item>` elements alone) or its
            // `order` (the cache and the items) first, then the rest in first-seen order.
            let listed = spec.get("item_order").unwrap_or(&spec["order"]);
            let first: Vec<u32> = listed
                .as_array()
                .into_iter()
                .flatten()
                .map(|value| item_of(&cache, index, value))
                .collect();
            let rest = (0..u32::try_from(cache_field.items.len()).unwrap())
                .filter(|item| !first.contains(item));
            pivot_field.items = first
                .iter()
                .copied()
                .chain(rest)
                .map(PivotItem::data)
                .collect();
        }
        for hidden in spec["hidden"].as_array().into_iter().flatten() {
            pivot_field.hide(item_of(&cache, index, hidden));
        }
        // `collapsed` items are written `sd="0"`.
        for collapsed in spec["collapsed"].as_array().into_iter().flatten() {
            let item = item_of(&cache, index, collapsed);
            for listed in &mut pivot_field.items {
                if listed.index == Some(item) {
                    listed.expanded = false;
                }
            }
        }
        if let Some(subtotals) = spec["subtotals"].as_array() {
            pivot_field.default_subtotal = false;
            pivot_field.custom_subtotals = subtotals
                .iter()
                .map(|s| subtotal_named(s.as_str().unwrap()))
                .collect();
        }
        table.fields.push(pivot_field);
    }
    for page in definition["pages"].as_array().into_iter().flatten() {
        let index = field(page["field"].as_str().unwrap());
        table.page_fields.push(PageField {
            field: index,
            item: page.get("item").map(|item| item_of(&cache, index, item)),
            hierarchy: Some(-1),
        });
    }
    table.data_fields = data
        .iter()
        .map(|d| {
            let (name, subtotal) = (d["field"].as_str().unwrap(), d["subtotal"].as_str());
            let subtotal = subtotal.unwrap_or("sum");
            DataField {
                caption: d["name"].as_str().map_or_else(
                    || format!("{} of {name}", aggregate_named(subtotal).word()),
                    str::to_string,
                ),
                field: field(name),
                function: aggregate_named(subtotal),
                number_format: d["numFmtId"].as_u64().map(|id| u32::try_from(id).unwrap()),
                // The builder writes `baseField="0" baseItem="0"` unless the spec sets them, and
                // an x14 mode as the extension's `pivotShowAs` (`pivot_definition.rs`).
                show_data_as: d["showDataAs"].as_str().map_or(ShowDataAs::Normal, |mode| {
                    ShowDataAs::from_name(mode).unwrap_or_else(|| panic!("showDataAs {mode}"))
                }),
                base_field: match &d["baseField"] {
                    Json::String(name) => i32::try_from(field(name)).unwrap(),
                    other => i32::try_from(other.as_i64().unwrap_or(0)).unwrap(),
                },
                base_item: u32::try_from(d["baseItem"].as_u64().unwrap_or(0)).unwrap(),
            }
        })
        .collect();
    let options = &definition["table"];
    let text = |key: &str| options[key].as_str().map(str::to_string);
    table.options.show_error = options["showError"].as_bool().unwrap_or(false);
    table.options.error_caption = text("errorCaption");
    table.options.missing_caption = text("missingCaption");
    table.options.row_header_caption = text("rowHeaderCaption");
    table.options.column_header_caption = text("colHeaderCaption");
    table.options.grand_total_caption = text("grandTotalCaption");
    // `dataCaption="Values"` unless `table` sets it (`_TABLE_DEFAULTS`).
    table.options.data_caption = text("dataCaption").or_else(|| Some("Values".to_string()));
    // The form flags as the builder writes them: `outline` and `outlineData` on unless `table`
    // sets them (`_TABLE_DEFAULTS`), the rest at the schema's default unless set.
    let flag = |value: &Json, default: bool| value.as_bool().unwrap_or(default);
    let o = &mut table.options;
    o.show_missing = flag(&options["showMissing"], o.show_missing);
    o.compact = flag(&options["compact"], o.compact);
    o.outline = flag(&options["outline"], true);
    o.compact_data = flag(&options["compactData"], o.compact_data);
    o.outline_data = flag(&options["outlineData"], true);
    o.row_grand_totals = flag(&options["rowGrandTotals"], o.row_grand_totals);
    o.column_grand_totals = flag(&options["colGrandTotals"], o.column_grand_totals);
    o.page_over_then_down = flag(&options["pageOverThenDown"], o.page_over_then_down);
    o.grid_drop_zones = flag(&options["gridDropZones"], o.grid_drop_zones);
    if let Some(wrap) = options["pageWrap"].as_u64() {
        o.page_wrap = u32::try_from(wrap).unwrap();
    }
    for (cache_field, pivot_field) in cache.fields.iter().zip(&mut table.fields) {
        let attrs = &definition["fields"][&cache_field.name]["attrs"];
        pivot_field.compact = flag(&attrs["compact"], pivot_field.compact);
        pivot_field.outline = flag(&attrs["outline"], pivot_field.outline);
        pivot_field.subtotal_top = flag(&attrs["subtotalTop"], pivot_field.subtotal_top);
        pivot_field.insert_blank_row = flag(&attrs["insertBlankRow"], pivot_field.insert_blank_row);
        pivot_field.subtotal_caption = attrs["subtotalCaption"].as_str().map(str::to_string);
        // The field's raw `extLst` (`xml.extLst`), whose x14 `fillDownLabels` the importer types.
        // The match is deliberately narrow: the probes write the flag as `fillDownLabels="1"`
        // only (t4129 X14F, t4698); the importer's own reading is tested in
        // crates/rustsheet-xlsx/src/import/pivots/table/tests.rs.
        let ext = definition["fields"][&cache_field.name]["xml"]["extLst"].as_str();
        pivot_field.fill_down_labels = ext.is_some_and(|x| x.contains(r#"fillDownLabels="1""#));
    }
    (cache, table)
}

/// A model holding the case's sheets, in order, their cells and their number formats: a text
/// starting with `=` is a formula, evaluated by [`Model::evaluate`] like any other.
pub(crate) fn probe_model(case: &Json) -> Model {
    let mut model = Model::new_empty("probe", "en", "UTC", "en").unwrap();
    for (index, spec) in (0u32..).zip(case["sheets"].as_array().unwrap()) {
        let name = spec["name"].as_str().unwrap();
        if index == 0 {
            model.rename_sheet_by_index(0, name).unwrap();
        } else {
            model.add_sheet(name).unwrap();
        }
        for (at, value) in spec["cells"].as_object().unwrap() {
            let (column, row) = position(at);
            let (row, column) = (
                i32::try_from(row + 1).unwrap(),
                i32::try_from(column + 1).unwrap(),
            );
            match value {
                Json::String(text) => model.set_user_input(index, row, column, text.clone()),
                Json::Number(n) => {
                    model.update_cell_with_number(index, row, column, n.as_f64().unwrap())
                }
                Json::Bool(b) => model.update_cell_with_bool(index, row, column, *b),
                // A typed error cell, `{"t": "e", "v": "#N/A"}`, as its literal.
                Json::Object(typed) if typed["t"] == "e" => {
                    model.set_user_input(index, row, column, typed["v"].as_str().unwrap().into())
                }
                other => panic!("case cell {at}: {other}"),
            }
            .unwrap();
        }
        // The sheet's `number_formats`, so a refresh reads a date-formatted source as dates.
        for (at, code) in spec["number_formats"].as_object().into_iter().flatten() {
            let (column, row) = position(at);
            let (row, column) = (
                i32::try_from(row + 1).unwrap(),
                i32::try_from(column + 1).unwrap(),
            );
            let mut style = model.get_style_for_cell(index, row, column).unwrap();
            style.num_fmt = code.as_str().unwrap().to_string();
            model.set_cell_style(index, row, column, &style).unwrap();
        }
    }
    model
}

/// A diagonal pivot: record `i` is row item `i` and column item `i`, Sum of its index.
pub(crate) fn diagonal(n: u32, data: bool) -> (PivotCache, PivotTable) {
    let mut builder = CacheBuilder::new(None, MAX_RECORD_VALUES);
    for name in ["R", "C", "V"] {
        builder.add_field(name.to_string()).unwrap();
    }
    for i in 0..n {
        builder
            .push_record(vec![
                RecordValue::Inline(CacheItem::Number(f64::from(i))),
                RecordValue::Inline(CacheItem::Number(f64::from(i))),
                RecordValue::Inline(CacheItem::Number(f64::from(i))),
            ])
            .unwrap();
    }
    let cache = builder.finish();
    let area = CellArea {
        top: 0,
        left: 0,
        bottom: 0,
        right: 0,
    };
    let mut table = PivotTable::new("D".to_string(), 1, area, 0);
    let len = |f: usize| cache.fields[f].items.len();
    table.fields = vec![
        PivotField::new(Some(Axis::Row), len(0)),
        PivotField::new(Some(Axis::Column), len(1)),
        PivotField::new(None, len(2)),
    ];
    table.row_fields = vec![AxisField::Field(0)];
    table.column_fields = vec![AxisField::Field(1)];
    if data {
        table.data_fields = vec![DataField {
            field: 2,
            ..DataField::default()
        }];
    }
    (cache, table)
}

/// The cells `table` over `cache` renders on the day `today` (`compute_on`), by address.
fn rendered(
    table: &PivotTable,
    cache: &PivotCache,
    today: Option<f64>,
) -> HashMap<String, PivotCell> {
    let mut work = MAX_PIVOT_WORK;
    let result = compute_on(table, cache, today, &mut work).unwrap();
    let layout = lay_out(table, cache, &result, &mut work).unwrap();
    let mut cells = HashMap::new();
    for cell in render(table, cache, &result, &layout).unwrap() {
        let at = address(cell.column, cell.row);
        assert!(
            cells.insert(at.clone(), cell.value).is_none(),
            "{at} rendered twice"
        );
    }
    cells
}

/// A sidecar cell as a rendered one; `None` for an empty cell.
fn expected(cell: &Json) -> Option<PivotCell> {
    match cell["t"].as_str().unwrap() {
        "empty" => None,
        "s" => Some(PivotCell::Text(cell["v"].as_str().unwrap().to_string())),
        "n" => Some(PivotCell::Number(cell["v"].as_f64().unwrap())),
        "e" => Some(PivotCell::Error(
            get_error_by_english_name(cell["v"].as_str().unwrap()).unwrap(),
        )),
        other => panic!("cell type {other}"),
    }
}

/// The cells of sheet `name`'s pivot (`table` over `cache`, rendered on the day `today`) that are
/// not the ones Excel saved in probe `stem`'s sidecar, but the cells `skip`. Every sidecar cell
/// is compared, an empty one included, and every rendered cell must lie in the sidecar's capture
/// window.
pub(crate) fn sidecar_differences(
    stem: &str,
    name: &str,
    cache: &PivotCache,
    table: &PivotTable,
    today: Option<f64>,
    skip: &[&str],
) -> Vec<String> {
    let oracle = read(stem, "oracle");
    let mut actual = rendered(table, cache, today);
    let pivot = oracle["pivots"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["sheet"] == name)
        .unwrap();
    let mut wrong = Vec::new();
    for (at, cell) in pivot["cells"].as_object().unwrap() {
        let mine = actual.remove(at);
        if skip.contains(&at.as_str()) {
            continue;
        }
        let excel = expected(cell);
        if mine != excel {
            wrong.push(format!("{at}: Excel {excel:?}, engine {mine:?}"));
        }
    }
    for (at, cell) in actual {
        wrong.push(format!("{at}: outside the capture window, engine {cell:?}"));
    }
    wrong.sort();
    wrong
        .into_iter()
        .map(|w| format!("{stem} {name} {w}"))
        .collect()
}
