#![allow(clippy::unwrap_used, clippy::panic, reason = "tests")]
//! The grid writer against probes/t4128-pivot-spike: R, a pivot whose source holds a formula,
//! re-rendered by a refresh after the recalculation, and S, whose layout would meet the constant
//! S!C5, collapsed to `#SPILL!` at its anchor. The definitions come from the case file, the
//! sheets are the case's cells in a real model (its formulas evaluated), and the expectations
//! are the sidecar's `pivots`, never the engine's. The rest pins the engine's own readings
//! (pivots.md "The grid writer's own readings", UNVERIFIED, T-4664).

use serde_json::Value as Json;

use super::*;
use crate::pivot::probe_cases::{
    address, builder_cache, position, probe_model, probe_pivot, probe_pivot_over, read, sheet,
};
use crate::pivot::{
    Axis, CacheField, CacheItem, CacheSource, OpaqueEntry, OpaqueNode, OpaqueOwner,
};

const SPIKE: &str = "t4128-pivot-spike";

fn sheet_of(model: &Model, name: &str) -> u32 {
    model.get_sheet_index_by_name(name).unwrap()
}

/// The value cell `at` (A1) of sheet `sheet` holds, as the renderer types one; `None` when empty.
fn cell(model: &Model, sheet: u32, at: &str) -> Option<PivotCell> {
    let (column, row) = position(at);
    let worksheet = model.workbook.worksheet(sheet).unwrap();
    let (row, column) = (
        i32::try_from(row + 1).unwrap(),
        i32::try_from(column + 1).unwrap(),
    );
    match worksheet.cell(row, column)? {
        Cell::EmptyCell { .. } => None,
        Cell::NumberCell { v, .. } => Some(PivotCell::Number(*v)),
        Cell::ErrorCell { ei, .. } => Some(PivotCell::Error(ei.clone())),
        Cell::SharedString { si, .. } => Some(PivotCell::Text(
            model.workbook.shared_strings[usize::try_from(*si).unwrap()].clone(),
        )),
        other => panic!("{at}: {other:?}"),
    }
}

/// Every stored, non-empty cell of sheet `sheet`, as sorted A1 addresses.
fn stored(model: &Model, sheet: u32) -> Vec<String> {
    let stored = model.stored_in(sheet, &area("A1:Z100"), &mut PivotBudget::default());
    let mut cells: Vec<String> = (stored.unwrap().into_iter())
        .map(|((row, column), _)| address(column, row))
        .collect();
    cells.sort();
    cells
}

fn area(range: &str) -> CellArea {
    let (from, to) = range.split_once(':').unwrap_or((range, range));
    let ((left, top), (right, bottom)) = (position(from), position(to));
    CellArea {
        top,
        left,
        bottom,
        right,
    }
}

/// Checks sheet `name` of `model` against the spike sidecar's pivot there ([`assert_probe`]).
fn assert_excels(model: &Model, name: &str) {
    assert_probe(model, SPIKE, name);
}

/// Checks sheet `name` of `model` against the pivot there in the sidecar of probe `stem`: every
/// cell of its capture window, the location and the geometry of the model's first pivot.
fn assert_probe(model: &Model, stem: &str, name: &str) {
    let oracle = read(stem, "oracle");
    let pivots = oracle["pivots"].as_array().unwrap();
    let pivot: &Json = pivots.iter().find(|p| p["sheet"] == name).unwrap();
    let sheet = sheet_of(model, name);
    let mut wrong = Vec::new();
    for (at, expected) in pivot["cells"].as_object().unwrap() {
        let value = &expected["v"];
        let expected = match expected["t"].as_str().unwrap() {
            "empty" => None,
            "s" => Some(PivotCell::Text(value.as_str().unwrap().into())),
            "n" => Some(PivotCell::Number(value.as_f64().unwrap())),
            "e" if value == "#SPILL!" => Some(PivotCell::Error(Error::SPILL)),
            other => panic!("{other} {value}"),
        };
        let mine = cell(model, sheet, at);
        if mine != expected {
            wrong.push(format!("{at}: Excel {expected:?}, engine {mine:?}"));
        }
    }
    assert!(wrong.is_empty(), "{name}:\n{}", wrong.join("\n"));
    let table = &model.workbook.pivot_tables[0];
    assert_eq!(table.location, area(pivot["location"].as_str().unwrap()));
    let geometry = &table.geometry;
    let saved = ["first_header_row", "first_data_row", "first_data_col"]
        .map(|key| u32::try_from(pivot[key].as_u64().unwrap()).unwrap());
    assert_eq!(
        [
            geometry.first_header_row,
            geometry.first_data_row,
            geometry.first_data_column
        ],
        saved
    );
}

/// The spike's cells in a model holding sheet `name`'s pivot ([`probe_in_model`]).
fn spike(name: &str) -> Model {
    probe_in_model(SPIKE, name)
}

/// Probe `stem`'s cells in a model holding sheet `name`'s pivot, its cache read by the engine's
/// refresh from the cells as they are before any recalculation.
fn probe_in_model(stem: &str, name: &str) -> Model {
    let case = read(stem, "case");
    let mut model = probe_model(&case);
    let spec = &sheet(&case, name)["pivot_tables"][0];
    let (sheet_name, range) = spec["source"].as_str().unwrap().split_once('!').unwrap();
    let empty = PivotCache {
        source: Some(CacheSource {
            sheet_name: sheet_name.into(),
            range: range.into(),
        }),
        ..PivotCache::default()
    };
    let stale = refresh(&empty, &model, MAX_RECORD_VALUES).unwrap();
    let (cache, mut table) = probe_pivot_over(&case, name, stale);
    let index = sheet_of(&model, name);
    table.sheet_id = model.workbook.worksheet(index).unwrap().sheet_id;
    model.workbook.pivot_caches.push(cache);
    model.workbook.pivot_tables.push(table);
    model
}

fn refreshed(model: &mut Model) -> PivotSwap {
    let plan = model
        .plan_pivot_refresh(0, &mut PivotBudget::default())
        .unwrap();
    model.apply_pivot_refresh(plan).unwrap()
}

#[test]
fn the_spike_r_pivot_re_renders_from_the_recalculated_source() {
    let mut model = spike("R");
    // DataF!C2 is `=1000`, which the file's cache records as missing (`<m/>`): only a refresh
    // reads it (Excel's R!B5 is 1004).
    let sales = &mut model.workbook.pivot_caches[0].fields[2];
    sales.items[0] = CacheItem::Missing;
    model.evaluate();
    refreshed(&mut model);
    assert_excels(&model, "R");
}

#[test]
fn the_spike_s_pivot_collapses_to_spill_at_its_anchor() {
    let mut model = spike("S");
    model.evaluate();
    let plan = model
        .plan_pivot_refresh(0, &mut PivotBudget::default())
        .unwrap();
    assert!(plan.placements[0].spilled);
    model.apply_pivot_refresh(plan).unwrap();
    assert_excels(&model, "S");
    // The foreign constant stays, and the pivot wrote nothing else.
    let s = sheet_of(&model, "S");
    assert_eq!(stored(&model, s), ["A3", "C5"]);
    // Laid out again from its one-cell location it still meets C5; without C5 it renders
    // from the same anchor.
    refreshed(&mut model);
    assert_excels(&model, "S");
    (model.workbook.worksheet_mut(s).unwrap())
        .cell_clear_contents(5, 3)
        .unwrap();
    refreshed(&mut model);
    assert_eq!(model.workbook.pivot_tables[0].location, area("A3:E7"));
    assert_eq!(cell(&model, s, "C5"), Some(PivotCell::Number(5.0)));
}

#[test]
fn a_refresh_clears_what_the_pivot_owned_and_grows_only_into_empty_cells() {
    let mut model = spike("R");
    model.evaluate();
    refreshed(&mut model);
    let (r, data) = (sheet_of(&model, "R"), sheet_of(&model, "DataF"));
    // A constant left in the pivot's own area (D6, a blank intersection) is cleared; one away
    // from it is kept.
    model.update_cell_with_text(r, 6, 4, "stale").unwrap();
    model.update_cell_with_text(r, 20, 1, "away").unwrap();
    // A source edit changes nothing until the next refresh.
    model.update_cell_with_number(data, 3, 3, 25.0).unwrap();
    model.evaluate();
    assert_eq!(cell(&model, r, "B6"), Some(PivotCell::Number(20.0)));
    refreshed(&mut model);
    assert_eq!(cell(&model, r, "D6"), None);
    assert_eq!(cell(&model, r, "B6"), Some(PivotCell::Number(25.0)));
    assert_eq!(cell(&model, r, "A20"), Some(PivotCell::Text("away".into())));
    // A new product grows the pivot by a column, into empty cells.
    model.update_cell_with_text(data, 3, 2, "D").unwrap();
    model.evaluate();
    refreshed(&mut model);
    assert_eq!(model.workbook.pivot_tables[0].location, area("A3:F7"));
    assert_eq!(cell(&model, r, "E4"), Some(PivotCell::Text("D".into())));
    // A constant where it would grow next collapses it, and its old area is emptied.
    model.update_cell_with_text(data, 4, 2, "E").unwrap();
    model.update_cell_with_text(r, 4, 7, "x").unwrap();
    model.evaluate();
    refreshed(&mut model);
    assert_eq!(model.workbook.pivot_tables[0].location, area("A3"));
    assert_eq!(cell(&model, r, "A3"), Some(PivotCell::Error(Error::SPILL)));
    assert_eq!(stored(&model, r), ["A20", "A3", "G4"]);
}

#[test]
fn a_swap_restores_the_cache_and_pivot() {
    let mut model = spike("R");
    model.evaluate();
    let (cache, table) = (
        model.workbook.pivot_caches[0].clone(),
        model.workbook.pivot_tables[0].clone(),
    );
    let data = sheet_of(&model, "DataF");
    model.update_cell_with_number(data, 3, 3, 25.0).unwrap();
    model.evaluate();
    let swap = refreshed(&mut model);
    assert_ne!(model.workbook.pivot_caches[0], cache);
    model.restore_pivot_swap(swap);
    assert_eq!(model.workbook.pivot_caches[0], cache);
    assert_eq!(model.workbook.pivot_tables[0], table);
}

#[test]
fn another_pivots_area_is_foreign_even_where_it_is_blank() {
    let mut model = spike("R");
    model.evaluate();
    refreshed(&mut model);
    // A second pivot over the same cache, anchored on R's blank D6.
    let mut second = model.workbook.pivot_tables[0].clone();
    second.name = "Second".into();
    second.location = area("D6");
    second.geometry = PivotGeometry::default();
    model.workbook.pivot_tables.push(second);
    let plan = model
        .plan_pivot_render(1, &mut PivotBudget::default())
        .unwrap();
    assert!(plan.placements[0].spilled);
    // A refresh of the cache lays out both, each meeting the other's area.
    let plan = model
        .plan_pivot_refresh(0, &mut PivotBudget::default())
        .unwrap();
    assert_eq!(plan.placements.len(), 2);
    assert!(plan.placements.iter().all(|p| p.spilled));
}

#[test]
fn a_pivot_moved_by_a_row_insert_refreshes_where_it_now_is() {
    let mut model = spike("R");
    model.evaluate();
    refreshed(&mut model);
    let r = sheet_of(&model, "R");
    // T-2086: the pivot moves with its cells.
    model.insert_rows(r, 1, 2).unwrap();
    assert_eq!(model.workbook.pivot_tables[0].location, area("A5:E9"));
    let plan = model
        .plan_pivot_refresh(0, &mut PivotBudget::default())
        .unwrap();
    assert!(!plan.placements[0].spilled);
    model.apply_pivot_refresh(plan).unwrap();
    assert_eq!(model.workbook.pivot_tables[0].location, area("A5:E9"));
    assert_eq!(cell(&model, r, "B7"), Some(PivotCell::Number(1004.0)));
    assert_eq!(cell(&model, r, "A3"), None);
    // An insert through it is still refused.
    assert!(model.insert_rows(r, 7, 1).is_err());
}

#[test]
fn the_page_area_is_owned_with_the_location() {
    let geometry = PivotGeometry {
        row_page_count: 2,
        column_page_count: 2,
        ..PivotGeometry::default()
    };
    assert_eq!(
        owned_areas(area("B10:C12"), &geometry),
        [Some(area("B10:C12")), Some(area("B7:F8"))]
    );
    assert_eq!(owned_areas(area("A2"), &geometry), [Some(area("A2")), None]);
}

fn cache_of(names: &[&str]) -> PivotCache {
    PivotCache {
        fields: (names.iter())
            .map(|name| CacheField {
                name: (*name).into(),
                ..CacheField::default()
            })
            .collect(),
        ..PivotCache::default()
    }
}

fn element(xml: &str) -> OpaqueNode {
    OpaqueNode::Element(xml.into())
}

fn attributes(xml: &str) -> OpaqueNode {
    OpaqueNode::Attributes(xml.into())
}

const STYLE: &str = r#"<pivotTableStyleInfo name="PivotStyleLight16" showRowHeaders="1"/>"#;
/// `x@v` and `i@i` are item and data field indices (ECMA-376 §18.10.1.96), not field indices.
const ROW_ITEMS: &str =
    r#"<rowItems count="2"><i><x v="1"/></i><i t="grand" i="0"><x/></i></rowItems>"#;
const FILTERS: &str = r#"<filters count="1"><filter fld="0" type="count" evalOrder="-1" id="1" iMeasureFld="0"><autoFilter ref="A1"><filterColumn colId="0"><top10 val="2" filterVal="2"/></filterColumn></autoFilter></filter></filters>"#;
/// A value in quotes (`'>'`) and a prefix do not hide the `reference@field`.
const FORMATS: &str = r#"<formats count="1"><format dxfId="0"><pivotArea type="data" outline="0" fieldPosition="0" x:note='a>b'><references count="1"><x:reference FIELD = "1" count="1"><x v="0"/></x:reference></references></pivotArea></format></formats>"#;
const SORT_SCOPE: &str = r#"<autoSortScope><pivotArea dataOnly="0" outline="0" field="2" axis="axisRow"/></autoSortScope>"#;

#[test]
fn which_unread_parts_hold_field_indices() {
    let kept = |owner, node| OpaqueEntry { owner, node }.holds_field_index();
    let definition = OpaqueOwner::Definition;
    assert!(!kept(definition, element(STYLE)));
    assert!(!kept(definition, element(ROW_ITEMS)));
    assert!(!kept(
        definition,
        element(r#"<format dxfId="0"><pivotArea type="all" fieldPosition="0"/></format>"#)
    ));
    // A field name in a value or between tags is not an attribute.
    assert!(!kept(
        definition,
        element(r#"<ext uri="field=1">fld="2"</ext>"#)
    ));
    assert!(kept(definition, element(FILTERS)));
    assert!(kept(definition, element(FORMATS)));
    assert!(kept(OpaqueOwner::Field(0), element(SORT_SCOPE)));
    // `pivotField@dataField` is a flag; anywhere else the name counts.
    assert!(!kept(
        OpaqueOwner::Field(0),
        attributes(r#" dataField="1" rankBy="0""#)
    ));
    assert!(kept(
        OpaqueOwner::DataField(0),
        attributes(r#" dataField="1""#)
    ));
    assert!(kept(definition, attributes(r#" x14:sourceField="1""#)));
    assert!(!kept(
        definition,
        attributes(r#" fieldPrintTitles="1" disableFieldList="0""#)
    ));
    assert!(!kept(
        OpaqueOwner::Field(0),
        attributes(r#" serverField="1""#)
    ));
    // A comment, CDATA section or processing instruction is skipped to its end, so a quote in
    // it hides nothing after it; one left open, an open quote or tag, or a declaration refuses.
    let reference = r#"<reference field="1"/>"#;
    for hider in ["<!-- don't -->", "<![CDATA[ \" ]]>", "<?pi ' ?>"] {
        assert!(kept(definition, element(&format!("{hider}{reference}"))));
        assert!(!kept(definition, element(&format!("{hider}{STYLE}"))));
    }
    for open in [
        r#"<filter fld=""#,
        r#"<filter type="count""#,
        "<!-- don't",
        "<![CDATA[ x",
        "<?pi",
        "<!DOCTYPE x>",
    ] {
        assert!(kept(definition, element(open)), "{open}");
    }
    assert!(kept(definition, attributes(r#" name="open"#)));
    assert!(kept(definition, attributes(r#" a="1">"#)));
}

#[test]
fn field_indices_follow_their_names() {
    let (_, mut table) = probe_pivot(&read(SPIKE, "case"), "S");
    let old = cache_of(&["Region", "Product", "Sales"]);
    // A new source column before the others, and the names refreshed in another case.
    let new = cache_of(&["Day", "SALES", "region", "Product"]);
    let remapped = remap(&table, &old, &new).unwrap();
    assert_eq!(remapped.row_fields, [AxisField::Field(2)]);
    assert_eq!(remapped.column_fields, [AxisField::Field(3)]);
    assert_eq!(remapped.data_fields[0].field, 1);
    assert_eq!(remapped.fields.len(), 4);
    assert_eq!(remapped.fields[2].axis, table.fields[0].axis);
    assert_eq!(remapped.fields[0].axis, None);
    // A field the pivot shows that left the source refuses the refresh.
    let gone = cache_of(&["Region", "Sales"]);
    assert_eq!(
        remap(&table, &old, &gone),
        Err(PivotRefreshError::FieldGone("Product".into()))
    );
    // Unread parts that hold no field index renumber; a field's own follow it.
    let kept = |owner, node| OpaqueEntry { owner, node };
    table.opaque = vec![
        kept(
            OpaqueOwner::Definition,
            attributes(r#" updatedVersion="6""#),
        ),
        kept(OpaqueOwner::Definition, element(STYLE)),
        kept(OpaqueOwner::Definition, element(ROW_ITEMS)),
        kept(OpaqueOwner::Definition, element("<filters/>")),
        kept(OpaqueOwner::Field(0), attributes(r#" dataField="1""#)),
        kept(OpaqueOwner::Items(1), attributes(r#" x14:extra="1""#)),
        kept(
            OpaqueOwner::Item { field: 2, item: 0 },
            attributes(r#" n="Total""#),
        ),
    ];
    let remapped = remap(&table, &old, &new).unwrap();
    let owners: Vec<OpaqueOwner> = remapped.opaque.iter().map(|e| e.owner).collect();
    assert_eq!(
        owners[4..],
        [
            OpaqueOwner::Field(2),
            OpaqueOwner::Items(3),
            OpaqueOwner::Item { field: 1, item: 0 }
        ]
    );
    assert_eq!(remapped.opaque[..4], table.opaque[..4]);
    // A dropped field's entries go with it.
    let without_sales = cache_of(&["Product", "Region", "Day"]);
    table.data_fields.clear();
    let dropped = remap(&table, &old, &without_sales).unwrap();
    assert_eq!(dropped.opaque.len(), 6);
    let (_, mut table) = probe_pivot(&read(SPIKE, "case"), "S");
    // Unread parts that hold a field index: no renumbering under them.
    for (owner, node) in [
        (OpaqueOwner::Definition, element(FILTERS)),
        (OpaqueOwner::Definition, element(FORMATS)),
        (OpaqueOwner::Field(1), element(SORT_SCOPE)),
        (
            OpaqueOwner::DataField(0),
            attributes(r#" x14:sourceField="2""#),
        ),
    ] {
        table.opaque = vec![kept(owner, node)];
        assert_eq!(
            remap(&table, &old, &new),
            Err(PivotRefreshError::FieldsMoved)
        );
    }
    // Appending a field renumbers nothing.
    let appended = cache_of(&["Region", "Product", "Sales", "Day"]);
    assert_eq!(remap(&table, &old, &appended).unwrap().fields.len(), 4);
}

/// The R pivot rendered once, and the plan of its next refresh with `extra` more copies of it on
/// its cache, each anchored ten columns right of the one before.
fn with_copies(extra: usize, budget: &mut PivotBudget) -> Result<PivotRefresh, PivotRefreshError> {
    let mut model = spike("R");
    model.evaluate();
    refreshed(&mut model);
    let pivot = model.workbook.pivot_tables[0].clone();
    for k in 1..=extra {
        let mut copy = pivot.clone();
        let left = u32::try_from(k * 10).unwrap();
        copy.location = CellArea {
            left,
            right: left,
            ..pivot.location
        };
        copy.geometry = PivotGeometry::default();
        model.workbook.pivot_tables.push(copy);
    }
    model.plan_pivot_refresh(0, budget)
}

#[test]
fn every_pivot_of_a_plan_shares_one_budget() {
    // What one refresh of R spends, to the unit.
    let mut one = PivotBudget::default();
    with_copies(0, &mut one).unwrap();
    let full = PivotBudget::default();
    let (work, cells) = (full.work - one.work, full.cells - one.cells);
    // Ten pivots on the cache spend more than ten times the source read alone: well past what
    // three of them may spend together.
    let mut three = PivotBudget {
        work: work * 3,
        cells: cells * 3,
    };
    assert_eq!(
        with_copies(9, &mut three),
        Err(PivotRefreshError::Compute(PivotComputeError::Limit))
    );
    // What the failed plan spent stays spent.
    assert!(three.work == 0 || three.cells == 0, "{three:?}");
    // The cells written count too, apart from the work.
    let mut few_cells = PivotBudget {
        work: full.work,
        cells: cells * 3,
    };
    assert_eq!(
        with_copies(9, &mut few_cells),
        Err(PivotRefreshError::Compute(PivotComputeError::Limit))
    );
    let mut enough = PivotBudget::default();
    assert_eq!(with_copies(9, &mut enough).unwrap().placements.len(), 10);
}

/// The R pivot's refresh plan, laid out from A3 again, after `block` changes the R sheet.
fn r_blocked(block: impl FnOnce(&mut Model, u32)) -> PivotRefresh {
    let mut model = spike("R");
    model.evaluate();
    let r = sheet_of(&model, "R");
    block(&mut model, r);
    model
        .plan_pivot_refresh(0, &mut PivotBudget::default())
        .unwrap()
}

#[test]
fn merged_cells_tables_array_formulas_and_the_sheet_edge_collapse_it() {
    // An empty merged range inside the layout (D5:D6).
    let merged = r_blocked(|model, r| {
        let ws = model.workbook.worksheet_mut(r).unwrap();
        ws.merged_cells.push(crate::types::MergedCell {
            row: 5,
            column: 4,
            width: 1,
            height: 2,
        });
    });
    assert!(merged.placements[0].spilled);
    // A Table whose header D2 is above the layout and whose body D3:D9 is empty.
    let table = r_blocked(|model, r| {
        model.update_cell_with_text(r, 2, 4, "Head").unwrap();
        model.add_table(r, "Blocker", (2, 4, 9, 4), false).unwrap();
    });
    assert!(table.placements[0].spilled);
    // An array formula's cell inside the layout (G is outside it, so it alone does not block).
    let array = r_blocked(|model, r| {
        model
            .set_user_input(r, 6, 3, "=SEQUENCE(2)".into())
            .unwrap();
        model.evaluate();
    });
    assert!(array.placements[0].spilled);
    // A layout past the sheet's last row: the anchor at the last row but two.
    let edge = r_blocked(|model, _| {
        let last = u32::try_from(crate::constants::LAST_ROW).unwrap() - 1;
        let location = &mut model.workbook.pivot_tables[0].location;
        (location.top, location.bottom) = (last - 2, last - 2);
        location.right = location.left;
    });
    let placement = &edge.placements[0];
    assert!(placement.spilled);
    assert_eq!(placement.cells.len(), 1);
    assert_eq!(placement.cells[0].value, PivotCell::Error(Error::SPILL));
}

#[test]
fn a_cleared_area_skips_array_formula_cells() {
    let mut model = spike("R");
    model.evaluate();
    refreshed(&mut model);
    let r = sheet_of(&model, "R");
    // A constant on the pivot's blank D6 is the pivot's to clear; an array formula's cells in
    // its saved area, outside the new layout, are not.
    model.update_cell_with_text(r, 6, 4, "stale").unwrap();
    let plan = model
        .plan_pivot_refresh(0, &mut PivotBudget::default())
        .unwrap();
    assert_eq!(plan.placements[0].cleared, [(5, 3)]);
    model
        .set_user_input(r, 9, 1, "=SEQUENCE(1,2)".into())
        .unwrap();
    model.update_cell_with_text(r, 6, 4, "").unwrap();
    model.evaluate();
    let ws = model.workbook.worksheet_mut(r).unwrap();
    ws.cell_clear_contents(6, 4).unwrap();
    // The saved area stretched by hand over A9:B9, where the array spills.
    model.workbook.pivot_tables[0].location.bottom = 8;
    let plan = model
        .plan_pivot_refresh(0, &mut PivotBudget::default())
        .unwrap();
    let placement = &plan.placements[0];
    assert!(!placement.spilled);
    assert!(!placement.cleared.contains(&(8, 0)) && !placement.cleared.contains(&(8, 1)));
}

/// Pivot `index` of `model` with Product alone on `axis` (rows or columns) at `anchor`.
fn product_on(model: &mut Model, index: usize, axis: Axis, anchor: &str) {
    let mut pivot = model.workbook.pivot_tables[0].clone();
    pivot.location = area(anchor);
    pivot.geometry = PivotGeometry::default();
    pivot.fields[0].axis = None;
    pivot.fields[1].axis = Some(axis);
    (pivot.row_fields, pivot.column_fields) = match axis {
        Axis::Row => (vec![AxisField::Field(1)], Vec::new()),
        _ => (Vec::new(), vec![AxisField::Field(1)]),
    };
    pivot.name = format!("P{index}");
    if index < model.workbook.pivot_tables.len() {
        model.workbook.pivot_tables[index] = pivot;
    } else {
        model.workbook.pivot_tables.push(pivot);
    }
}

#[test]
fn pivots_of_one_plan_that_grow_into_each_other_collapse() {
    let mut model = spike("R");
    model.evaluate();
    // Product on columns from A3 (three rows, five columns) and on rows from B1 (five rows, two
    // columns): neither anchor is in the other's layout, but the layouts cross at B3:C5.
    product_on(&mut model, 0, Axis::Column, "A3");
    product_on(&mut model, 1, Axis::Row, "B1");
    for (index, expected) in [(0, "A3:E5"), (1, "B1:C5")] {
        let alone = model
            .plan_pivot_render(index, &mut PivotBudget::default())
            .unwrap();
        assert_eq!(alone.placements[0].pivot.location, area(expected));
    }
    let plan = model
        .plan_pivot_refresh(0, &mut PivotBudget::default())
        .unwrap();
    let [first, second] = &plan.placements[..] else {
        panic!("two placements");
    };
    assert!(!first.spilled);
    assert!(second.spilled);
    assert_eq!(second.cells.len(), 1);
}

#[test]
fn a_merged_range_in_the_pivots_own_area_refuses_the_refresh() {
    let mut model = spike("R");
    model.evaluate();
    refreshed(&mut model);
    let r = sheet_of(&model, "R");
    // A merge of A5:A6, inside the rendered area, as `mergeItem` makes for labels.
    model
        .workbook
        .worksheet_mut(r)
        .unwrap()
        .merged_cells
        .push(crate::types::MergedCell {
            row: 5,
            column: 1,
            width: 1,
            height: 2,
        });
    assert_eq!(
        model.plan_pivot_refresh(0, &mut PivotBudget::default()),
        Err(PivotRefreshError::MergedArea)
    );
}

/// The pivots of probes/t4585-pivot-field-names (N1) and t4627-pivot-field-name-collisions (the
/// P_ sheets) as the case builder saves them ([`builder_cache`]), with the verbatim parts
/// it writes (the root's attributes, the data field's `dataField` flag, `pivotTableStyleInfo`).
/// Excel's refresh on open renames the repeated, formula and blank headers, so the fields
/// renumber; the parts hold no field index, so the pivot renders, as the sidecar has it.
#[test]
fn a_renamed_field_renders_the_pivot_under_its_unread_parts() {
    let probes: [(&str, &[&str]); 2] = [
        ("t4585-pivot-field-names", &["N1"]),
        (
            "t4627-pivot-field-name-collisions",
            &[
                "P_SufA", "P_SufB", "P_FldA", "P_FldB", "P_Off", "P_Empty", "P_Const", "P_Space",
                "P_Fold",
            ],
        ),
    ];
    for (stem, names) in probes {
        let case = read(stem, "case");
        for &name in names {
            let mut model = probe_model(&case);
            model.evaluate();
            let spec = &sheet(&case, name)["pivot_tables"][0];
            let names = spec["definition"].get("cache_names");
            let stale = builder_cache(&case, spec["source"].as_str().unwrap(), names);
            let (cache, mut table) = probe_pivot_over(&case, name, stale);
            let data_field = table.data_fields[0].field;
            table.opaque = vec![
                OpaqueEntry {
                    owner: OpaqueOwner::Definition,
                    node: attributes(
                        r#" applyNumberFormats="0" createdVersion="6" indent="0" itemPrintTitles="1" minRefreshableVersion="3" updatedVersion="6" useAutoFormatting="1""#,
                    ),
                },
                OpaqueEntry {
                    owner: OpaqueOwner::Field(data_field),
                    node: attributes(r#" dataField="1""#),
                },
                OpaqueEntry {
                    owner: OpaqueOwner::Definition,
                    node: element(STYLE),
                },
            ];
            let index = sheet_of(&model, name);
            table.sheet_id = model.workbook.worksheet(index).unwrap().sheet_id;
            model.workbook.pivot_caches.push(cache);
            model.workbook.pivot_tables.push(table);
            refreshed(&mut model);
            assert_probe(&model, stem, name);
            // The flag followed its field to the refreshed cache's same-named one.
            let refreshed = &model.workbook.pivot_tables[0];
            let sales = refreshed.data_fields[0].field;
            assert_eq!(refreshed.opaque[1].owner, OpaqueOwner::Field(sales));
        }
    }
}

#[test]
fn value_cells_show_their_data_fields_number_format() {
    // probes/t4169-pivot-number-formats: every cell of each pivot's area shows Excel's displayed
    // text (the sidecar's `text`), value cells in their data field's `numFmtId`: currency (7),
    // percent (10), date (14), the workbook's custom 164, General (0) and none, on item,
    // subtotal and grand-total lines and the grand-total column.
    const STEM: &str = "t4169-pivot-number-formats";
    let oracle = read(STEM, "oracle");
    for name in ["P1", "P2"] {
        let mut model = probe_in_model(STEM, name);
        // Q!B1 carries the case's one custom format, which the builder writes as numFmt 164.
        let q = sheet_of(&model, "Q");
        let mut style = model.get_style_for_cell(q, 1, 2).unwrap();
        style.num_fmt = "0.0 \"kg\"".into();
        model.set_cell_style(q, 1, 2, &style).unwrap();
        model.evaluate();
        refreshed(&mut model);
        let pivots = oracle["pivots"].as_array().unwrap();
        let pivot = pivots.iter().find(|p| p["sheet"] == name).unwrap();
        let sheet = sheet_of(&model, name);
        let mut wrong = Vec::new();
        for (at, expected) in pivot["text"].as_object().unwrap() {
            let (column, row) = position(at);
            let (row, column) = (
                i32::try_from(row + 1).unwrap(),
                i32::try_from(column + 1).unwrap(),
            );
            let mine = model.get_formatted_cell_value(sheet, row, column).unwrap();
            if mine != expected.as_str().unwrap() {
                wrong.push(format!("{at}: Excel {expected}, engine {mine:?}"));
            }
        }
        assert!(wrong.is_empty(), "{name}:\n{}", wrong.join("\n"));
    }
}

#[test]
fn date_item_labels_show_as_short_dates() {
    // pivots.md "Item casing when cache and source disagree, and date items": a date item's
    // label shows as `m/d/yyyy`, with ` h:mm` when it has a time part, whatever the source
    // format: t2085 D's source is `m/d/yyyy`, t4164's is `m/d/yyyy h:mm` (midnight reads
    // `4/17/2024`, WOe1!A5; 12:00 reads `4/16/2024 12:00`, A4). Only the date labels are
    // checked, on sheets whose filters this harness leaves out without changing the items shown.
    let probes: [(&str, &[&str]); 3] = [
        ("t2085-getpivotdata-items", &["D"]),
        ("t4164-pivot-date-times", &["WOe1"]),
        ("t4130-pivot-date-filters", &["DLit", "DOldEq"]),
    ];
    let mut wrong = Vec::new();
    for (stem, names) in probes {
        let oracle = read(stem, "oracle");
        for &name in names {
            let mut model = probe_in_model(stem, name);
            model.evaluate();
            refreshed(&mut model);
            let pivots = oracle["pivots"].as_array().unwrap();
            let pivot = pivots.iter().find(|p| p["sheet"] == name).unwrap();
            let sheet = sheet_of(&model, name);
            let mut seen = 0;
            for (at, expected) in pivot["text"].as_object().unwrap() {
                let expected = expected.as_str().unwrap();
                if !expected.contains('/') {
                    continue;
                }
                seen += 1;
                let (column, row) = position(at);
                let (row, column) = (
                    i32::try_from(row + 1).unwrap(),
                    i32::try_from(column + 1).unwrap(),
                );
                let mine = model.get_formatted_cell_value(sheet, row, column).unwrap();
                if mine != expected {
                    wrong.push(format!(
                        "{stem} {name}!{at}: Excel {expected}, engine {mine:?}"
                    ));
                }
            }
            assert!(seen > 0, "{stem} {name} has no date label");
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
