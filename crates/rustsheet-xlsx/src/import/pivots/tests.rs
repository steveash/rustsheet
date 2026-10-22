#![allow(clippy::unwrap_used)]

use std::io::{Cursor, Write};

use rustsheet_engine::expressions::token::Error;
use rustsheet_engine::pivot::{MAX_FIELDS, MAX_ITEMS, MAX_RECORDS};

use super::super::package::ImportLimits;
use super::*;

const REL: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";

fn rels(entries: &[(&str, &str, &str)]) -> String {
    let mut body = String::new();
    for (id, kind, target) in entries {
        body.push_str(&format!(
            r#"<Relationship Id="{id}" Type="{REL}/{kind}" Target="{target}"/>"#
        ));
    }
    format!(
        r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">{body}</Relationships>"#
    )
}

fn definition(fields: &str) -> String {
    format!(
        r#"<pivotCacheDefinition xmlns="m" xmlns:r="{REL}" r:id="rId1"><cacheSource type="worksheet"><worksheetSource ref="A1:C7" sheet="Data"/></cacheSource><cacheFields>{fields}</cacheFields></pivotCacheDefinition>"#
    )
}

const FIELDS: &str = r#"<cacheField name="Region"><sharedItems><s v="East"/><s v="West"/></sharedItems></cacheField><cacheField name="Product"><sharedItems/></cacheField><cacheField name="Sales"><sharedItems/></cacheField>"#;

fn records(rows: &str) -> String {
    format!(r#"<pivotCacheRecords xmlns="m">{rows}</pivotCacheRecords>"#)
}

const ROWS: &str = r##"<r><x v="0"/><s v="A"/><n v="10"/></r><r><x v="1"/><s v="B"/><n v="20"/></r><r><x v="0"/><s v="A"/><n v="5"/></r><r><x v="1"/><m/><b v="1"/></r><r><s v="West"/><d v="2024-01-01T00:00:00"/><e v="#N/A"/></r>"##;

fn table(body: &str) -> String {
    format!(
        r#"<pivotTableDefinition xmlns="m" name="PivotSum" cacheId="1">{body}</pivotTableDefinition>"#
    )
}

const TABLE_BODY: &str = r#"<location ref="A3:E7" firstHeaderRow="1"/><pivotFields count="3"><pivotField axis="axisRow"><items><item x="0"/><item x="1" h="1"/><item t="default"/></items></pivotField><pivotField axis="axisCol"/><pivotField dataField="1"/></pivotFields><rowFields><field x="0"/></rowFields><colFields><field x="1"/><field x="-2"/></colFields><dataFields><dataField name="Sum of Sales" fld="2" baseField="0" baseItem="0"/></dataFields>"#;

/// The parts of a workbook with one cache (cacheId 1) and one pivot on sheet1.
fn parts() -> Vec<(String, String)> {
    vec![
        (
            "xl/_rels/workbook.xml.rels".into(),
            rels(&[(
                "rIdC1",
                "pivotCacheDefinition",
                "pivotCache/pivotCacheDefinition1.xml",
            )]),
        ),
        (
            "xl/pivotCache/pivotCacheDefinition1.xml".into(),
            definition(FIELDS),
        ),
        (
            "xl/pivotCache/_rels/pivotCacheDefinition1.xml.rels".into(),
            rels(&[("rId1", "pivotCacheRecords", "pivotCacheRecords1.xml")]),
        ),
        ("xl/pivotCache/pivotCacheRecords1.xml".into(), records(ROWS)),
        (
            "xl/worksheets/_rels/sheet1.xml.rels".into(),
            rels(&[("rIdP1", "pivotTable", "../pivotTables/pivotTable1.xml")]),
        ),
        ("xl/pivotTables/pivotTable1.xml".into(), table(TABLE_BODY)),
    ]
}

/// `parts` with the part `name` replaced (or added) by `content`.
fn with(mut parts: Vec<(String, String)>, name: &str, content: String) -> Vec<(String, String)> {
    parts.retain(|(n, _)| n != name);
    parts.push((name.to_string(), content));
    parts
}

fn without(mut parts: Vec<(String, String)>, name: &str) -> Vec<(String, String)> {
    parts.retain(|(n, _)| n != name);
    parts
}

fn package(parts: &[(String, String)]) -> Package<Cursor<Vec<u8>>> {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, content) in parts {
        zip.start_file(name.as_str(), zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(content.as_bytes()).unwrap();
    }
    let bytes = zip.finish().unwrap().into_inner();
    Package::new(Cursor::new(bytes), ImportLimits::default()).unwrap()
}

fn workbook_rels(parts: &[(String, String)]) -> HashMap<String, Relationship> {
    let (_, xml) = parts
        .iter()
        .find(|(n, _)| n == "xl/_rels/workbook.xml.rels")
        .unwrap();
    parse_relationships(&XmlNode::parse_str(xml).unwrap())
        .unwrap()
        .into_iter()
        .collect()
}

fn caches(list: &[(&str, &str)]) -> Vec<(Option<String>, Option<String>)> {
    list.iter()
        .map(|(id, rel)| (Some((*id).to_string()), Some((*rel).to_string())))
        .collect()
}

fn load_with(
    parts: &[(String, String)],
    list: &[(&str, &str)],
) -> (Vec<PivotCache>, Vec<PivotTable>) {
    let sheets = [PivotSheet {
        sheet_id: 7,
        path: "xl/worksheets/sheet1.xml".into(),
    }];
    load_pivots(
        &mut package(parts),
        &workbook_rels(parts),
        &caches(list),
        &sheets,
    )
}

fn load(parts: &[(String, String)]) -> (Vec<PivotCache>, Vec<PivotTable>) {
    load_with(parts, &[("1", "rIdC1")])
}

fn text(s: &str) -> CacheItem {
    CacheItem::Text(s.to_string())
}

#[test]
fn shared_and_inline_values_load_into_the_cache_and_the_pivot() {
    let (caches, pivots) = load(&parts());
    assert_eq!(caches.len(), 1);
    let cache = &caches[0];
    assert_eq!(
        cache.source,
        Some(CacheSource {
            sheet_name: "Data".into(),
            range: "A1:C7".into()
        })
    );
    assert_eq!(cache.record_count, 5);
    let names: Vec<&str> = cache.fields.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(names, ["Region", "Product", "Sales"]);
    // An inline value equal to a shared item finds it.
    assert_eq!(cache.fields[0].items, [text("East"), text("West")]);
    assert_eq!(cache.fields[0].records, [0, 1, 0, 1, 1]);
    assert_eq!(
        cache.fields[1].items,
        [
            text("A"),
            text("B"),
            CacheItem::Missing,
            CacheItem::Date(45292.0)
        ]
    );
    assert_eq!(cache.fields[1].records, [0, 1, 0, 2, 3]);
    assert_eq!(
        cache.fields[2].items,
        [
            CacheItem::Number(10.0),
            CacheItem::Number(20.0),
            CacheItem::Number(5.0),
            CacheItem::Boolean(true),
            CacheItem::Error(Error::NA)
        ]
    );
    assert_eq!(pivots.len(), 1);
    let pivot = &pivots[0];
    assert_eq!(pivot.name, "PivotSum");
    assert_eq!(pivot.sheet_id, 7);
    assert_eq!(pivot.cache, 0);
    assert_eq!(
        pivot.location,
        CellArea {
            top: 2,
            left: 0,
            bottom: 6,
            right: 4
        }
    );
    let axes: Vec<Option<Axis>> = pivot.fields.iter().map(|f| f.axis).collect();
    assert_eq!(axes, [Some(Axis::Row), Some(Axis::Column), None]);
    assert!(!pivot.fields[0].is_hidden(0));
    assert!(pivot.fields[0].is_hidden(1));
    assert_eq!(pivot.row_fields, [AxisField::Field(0)]);
    assert_eq!(
        pivot.column_fields,
        [AxisField::Field(1), AxisField::Values]
    );
    assert_eq!(
        pivot.data_fields,
        [DataField {
            caption: "Sum of Sales".into(),
            field: 2,
            function: Aggregate::Sum
        }]
    );
}

/// Asserts that `parts` load no cache (and so no pivot).
fn assert_no_cache(parts: &[(String, String)]) {
    assert_eq!(load(parts), (Vec::new(), Vec::new()));
}

/// Asserts that `parts` load the cache but not the pivot.
fn assert_no_pivot(parts: &[(String, String)]) {
    let (caches, pivots) = load(parts);
    assert_eq!(caches.len(), 1);
    assert!(pivots.is_empty(), "{parts:?}");
}

const DEFINITION_PART: &str = "xl/pivotCache/pivotCacheDefinition1.xml";
const RECORDS_PART: &str = "xl/pivotCache/pivotCacheRecords1.xml";
const TABLE_PART: &str = "xl/pivotTables/pivotTable1.xml";

#[test]
fn a_cache_saved_without_records_is_skipped() {
    let no_id = definition(FIELDS).replace(r#" r:id="rId1""#, r#" saveData="0""#);
    assert_no_cache(&with(parts(), DEFINITION_PART, no_id));
    assert_no_cache(&without(parts(), RECORDS_PART));
}

#[test]
fn olap_and_other_sources_are_skipped() {
    let olap = definition(FIELDS).replace("</cacheFields>", "</cacheFields><cacheHierarchies/>");
    assert_no_cache(&with(parts(), DEFINITION_PART, olap));
    let external = definition(FIELDS).replace(r#"type="worksheet""#, r#"type="external""#);
    assert_no_cache(&with(parts(), DEFINITION_PART, external));
}

#[test]
fn grouped_and_calculated_fields_are_skipped() {
    let grouped = definition(FIELDS).replace(
        "<sharedItems/></cacheField>",
        r#"<sharedItems/><fieldGroup base="1"/></cacheField>"#,
    );
    assert_no_cache(&with(parts(), DEFINITION_PART, grouped));
    let calculated = definition(&format!(
        r#"{FIELDS}<cacheField name="Double" formula="Sales*2" databaseField="0"/>"#
    ));
    assert_no_cache(&with(parts(), DEFINITION_PART, calculated));
}

#[test]
fn a_named_source_keeps_the_cache_without_a_source() {
    let named = definition(FIELDS).replace(r#"ref="A1:C7" sheet="Data""#, r#"name="Sales""#);
    let (caches, pivots) = load(&with(parts(), DEFINITION_PART, named));
    assert_eq!(caches[0].source, None);
    assert_eq!(pivots.len(), 1);
}

#[test]
fn out_of_range_indices_skip_the_cache_or_the_pivot() {
    let bad_shared = records(&ROWS.replace(r#"<x v="1"/>"#, r#"<x v="2"/>"#));
    assert_no_cache(&with(parts(), RECORDS_PART, bad_shared));
    let short = records(r#"<r><x v="0"/><s v="A"/></r>"#);
    assert_no_cache(&with(parts(), RECORDS_PART, short));
    let long = records(r#"<r><x v="0"/><s v="A"/><n v="1"/><n v="2"/></r>"#);
    assert_no_cache(&with(parts(), RECORDS_PART, long));
    for (from, to) in [
        (r#"<item x="1" h="1"/>"#, r#"<item x="2"/>"#),
        (r#"<field x="0"/>"#, r#"<field x="3"/>"#),
        (r#"<field x="0"/>"#, r#"<field x="-1"/>"#),
        (r#"fld="2""#, r#"fld="9""#),
        ("A3:E7", "A0:E7"),
        ("A3:E7", "E7:A3"),
        ("A3:E7", "A3:XFE7"),
        (r#"<pivotField dataField="1"/>"#, ""),
        (
            r#"<pivotField dataField="1"/>"#,
            r#"<pivotField dataField="1"/><pivotField/>"#,
        ),
        (r#"name="Sum of Sales""#, r#"subtotal="median""#),
    ] {
        let body = TABLE_BODY.replace(from, to);
        assert_no_pivot(&with(parts(), TABLE_PART, table(&body)));
    }
    let unknown_cache = table(TABLE_BODY).replace(r#"cacheId="1""#, r#"cacheId="2""#);
    assert_no_pivot(&with(parts(), TABLE_PART, unknown_cache));
    let count = TABLE_BODY.replace(r#"name="Sum of Sales""#, r#"subtotal="countNums""#);
    let (_, pivots) = load(&with(parts(), TABLE_PART, table(&count)));
    assert_eq!(pivots[0].data_fields[0].function, Aggregate::CountNums);
    let shown_as = TABLE_BODY.replace(r#"baseItem="0""#, r#"showDataAs="percentOfTotal""#);
    assert_no_pivot(&with(parts(), TABLE_PART, table(&shown_as)));
}

#[test]
fn external_escaping_and_missing_targets_skip_the_pivot() {
    let workbook = "xl/_rels/workbook.xml.rels";
    let external = rels(&[(
        "rIdC1",
        "pivotCacheDefinition",
        "pivotCache/pivotCacheDefinition1.xml",
    )])
    .replace("/>", r#" TargetMode="External"/>"#);
    assert_no_cache(&with(parts(), workbook, external));
    for target in [
        "../../xl/pivotCache/pivotCacheDefinition1.xml",
        "/xl/pivotCache/pivotCacheDefinition1.xml",
        "pivotCache/missing.xml",
    ] {
        let escaping = rels(&[("rIdC1", "pivotCacheDefinition", target)]);
        assert_no_cache(&with(parts(), workbook, escaping));
    }
    let records_rels = "xl/pivotCache/_rels/pivotCacheDefinition1.xml.rels";
    let escaping = rels(&[(
        "rId1",
        "pivotCacheRecords",
        "../../../pivotCacheRecords1.xml",
    )]);
    assert_no_cache(&with(parts(), records_rels, escaping));
    let sheet_rels = "xl/worksheets/_rels/sheet1.xml.rels";
    for target in [
        "../../../pivotTable1.xml",
        "/xl/pivotTables/pivotTable1.xml",
    ] {
        assert_no_pivot(&with(
            parts(),
            sheet_rels,
            rels(&[("rIdP1", "pivotTable", target)]),
        ));
    }
    assert_no_pivot(&without(parts(), TABLE_PART));
}

#[test]
fn duplicate_cache_ids_and_parts_reached_twice_are_skipped() {
    // The same cacheId twice: neither loads.
    let (caches, _) = load_with(&parts(), &[("1", "rIdC1"), ("1", "rIdC1")]);
    assert!(caches.is_empty());
    // Two cacheIds reaching one definition: neither loads.
    let (caches, _) = load_with(&parts(), &[("1", "rIdC1"), ("2", "rIdC1")]);
    assert!(caches.is_empty());
    // The pivot part reached twice from a sheet.
    let twice = rels(&[
        ("rIdP1", "pivotTable", "../pivotTables/pivotTable1.xml"),
        ("rIdP2", "pivotTable", "../pivotTables/./pivotTable1.xml"),
    ]);
    assert_no_pivot(&with(parts(), "xl/worksheets/_rels/sheet1.xml.rels", twice));
    // A bad cacheId and an unknown relationship skip only their cache.
    let (caches, _) = load_with(&parts(), &[("x", "rIdC1"), ("2", "rIdNone")]);
    assert!(caches.is_empty());
}

#[test]
fn the_workbook_record_value_budget_drops_whole_caches() {
    let parts = parts();
    let two = with(
        parts,
        "xl/_rels/workbook.xml.rels",
        rels(&[
            (
                "rIdC1",
                "pivotCacheDefinition",
                "pivotCache/pivotCacheDefinition1.xml",
            ),
            (
                "rIdC2",
                "pivotCacheDefinition",
                "pivotCache/pivotCacheDefinition2.xml",
            ),
        ]),
    );
    let two = with(
        two,
        "xl/pivotCache/pivotCacheDefinition2.xml",
        definition(FIELDS).replace("rId1", "rId2"),
    );
    let two = with(
        two,
        "xl/pivotCache/_rels/pivotCacheDefinition2.xml.rels",
        rels(&[("rId2", "pivotCacheRecords", "pivotCacheRecords1.xml")]),
    );
    let list = caches(&[("1", "rIdC1"), ("2", "rIdC2")]);
    let workbook_rels = workbook_rels(&two);
    // Each cache holds 5 records x 3 fields = 15 values.
    let (loaded, ids) = load_caches(&mut package(&two), &workbook_rels, &list, 29);
    assert_eq!(loaded.len(), 1);
    assert_eq!(ids, HashMap::from([(1, 0)]));
    let (loaded, ids) = load_caches(&mut package(&two), &workbook_rels, &list, 30);
    assert_eq!(loaded.len(), 2);
    assert_eq!(ids, HashMap::from([(1, 0), (2, 1)]));
}

#[test]
fn the_cache_limits_drop_the_cache() {
    let fields = r#"<cacheField name="F"><sharedItems/></cacheField>"#.repeat(MAX_FIELDS + 1);
    assert_no_cache(&with(parts(), DEFINITION_PART, definition(&fields)));
    let items = format!(
        r#"<cacheField name="F"><sharedItems>{}</sharedItems></cacheField>"#,
        "<m/>".repeat(MAX_ITEMS + 1)
    );
    assert_no_cache(&with(parts(), DEFINITION_PART, definition(&items)));
    // A cache with one field and more records than allowed.
    let one = definition(r#"<cacheField name="F"><sharedItems><m/></sharedItems></cacheField>"#);
    let rows = records(&r#"<r><x v="0"/></r>"#.repeat(MAX_RECORDS + 1));
    assert_no_cache(&with(
        with(parts(), DEFINITION_PART, one),
        RECORDS_PART,
        rows,
    ));
}

#[test]
fn the_pivot_limits_drop_the_pivot() {
    let many_rows = TABLE_BODY.replace(
        r#"<rowFields><field x="0"/></rowFields>"#,
        &format!(
            "<rowFields>{}</rowFields>",
            r#"<field x="0"/>"#.repeat(MAX_AXIS_FIELDS + 1)
        ),
    );
    assert_no_pivot(&with(parts(), TABLE_PART, table(&many_rows)));
    let at_limit = TABLE_BODY.replace(
        r#"<rowFields><field x="0"/></rowFields>"#,
        &format!(
            "<rowFields>{}</rowFields>",
            r#"<field x="0"/>"#.repeat(MAX_AXIS_FIELDS)
        ),
    );
    assert_eq!(
        load(&with(parts(), TABLE_PART, table(&at_limit))).1.len(),
        1
    );
    let data = r#"<dataField name="Sum of Sales" fld="2"/>"#;
    let many_data = TABLE_BODY.replace(
        r#"<dataFields>"#,
        &format!("<dataFields>{}", data.repeat(MAX_DATA_FIELDS)),
    );
    assert_no_pivot(&with(parts(), TABLE_PART, table(&many_data)));
    let pages = r#"<pageField fld="1"/>"#.repeat(MAX_AXIS_FIELDS + 1);
    let many_pages = TABLE_BODY.replace(
        "<dataFields>",
        &format!("<pageFields>{pages}</pageFields><dataFields>"),
    );
    assert_no_pivot(&with(parts(), TABLE_PART, table(&many_pages)));
    let deep = TABLE_BODY.replace(
        "<location",
        &format!(
            "{}<location",
            "<a>".repeat(MAX_DEPTH) + &"</a>".repeat(MAX_DEPTH)
        ),
    );
    assert_no_pivot(&with(parts(), TABLE_PART, table(&deep)));
    // 65 pivots on one sheet: the first 64 load.
    let mut many = parts();
    let mut sheet_rels = Vec::new();
    for n in 0..=MAX_PIVOTS_PER_SHEET {
        let part = format!("pivotTable{n}x.xml");
        many = with(many, &format!("xl/pivotTables/{part}"), table(TABLE_BODY));
        sheet_rels.push((format!("rIdP{n}"), format!("../pivotTables/{part}")));
    }
    let entries: Vec<(&str, &str, &str)> = sheet_rels
        .iter()
        .map(|(id, target)| (id.as_str(), "pivotTable", target.as_str()))
        .collect();
    many = with(many, "xl/worksheets/_rels/sheet1.xml.rels", rels(&entries));
    assert_eq!(load(&many).1.len(), MAX_PIVOTS_PER_SHEET);
}
