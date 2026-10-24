//! Unit tests of the xlsx snapshot reader.

use super::*;
use std::io::Write;

#[test]
fn a_calc_workbook_claiming_excel_is_not_excel_authored() {
    // poi/clone_sheet.xlsx: app.xml says Microsoft Excel, workbook.xml says appName="Calc".
    let parsed = parse_workbook(r#"<workbook><fileVersion appName="Calc"/></workbook>"#).unwrap();
    assert_eq!(parsed.file_app_name.as_deref(), Some("Calc"));
    let parsed_xl = parse_workbook(r#"<workbook><fileVersion appName="xl"/></workbook>"#).unwrap();
    assert_eq!(parsed_xl.file_app_name.as_deref(), Some("xl"));
    let snapshot = |file_app_name: Option<&str>| WorkbookSnapshot {
        application: Some("Microsoft Excel".into()),
        app_version: None,
        file_app_name: file_app_name.map(str::to_string),
        date1904: false,
        sheets: Vec::new(),
        defined_names: Vec::new(),
        tables: Vec::new(),
    };
    assert!(!snapshot(Some("Calc")).is_excel_authored());
    assert!(snapshot(Some("xl")).is_excel_authored());
    assert!(snapshot(None).is_excel_authored());
}

#[test]
fn resolve_target_handles_absolute_and_dotdot() {
    assert_eq!(
        resolve_target("xl", "worksheets/sheet1.xml"),
        "xl/worksheets/sheet1.xml"
    );
    assert_eq!(
        resolve_target("xl/worksheets", "../tables/table1.xml"),
        "xl/tables/table1.xml"
    );
    assert_eq!(resolve_target("xl", "/xl/sheet2.xml"), "xl/sheet2.xml");
}

#[test]
fn relationships_reads_entries_and_defaults_missing_type() {
    let xml = r#"<Relationships><Relationship Id="r1" Type="t/table" Target="a.xml"/><Relationship Id="r2" Target="b.xml"/><Relationship Id="r3" Type="x"/></Relationships>"#;
    let rels = relationships(xml, "x.rels").unwrap();
    assert_eq!(rels.len(), 2);
    assert_eq!(
        (
            rels[0].id.as_str(),
            rels[0].kind.as_str(),
            rels[0].target.as_str()
        ),
        ("r1", "t/table", "a.xml")
    );
    assert_eq!(
        (rels[1].kind.as_str(), rels[1].target.as_str()),
        ("", "b.xml")
    );
}

fn zip_with(name: &str, body: &[u8]) -> Zip {
    let mut w = zip::ZipWriter::new(Cursor::new(Vec::new()));
    w.start_file(name, zip::write::SimpleFileOptions::default())
        .unwrap();
    w.write_all(body).unwrap();
    let bytes = w.finish().unwrap().into_inner();
    Zip::new(&bytes).unwrap()
}

#[test]
fn part_over_the_size_limit_is_malformed() {
    let mut zip = zip_with("xl/big.xml", &[b'a'; 100]);
    assert_eq!(
        read_part_capped(&mut zip, "xl/big.xml", 100)
            .unwrap()
            .unwrap()
            .len(),
        100
    );
    let err = read_part_capped(&mut zip, "xl/big.xml", 99).unwrap_err();
    assert!(matches!(&err, SnapshotError::Malformed(m) if m.contains("xl/big.xml")));
}

#[test]
fn parts_are_found_case_insensitively_across_backslashes() {
    let mut zip = zip_with("xl\\sharedstrings.xml", b"s");
    assert_eq!(
        read_part(&mut zip, "xl/sharedStrings.xml")
            .unwrap()
            .as_deref(),
        Some("s")
    );
    assert_eq!(read_part(&mut zip, "xl/styles.xml").unwrap(), None);
}

const SHEET: &str = r#"<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData>
<row r="1"><c r="A1"><v>1.5</v></c><c r="B1" t="s"><v>0</v></c><c r="C1" t="b"><v>1</v></c></row>
<row r="2"><c r="A2"><f>A1*2</f><v>3</v></c><c r="B2" t="str"><f>B1&amp;"!"</f><v>hi!</v></c><c r="C2" t="e"><f>1/0</f><v>#DIV/0!</v></c></row>
<row r="3"><c r="A3"><f t="shared" ref="A3:A4" si="0">A2+1</f><v>4</v></c><c r="B3"><f t="array" ref="B3:B4">SEQUENCE(2)</f><v>1</v></c></row>
<row r="4"><c r="A4"><f t="shared" si="0"/><v>5</v></c><c r="B4"><v>2</v></c><c r="C4" s="3"/></row>
</sheetData></worksheet>"#;

#[test]
fn shared_member_before_master_is_shifted_with_absolute_refs() {
    let xml = r#"<worksheet><sheetData>
<row r="2"><c r="A2"><f t="shared" si="3"/><v>0</v></c></row>
<row r="1"><c r="A1"><f t="shared" ref="A1:A2" si="3">$B1+C$1+ SUM(D1:E2)</f><v>0</v></c></row>
</sheetData></worksheet>"#;
    let cells = parse_sheet(xml, "s.xml", &[], &[], DateSystem::Excel1900).unwrap_or_default();
    let a2 = Addr::parse("A2")
        .and_then(|a| cells.get(&a))
        .and_then(|c| c.formula.clone());
    assert_eq!(a2.as_deref(), Some("$B2+C$1+ SUM(D2:E3)"));
}

#[test]
fn parses_values_formulas_shared_and_arrays() {
    let cells = parse_sheet(
        SHEET,
        "sheet1.xml",
        &["hi".to_string()],
        &[],
        DateSystem::Excel1900,
    )
    .unwrap_or_default();
    let get = |s: &str| Addr::parse(s).and_then(|a| cells.get(&a)).cloned();
    assert_eq!(get("A1").map(|c| c.value), Some(Value::Number(1.5)));
    assert_eq!(get("B1").map(|c| c.value), Some(Value::Text("hi".into())));
    assert_eq!(get("C1").map(|c| c.value), Some(Value::Bool(true)));
    let a2 = get("A2");
    assert_eq!(
        a2.as_ref().and_then(|c| c.formula.clone()).as_deref(),
        Some("A1*2")
    );
    assert_eq!(get("B2").map(|c| c.value), Some(Value::Text("hi!".into())));
    assert_eq!(
        get("C2").map(|c| c.value),
        Some(Value::Error("#DIV/0!".into()))
    );
    assert_eq!(
        get("A4").and_then(|c| c.formula).as_deref(),
        Some("A3+1"),
        "shared member shows the master's formula shifted to its own cell"
    );
    assert_eq!(
        get("B4").and_then(|c| c.formula_kind),
        Some(FormulaKind::ArrayMember {
            anchor: "B3".into()
        })
    );
    assert!(get("C4").is_none(), "styled empty cells are dropped");
}

#[test]
fn empty_formulas_inside_a_volatile_spill_are_array_members() {
    // Excel saves the members of a spilled RANDARRAY as `<f ca="1"/>` with a value
    // (calipers roundtrip/formulas_dynamic_arrays F261:F265).
    let xml = r#"<worksheet><sheetData>
<row r="1"><c r="A1" cm="1"><f t="array" ref="A1:A3" ca="1">_xlfn.RANDARRAY(3)</f><v>0.5</v></c></row>
<row r="2"><c r="A2"><f ca="1"/><v>0.25</v></c></row>
<row r="3"><c r="A3"><f ca="1"/><v>0.75</v></c></row>
<row r="4"><c r="A4"><f ca="1">RAND()</f><v>0.1</v></c></row>
</sheetData></worksheet>"#;
    let cells = parse_sheet(xml, "s.xml", &[], &[], DateSystem::Excel1900).unwrap_or_default();
    let kind = |s: &str| {
        Addr::parse(s)
            .and_then(|a| cells.get(&a))
            .and_then(|c| c.formula_kind.clone())
    };
    for member in ["A2", "A3"] {
        assert_eq!(
            kind(member),
            Some(FormulaKind::ArrayMember {
                anchor: "A1".into()
            }),
            "{member}"
        );
    }
    assert_eq!(
        kind("A4"),
        Some(FormulaKind::Normal),
        "outside the array range"
    );
}

#[test]
fn rich_value_errors_replace_the_cached_value_error() {
    let sheet = r#"<worksheet><sheetData><row r="1"><c r="A1" t="e" vm="1"><f>1</f><v>#VALUE!</v></c><c r="B1" t="e" vm="2"><f>1</f><v>#VALUE!</v></c><c r="C1" t="e"><f>1</f><v>#VALUE!</v></c></row></sheetData></worksheet>"#;
    // errorType 13 = #CALC!, 8 = #SPILL! (ERROR.TYPE.xlsx)
    let rich = [Some("#CALC!"), Some("#SPILL!")];
    let cells =
        parse_sheet(sheet, "sheet1.xml", &[], &rich, DateSystem::Excel1900).unwrap_or_default();
    let get = |s: &str| {
        Addr::parse(s)
            .and_then(|a| cells.get(&a))
            .map(|c| c.value.clone())
    };
    assert_eq!(get("A1"), Some(Value::Error("#CALC!".into())));
    assert_eq!(get("B1"), Some(Value::Error("#SPILL!".into())));
    assert_eq!(get("C1"), Some(Value::Error("#VALUE!".into())));
}

/// A `vm` that resolves to no error (out of range, no errorType) is undecoded, not `#VALUE!`.
#[test]
fn unresolved_rich_value_error_is_undecoded() {
    let sheet = r#"<worksheet><sheetData><row r="1"><c r="A1" t="e" vm="3"><f>1</f><v>#VALUE!</v></c><c r="B1" t="e" vm="1"><f>1</f><v>#VALUE!</v></c><c r="C1" t="e" vm="1"><f>1</f><v>#N/A</v></c></row></sheetData></worksheet>"#;
    let rich = [None, Some("#CALC!")];
    let cells =
        parse_sheet(sheet, "sheet1.xml", &[], &rich, DateSystem::Excel1900).unwrap_or_default();
    let get = |s: &str| {
        Addr::parse(s)
            .and_then(|a| cells.get(&a))
            .map(|c| c.value.clone())
    };
    assert_eq!(get("A1"), Some(Value::Error(UNDECODED_RICH_ERROR.into())));
    assert_eq!(get("B1"), Some(Value::Error(UNDECODED_RICH_ERROR.into())));
    assert_eq!(get("C1"), Some(Value::Error("#N/A".into())));
}

#[test]
fn decodes_xlsx_control_char_escapes_in_text_values() {
    // CHAR.xlsx stores CHAR(1) as `_x0001_`.
    let sheet = r#"<worksheet><sheetData><row r="1"><c r="A1" t="str"><f>CHAR(1)</f><v>_x0001_</v></c></row></sheetData></worksheet>"#;
    let cells =
        parse_sheet(sheet, "sheet1.xml", &[], &[], DateSystem::Excel1900).unwrap_or_default();
    let get = |s: &str| {
        Addr::parse(s)
            .and_then(|a| cells.get(&a))
            .map(|c| c.value.clone())
    };
    assert_eq!(get("A1"), Some(Value::Text("\u{1}".into())));
}

mod defined_name_tests {
    use crate::xlsx_snapshot::*;

    #[test]
    fn reads_scoped_empty_and_escaped_defined_names() {
        let xml = r#"<workbook><sheets><sheet name="S" r:id="rId1"/></sheets><definedNames>
            <definedName name="G">S!$A$1+1&lt;2</definedName>
            <definedName name="E"/>
            <definedName name="L" localSheetId="0">TODAY()</definedName>
            </definedNames></workbook>"#;
        let names = parse_workbook(xml).expect("parses").names;
        assert_eq!(
            names,
            [
                ("G".to_string(), None, "S!$A$1+1<2".to_string()),
                ("E".to_string(), None, String::new()),
                ("L".to_string(), Some(0), "TODAY()".to_string()),
            ]
        );
    }
}

mod anchor_formula_tests {
    use crate::xlsx_snapshot::*;

    fn cell(formula: Option<&str>, kind: Option<FormulaKind>) -> CellSnapshot {
        CellSnapshot {
            formula: formula.map(str::to_string),
            formula_kind: kind,
            value: Value::Empty,
        }
    }

    #[test]
    fn member_resolves_its_anchor_formula() {
        let anchor = Addr::parse("A1").expect("addr");
        let member = cell(
            None,
            Some(FormulaKind::ArrayMember {
                anchor: "A1".into(),
            }),
        );
        let sheets = [SheetSnapshot {
            name: "S".into(),
            cells: BTreeMap::from([(anchor, cell(Some("SUM(1)"), None))]),
        }];
        assert_eq!(
            member.anchor_formula(&sheets, "S").as_deref(),
            Some("SUM(1)")
        );
        assert_eq!(member.anchor_formula(&sheets, "Other"), None);
        assert_eq!(cell(None, None).anchor_formula(&sheets, "S"), None);
    }
}

/// A `t="d"` cell's cached value is its date's serial in the workbook's date system (T-2515,
/// fixtures/t-d-1904, t-d-1904-pre-epoch).
#[test]
fn t_d_values_follow_the_date_system() {
    let sheet = r#"<worksheet><sheetData><row r="1"><c r="A1" t="d"><v>2024-01-01T00:00:00</v></c><c r="B1" t="d"><v>1903-12-31</v></c></row></sheetData></worksheet>"#;
    let value = |system, s: &str| {
        let cells = parse_sheet(sheet, "sheet1.xml", &[], &[], system).unwrap_or_default();
        Addr::parse(s)
            .and_then(|a| cells.get(&a))
            .map(|c| c.value.clone())
    };
    assert_eq!(
        value(DateSystem::Excel1900, "A1"),
        Some(Value::Number(45292.0))
    );
    assert_eq!(
        value(DateSystem::Excel1904, "A1"),
        Some(Value::Number(43830.0))
    );
    assert_eq!(
        value(DateSystem::Excel1900, "B1"),
        Some(Value::Number(1461.0))
    );
    assert_eq!(
        value(DateSystem::Excel1904, "B1"),
        Some(Value::Error("#VALUE!".into()))
    );
}
