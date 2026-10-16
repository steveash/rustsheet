use std::fs;

use rustsheet_engine::types::{DefinedName, StyleIncludes};
use rustsheet_engine::Model;

use crate::error::XlsxError;
use crate::export::save_to_icalc;
use crate::import::load_from_icalc;
use crate::{export::save_to_xlsx, import::load_from_xlsx};

pub fn new_empty_model() -> Model {
    Model::new_empty("model", "en", "UTC", "en").unwrap()
}

#[test]
fn test_values() {
    let mut model = new_empty_model();
    // numbers
    model
        .set_user_input(0, 1, 1, "123.456".to_string())
        .unwrap();
    // strings
    model
        .set_user_input(0, 2, 1, "Hello world!".to_string())
        .unwrap();
    model
        .set_user_input(0, 3, 1, "Hello world!".to_string())
        .unwrap();
    model
        .set_user_input(0, 4, 1, "你好世界！".to_string())
        .unwrap();
    // booleans
    model.set_user_input(0, 5, 1, "TRUE".to_string()).unwrap();
    model.set_user_input(0, 6, 1, "FALSE".to_string()).unwrap();
    // errors
    model
        .set_user_input(0, 7, 1, "#VALUE!".to_string())
        .unwrap();

    // noop
    model.evaluate();
    {
        let temp_file_name = "temp_file_test_values.xlsx";
        save_to_xlsx(&model, temp_file_name).unwrap();

        let model = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
        assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "123.456");
        assert_eq!(
            model.get_formatted_cell_value(0, 2, 1).unwrap(),
            "Hello world!"
        );
        assert_eq!(
            model.get_formatted_cell_value(0, 3, 1).unwrap(),
            "Hello world!"
        );
        assert_eq!(
            model.get_formatted_cell_value(0, 4, 1).unwrap(),
            "你好世界！"
        );
        assert_eq!(model.get_formatted_cell_value(0, 5, 1).unwrap(), "TRUE");
        assert_eq!(model.get_formatted_cell_value(0, 6, 1).unwrap(), "FALSE");
        assert_eq!(model.get_formatted_cell_value(0, 7, 1).unwrap(), "#VALUE!");

        fs::remove_file(temp_file_name).unwrap();
    }
    {
        let temp_file_name = "temp_file_test_values.ic";
        save_to_icalc(&model, temp_file_name).unwrap();

        let model = load_from_icalc(temp_file_name, "en").unwrap();
        assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "123.456");
        assert_eq!(
            model.get_formatted_cell_value(0, 2, 1).unwrap(),
            "Hello world!"
        );
        assert_eq!(
            model.get_formatted_cell_value(0, 3, 1).unwrap(),
            "Hello world!"
        );
        assert_eq!(
            model.get_formatted_cell_value(0, 4, 1).unwrap(),
            "你好世界！"
        );
        assert_eq!(model.get_formatted_cell_value(0, 5, 1).unwrap(), "TRUE");
        assert_eq!(model.get_formatted_cell_value(0, 6, 1).unwrap(), "FALSE");
        assert_eq!(model.get_formatted_cell_value(0, 7, 1).unwrap(), "#VALUE!");

        fs::remove_file(temp_file_name).unwrap();
    }
}

#[test]
fn frozen_rows() {
    let mut model = new_empty_model();
    model.set_frozen_rows(0, 23).unwrap();
    model.evaluate();
    let temp_file_name = "temp_file_test_frozen_rows.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();
    let model = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    assert_eq!(model.get_frozen_rows_count(0).unwrap(), 23);
    fs::remove_file(temp_file_name).unwrap();
}

#[test]
fn frozen_columns() {
    let mut model = new_empty_model();
    model.set_frozen_columns(0, 42).unwrap();
    model.evaluate();
    let temp_file_name = "temp_file_test_frozen_columns.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();
    let model = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    assert_eq!(model.get_frozen_columns_count(0).unwrap(), 42);
    fs::remove_file(temp_file_name).unwrap();
}

#[test]
fn frozen_rows_and_columns() {
    let mut model = new_empty_model();
    model.set_frozen_rows(0, 23).unwrap();
    model.set_frozen_columns(0, 42).unwrap();
    model.evaluate();
    let temp_file_name = "temp_file_test_frozen_rows_and_columns.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();
    let model = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    assert_eq!(model.get_frozen_rows_count(0).unwrap(), 23);
    assert_eq!(model.get_frozen_columns_count(0).unwrap(), 42);
    fs::remove_file(temp_file_name).unwrap();
}

#[test]
fn test_formulas() {
    let mut model = new_empty_model();
    model.set_user_input(0, 1, 1, "5.5".to_string()).unwrap();
    model.set_user_input(0, 2, 1, "6.5".to_string()).unwrap();
    model.set_user_input(0, 3, 1, "7.5".to_string()).unwrap();

    model.set_user_input(0, 1, 2, "=A1*2".to_string()).unwrap();
    model.set_user_input(0, 2, 2, "=A2*2".to_string()).unwrap();
    model.set_user_input(0, 3, 2, "=A3*2".to_string()).unwrap();
    model
        .set_user_input(0, 4, 2, "=SUM(A1:B3)".to_string())
        .unwrap();

    model.evaluate();
    let temp_file_name = "temp_file_test_formulas.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();

    let model = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 1, 2).unwrap(), "11");
    assert_eq!(model.get_formatted_cell_value(0, 2, 2).unwrap(), "13");
    assert_eq!(model.get_formatted_cell_value(0, 3, 2).unwrap(), "15");
    assert_eq!(model.get_formatted_cell_value(0, 4, 2).unwrap(), "58.5");
    fs::remove_file(temp_file_name).unwrap();
}

/// The text of the part `name` of the xlsx file `path`.
fn part_text(path: &str, name: &str) -> String {
    let file = fs::File::open(path).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    let mut text = String::new();
    std::io::Read::read_to_string(&mut archive.by_name(name).unwrap(), &mut text).unwrap();
    text
}

// Excel takes `LET(R1C1,5,R1C1*2)` (10, probes/r1c1-let-name-r1c1) and writes the name as
// `_xlpm.R1C1`; the import reads it as a name, not the cell R1C1 (T-1658).
#[test]
fn test_let_name_spelled_like_an_r1c1_cell_round_trips() {
    let mut model = new_empty_model();
    model
        .set_user_input(0, 1, 1, "=LET(R1C1,5,R1C1*2)".to_string())
        .unwrap();
    model.evaluate();
    let temp_file_name = "temp_file_test_let_r1c1_name.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();

    let sheet_xml = part_text(temp_file_name, "xl/worksheets/sheet1.xml");
    assert!(
        sheet_xml.contains("_xlfn.LET(_xlpm.R1C1,5,_xlpm.R1C1*2)"),
        "{sheet_xml}"
    );

    let mut model = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    fs::remove_file(temp_file_name).unwrap();
    model.evaluate();
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "10");
    assert_eq!(
        model.get_cell_formula(0, 1, 1).unwrap(),
        Some("=LET(R1C1,5,R1C1*2)".to_string())
    );
}

#[test]
fn test_array_formulas_keep_implicit_intersection() {
    // The `@` of an array formula is not put back when the file is read, so it
    // has to be written. Without it `@A1:A3` in row 2 would be all of A1:A3.
    let mut model = new_empty_model();
    model.set_user_input(0, 1, 1, "1".to_string()).unwrap();
    model.set_user_input(0, 2, 1, "2".to_string()).unwrap();
    model.set_user_input(0, 3, 1, "3".to_string()).unwrap();
    // A CSE formula in C2 and one in D2:D3
    model
        .set_user_array_formula(0, 2, 3, 1, 1, "=POWER(@A1:A3, 3)")
        .unwrap();
    model
        .set_user_array_formula(0, 2, 4, 1, 2, "=SIN(@A1:A3)+A1:A2")
        .unwrap();
    model.evaluate();
    assert_eq!(model.get_formatted_cell_value(0, 2, 3).unwrap(), "8");

    let temp_file_name = "temp_file_test_array_formulas_intersection.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();

    let mut model = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    fs::remove_file(temp_file_name).unwrap();
    assert_eq!(
        model.get_cell_formula(0, 2, 3).unwrap(),
        Some("=POWER(@A1:A3,3)".to_string())
    );
    assert_eq!(
        model.get_cell_formula(0, 2, 4).unwrap(),
        Some("=SIN(@A1:A3)+A1:A2".to_string())
    );
    model.evaluate();
    assert_eq!(model.get_formatted_cell_value(0, 2, 3).unwrap(), "8");
    // sin(2) + 1 and sin(2) + 2
    assert_eq!(
        model.get_formatted_cell_value(0, 2, 4).unwrap(),
        "1.909297427"
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 3, 4).unwrap(),
        "2.909297427"
    );
}

#[test]
fn test_sheets() {
    let mut model = new_empty_model();
    model.add_sheet("With space").unwrap();
    // xml escaped
    model.add_sheet("Tango & Cash").unwrap();
    model.add_sheet("你好世界").unwrap();

    // noop
    model.evaluate();

    let temp_file_name = "temp_file_test_sheets.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();

    let model = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    assert_eq!(
        model.workbook.get_worksheet_names(),
        vec!["Sheet1", "With space", "Tango & Cash", "你好世界"]
    );
    fs::remove_file(temp_file_name).unwrap();
}

#[test]
fn test_move_sheet_order_roundtrips() {
    let mut model = new_empty_model();
    model.add_sheet("Second").unwrap();
    model.add_sheet("Third").unwrap();
    // A cross-sheet reference so we can confirm it survives the reorder + save.
    model.set_user_input(1, 1, 1, "42".to_string()).unwrap(); // Second!A1
    model
        .set_user_input(0, 1, 1, "=Second!A1".to_string())
        .unwrap(); // Sheet1!A1

    // Reorder: Sheet1 goes to the end.
    model.move_sheet(0, 2).unwrap();
    assert_eq!(
        model.workbook.get_worksheet_names(),
        vec!["Second", "Third", "Sheet1"]
    );

    let temp_file_name = "temp_file_test_move_sheet_order.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();

    let model = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    // The saved order is preserved on reload.
    assert_eq!(
        model.workbook.get_worksheet_names(),
        vec!["Second", "Third", "Sheet1"]
    );
    // Sheet1 (now index 2) still resolves its reference to Second by name.
    assert_eq!(model.get_formatted_cell_value(2, 1, 1).unwrap(), "42");
    fs::remove_file(temp_file_name).unwrap();
}

#[test]
fn test_named_styles() {
    let mut model = new_empty_model();
    model.set_user_input(0, 1, 1, "5.5".to_string()).unwrap();
    let mut style = model.get_style_for_cell(0, 1, 1).unwrap();
    style.font.b = true;
    style.font.i = true;
    let e = model.workbook.styles.create_named_style(
        "bold & italics",
        &style,
        StyleIncludes::default(),
    );
    assert!(e.is_ok());
    assert!(model
        .set_cell_style_by_name(0, 1, 1, "bold & italics")
        .is_ok());

    // noop
    model.evaluate();

    let temp_file_name = "temp_file_test_named_styles.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();

    let mut model = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    assert!(model
        .workbook
        .styles
        .get_style_index_by_name("bold & italics")
        .is_ok());
    // After the roundtrip the style record must still include every formatting
    // category (in cellStyleXfs absent apply* attributes mean "included";
    // exporting applyX="0" would make Excel treat the style as empty).
    let cell_style = model
        .workbook
        .styles
        .cell_styles
        .iter()
        .find(|cs| cs.name == "bold & italics")
        .unwrap();
    let record = &model.workbook.styles.cell_style_xfs[cell_style.xf_id as usize];
    assert!(record.apply_number_format);
    assert!(record.apply_font);
    assert!(record.apply_fill);
    assert!(record.apply_border);
    assert!(record.apply_alignment);
    assert!(record.apply_protection);

    // The cell is still linked to the named style: updating it restyles the cell
    let mut style = model.get_named_style("bold & italics").unwrap();
    assert!(style.font.b);
    style.font.u = true;
    model
        .update_named_style(
            "bold & italics",
            "bold & italics",
            &style,
            StyleIncludes::default(),
        )
        .unwrap();
    assert!(model.get_style_for_cell(0, 1, 1).unwrap().font.u);
    fs::remove_file(temp_file_name).unwrap();
}

#[test]
fn test_existing_file() {
    let file_name = "existing_file.xlsx";
    fs::File::create(file_name).unwrap();

    assert_eq!(
        save_to_xlsx(&new_empty_model(), file_name),
        Err(XlsxError::IO(
            "file existing_file.xlsx already exists".to_string()
        )),
    );

    fs::remove_file(file_name).unwrap();
}

#[test]
fn test_defined_name_lambda_exports_excel_prefixes() {
    let mut model = new_empty_model();
    model
        .new_defined_name("cal_year", None, "Sheet1!$A$1")
        .unwrap();
    model
        .new_defined_name(
            "month_start",
            None,
            "LAMBDA(mo,LET(anchor,DATE(cal_year,mo,1),anchor+1))",
        )
        .unwrap();
    model.set_user_input(0, 1, 1, "2024".to_string()).unwrap();
    model
        .set_user_input(0, 2, 1, "=month_start(3)".to_string())
        .unwrap();
    // reference value computed without defined names
    model
        .set_user_input(0, 3, 1, "=DATE(2024,3,1)+1".to_string())
        .unwrap();
    model.evaluate();
    let expected_value = model.get_cell_value_by_index(0, 3, 1).unwrap();
    assert_eq!(
        model.get_cell_value_by_index(0, 2, 1).unwrap(),
        expected_value
    );

    let temp_file_name = "temp_file_test_defined_name_lambda.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();

    // The workbook part must store the formula in Excel's form: future
    // functions prefixed with `_xlfn.` and lambda/let variables with `_xlpm.`.
    let workbook_xml = part_text(temp_file_name, "xl/workbook.xml");
    assert!(
        workbook_xml.contains(
            "_xlfn.LAMBDA(_xlpm.mo,_xlfn.LET(_xlpm.anchor,DATE(cal_year,_xlpm.mo,1),_xlpm.anchor+1))"
        ),
        "workbook.xml does not contain the Excel form of the defined name: {workbook_xml}"
    );
    // The plain range name is stored untouched.
    assert!(workbook_xml.contains("<definedName name=\"cal_year\">Sheet1!$A$1</definedName>"));

    // And the file round-trips: the lambda still evaluates and the defined
    // name displays without the prefixes.
    let model = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    assert_eq!(
        model.get_cell_value_by_index(0, 2, 1).unwrap(),
        expected_value
    );
    let defined_names = model.get_defined_name_list();
    let month_start = defined_names
        .iter()
        .find(|(name, ..)| name == "month_start")
        .unwrap();
    assert_eq!(
        month_start.2,
        "LAMBDA(mo,LET(anchor,DATE(cal_year,mo,1),anchor+1))"
    );
    fs::remove_file(temp_file_name).unwrap();
}

// Excel does not snap a grouped defined-name body to zero (numbers.md "Near-zero snapping",
// T-1164), so export and import must keep the outer parentheses.
#[test]
fn test_grouped_defined_name_keeps_parentheses_through_export() {
    let mut model = new_empty_model();
    model
        .define_name("grouped", None, "=(0.1+0.2-0.3)")
        .unwrap();
    let temp_file_name = "temp_file_test_grouped_defined_name.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();
    let model = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    fs::remove_file(temp_file_name).unwrap();
    let defined_names = model.get_defined_name_list();
    assert_eq!(defined_names[0].2.trim_start_matches('='), "(0.1+0.2-0.3)");
}

// A Print_Area-shaped union body is stored without parentheses and must come back as stored,
// still standing for its two areas (SPEC-0004 "Defined names", probes/union-defined-name).
#[test]
fn test_union_defined_name_round_trips_through_export() {
    let mut model = new_empty_model();
    model.set_user_input(0, 1, 1, "1".to_string()).unwrap();
    model.set_user_input(0, 2, 2, "5".to_string()).unwrap();
    model
        .define_name("mu", None, "Sheet1!$A$1,Sheet1!$B$2")
        .unwrap();
    model
        .set_user_input(0, 1, 3, "=SUM(mu)".to_string())
        .unwrap();
    model.evaluate();
    let temp_file_name = "temp_file_test_union_defined_name.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();
    let mut model = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    fs::remove_file(temp_file_name).unwrap();
    let defined_names = model.get_defined_name_list();
    assert_eq!(
        defined_names[0].2.trim_start_matches('='),
        "Sheet1!$A$1,Sheet1!$B$2"
    );
    model.evaluate();
    assert_eq!(model.get_formatted_cell_value(0, 1, 3).unwrap(), "6");
}

// A sheet-local Print_Area is `_xlnm.Print_Area` with `localSheetId` in workbook.xml and plain
// `Print_Area` in formulas (probes/multiarea-printarea, T-1193).
#[test]
fn test_sheet_local_print_area_round_trips_with_xlnm_prefix() {
    let mut model = new_empty_model();
    model.add_sheet("Other").unwrap();
    model.set_user_input(0, 1, 1, "1".to_string()).unwrap();
    model.set_user_input(0, 2, 2, "5".to_string()).unwrap();
    model
        .define_name("Print_Area", Some(0), "Sheet1!$A$1,Sheet1!$B$2")
        .unwrap();
    model
        .set_user_input(0, 1, 8, "=SUM(Print_Area)".to_string())
        .unwrap();
    model
        .set_user_input(0, 2, 8, "=AREAS(Print_Area)".to_string())
        .unwrap();
    model.evaluate();
    let temp_file_name = "temp_file_test_sheet_local_print_area.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();

    let workbook_xml = part_text(temp_file_name, "xl/workbook.xml");
    assert!(
        workbook_xml.contains(
            "<definedName name=\"_xlnm.Print_Area\" localSheetId=\"0\">Sheet1!$A$1,Sheet1!$B$2</definedName>"
        ),
        "{workbook_xml}"
    );

    let mut model = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    fs::remove_file(temp_file_name).unwrap();
    let defined_names = model.get_defined_name_list();
    assert_eq!(defined_names[0].0, "Print_Area");
    assert_eq!(defined_names[0].1, Some(0));
    model.evaluate();
    assert_eq!(model.get_formatted_cell_value(0, 1, 8).unwrap(), "6");
    assert_eq!(model.get_formatted_cell_value(0, 2, 8).unwrap(), "2");
}

#[test]
fn defined_name_scoped_to_a_missing_sheet_is_an_export_error() {
    let mut model = new_empty_model();
    model.workbook.defined_names.push(DefinedName {
        name: "Print_Area".to_string(),
        formula: "Sheet1!$A$1".to_string(),
        sheet_id: Some(99),
    });
    let result = crate::export::workbook::get_workbook_xml(&model.workbook, 0);
    assert!(matches!(result, Err(XlsxError::Workbook(_))));
}
