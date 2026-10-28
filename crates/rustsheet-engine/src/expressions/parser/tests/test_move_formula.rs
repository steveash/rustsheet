use std::collections::HashMap;

use crate::expressions::parser::move_formula::{move_formula as mf, MoveContext};
use crate::expressions::parser::stringify::to_english_string;
use crate::expressions::parser::tests::utils::new_parser;
use crate::expressions::parser::Node;
use crate::expressions::types::{Area, CellReferenceRC};
use crate::language::get_default_language;
use crate::locale::get_default_locale;

fn move_formula(node: &Node, context: &MoveContext) -> String {
    let locale = get_default_locale();
    let language = get_default_language();
    mf(node, context, locale, language)
}

#[test]
fn test_move_formula() {
    // top left corner C2
    let row = 2;
    let column = 3;
    let context = &CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row,
        column,
    };
    let worksheets = vec!["Sheet1".to_string()];
    let mut parser = new_parser(worksheets, vec![], HashMap::new());

    // Area is C2:F6
    let area = &Area {
        sheet: 0,
        row,
        column,
        width: 4,
        height: 5,
    };

    // formula AB31 will not change
    let node = parser.parse("AB31", context);
    let t = move_formula(
        &node,
        &MoveContext {
            source_sheet_name: "Sheet1",
            row,
            column,
            area,
            target_sheet_name: "Sheet1",
            row_delta: 10,
            column_delta: 10,
        },
    );
    assert_eq!(t, "AB31");

    // formula $AB$31 will not change
    let node = parser.parse("AB31", context);
    let t = move_formula(
        &node,
        &MoveContext {
            source_sheet_name: "Sheet1",
            row,
            column,
            area,
            target_sheet_name: "Sheet1",
            row_delta: 10,
            column_delta: 10,
        },
    );
    assert_eq!(t, "AB31");

    // but formula D5 will change to N15  (N = D + 10)
    let node = parser.parse("D5", context);
    let t = move_formula(
        &node,
        &MoveContext {
            source_sheet_name: "Sheet1",
            row,
            column,
            area,
            target_sheet_name: "Sheet1",
            row_delta: 10,
            column_delta: 10,
        },
    );
    assert_eq!(t, "N15");

    // Also formula $D$5 will change to N15  (N = D + 10)
    let node = parser.parse("$D$5", context);
    let t = move_formula(
        &node,
        &MoveContext {
            source_sheet_name: "Sheet1",
            row,
            column,
            area,
            target_sheet_name: "Sheet1",
            row_delta: 10,
            column_delta: 10,
        },
    );
    assert_eq!(t, "$N$15");
}

#[test]
fn test_move_formula_context_offset() {
    // context is E4
    let row = 4;
    let column = 5;
    let context = &CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row,
        column,
    };
    let worksheets = vec!["Sheet1".to_string()];
    let mut parser = new_parser(worksheets, vec![], HashMap::new());

    // Area is C2:F6
    let area = &Area {
        sheet: 0,
        row: 2,
        column: 3,
        width: 4,
        height: 5,
    };

    let node = parser.parse("-X9+C2%", context);
    let t = move_formula(
        &node,
        &MoveContext {
            source_sheet_name: "Sheet1",
            row,
            column,
            area,
            target_sheet_name: "Sheet1",
            row_delta: 10,
            column_delta: 10,
        },
    );
    assert_eq!(t, "-X9+M12%");
}

#[test]
fn test_move_formula_lambda_def() {
    // context E4, area C2:F6, delta (10, 10)
    // C2 is inside area → becomes M12
    // X9 is outside area → unchanged
    let row = 4;
    let column = 5;
    let context = &CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row,
        column,
    };
    let worksheets = vec!["Sheet1".to_string()];
    let mut parser = new_parser(worksheets, vec![], HashMap::new());

    let area = &Area {
        sheet: 0,
        row: 2,
        column: 3,
        width: 4,
        height: 5,
    };

    let node = parser.parse("LAMBDA(x, C2 + x)", context);
    let t = move_formula(
        &node,
        &MoveContext {
            source_sheet_name: "Sheet1",
            row,
            column,
            area,
            target_sheet_name: "Sheet1",
            row_delta: 10,
            column_delta: 10,
        },
    );
    // C2 is in area → M12; parameter x and the body's x are unchanged
    assert_eq!(t, "LAMBDA(x, M12 + x)");

    // Reference outside the area stays put
    let node = parser.parse("LAMBDA(y, X9 * y)", context);
    let t = move_formula(
        &node,
        &MoveContext {
            source_sheet_name: "Sheet1",
            row,
            column,
            area,
            target_sheet_name: "Sheet1",
            row_delta: 10,
            column_delta: 10,
        },
    );
    assert_eq!(t, "LAMBDA(y, X9 * y)");
}

#[test]
fn test_move_formula_lambda_call() {
    // context E4, area C2:F6, delta (10, 10)
    // C2 and D5 are inside area → M12, N15
    // X9 is outside → unchanged
    let row = 4;
    let column = 5;
    let context = &CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row,
        column,
    };
    let worksheets = vec!["Sheet1".to_string()];
    let mut parser = new_parser(worksheets, vec![], HashMap::new());

    let area = &Area {
        sheet: 0,
        row: 2,
        column: 3,
        width: 4,
        height: 5,
    };

    // Immediately-invoked LAMBDA: body ref and call arg both in area
    let node = parser.parse("LAMBDA(x, C2 + x)(D5)", context);
    let t = move_formula(
        &node,
        &MoveContext {
            source_sheet_name: "Sheet1",
            row,
            column,
            area,
            target_sheet_name: "Sheet1",
            row_delta: 10,
            column_delta: 10,
        },
    );
    assert_eq!(t, "LAMBDA(x, M12 + x)(N15)");

    // Call arg outside area → unchanged
    let node = parser.parse("LAMBDA(x, x + 1)(X9)", context);
    let t = move_formula(
        &node,
        &MoveContext {
            source_sheet_name: "Sheet1",
            row,
            column,
            area,
            target_sheet_name: "Sheet1",
            row_delta: 10,
            column_delta: 10,
        },
    );
    assert_eq!(t, "LAMBDA(x, x + 1)(X9)");
}

#[test]
fn test_move_formula_area_limits() {
    // context is E4
    let row = 4;
    let column = 5;
    let context = &CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row,
        column,
    };
    let worksheets = vec!["Sheet1".to_string()];
    let mut parser = new_parser(worksheets, vec![], HashMap::new());

    // Area is C2:F6
    let area = &Area {
        sheet: 0,
        row: 2,
        column: 3,
        width: 4,
        height: 5,
    };

    // Outside of the area. Not moved
    let node = parser.parse("B2+B3+C1+G6+H5", context);
    let t = move_formula(
        &node,
        &MoveContext {
            source_sheet_name: "Sheet1",
            row,
            column,
            area,
            target_sheet_name: "Sheet1",
            row_delta: 10,
            column_delta: 10,
        },
    );
    assert_eq!(t, "B2+B3+C1+G6+H5");

    // In the area. Moved
    let node = parser.parse("C2+F4+F5+F6", context);
    let t = move_formula(
        &node,
        &MoveContext {
            source_sheet_name: "Sheet1",
            row,
            column,
            area,
            target_sheet_name: "Sheet1",
            row_delta: 10,
            column_delta: 10,
        },
    );
    assert_eq!(t, "M12+P14+P15+P16");
}

#[test]
fn test_move_formula_ranges() {
    // top left corner C2
    let row = 2;
    let column = 3;
    let context = &CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row,
        column,
    };
    let worksheets = vec!["Sheet1".to_string()];
    let mut parser = new_parser(worksheets, vec![], HashMap::new());

    let area = &Area {
        sheet: 0,
        row,
        column,
        width: 4,
        height: 5,
    };
    // Ranges inside the area are fully displaced (absolute or not)
    let node = parser.parse("SUM(C2:F5)", context);
    let t = move_formula(
        &node,
        &MoveContext {
            source_sheet_name: "Sheet1",
            row,
            column,
            area,
            target_sheet_name: "Sheet1",
            row_delta: 10,
            column_delta: 10,
        },
    );
    assert_eq!(t, "SUM(M12:P15)");

    let node = parser.parse("SUM($C$2:$F$5)", context);
    let t = move_formula(
        &node,
        &MoveContext {
            source_sheet_name: "Sheet1",
            row,
            column,
            area,
            target_sheet_name: "Sheet1",
            row_delta: 10,
            column_delta: 10,
        },
    );
    assert_eq!(t, "SUM($M$12:$P$15)");

    // Ranges completely outside of the area are not touched
    let node = parser.parse("SUM(A1:B3)", context);
    let t = move_formula(
        &node,
        &MoveContext {
            source_sheet_name: "Sheet1",
            row,
            column,
            area,
            target_sheet_name: "Sheet1",
            row_delta: 10,
            column_delta: 10,
        },
    );
    assert_eq!(t, "SUM(A1:B3)");

    let node = parser.parse("SUM($A$1:$B$3)", context);
    let t = move_formula(
        &node,
        &MoveContext {
            source_sheet_name: "Sheet1",
            row,
            column,
            area,
            target_sheet_name: "Sheet1",
            row_delta: 10,
            column_delta: 10,
        },
    );
    assert_eq!(t, "SUM($A$1:$B$3)");

    // Ranges that overlap with the area are also NOT displaced
    let node = parser.parse("SUM(A1:F5)", context);
    let t = move_formula(
        &node,
        &MoveContext {
            source_sheet_name: "Sheet1",
            row,
            column,
            area,
            target_sheet_name: "Sheet1",
            row_delta: 10,
            column_delta: 10,
        },
    );
    assert_eq!(t, "SUM(A1:F5)");

    // Ranges that contain the area are also NOT displaced
    let node = parser.parse("SUM(A1:X50)", context);
    let t = move_formula(
        &node,
        &MoveContext {
            source_sheet_name: "Sheet1",
            row,
            column,
            area,
            target_sheet_name: "Sheet1",
            row_delta: 10,
            column_delta: 10,
        },
    );
    assert_eq!(t, "SUM(A1:X50)");
}

#[test]
fn test_move_formula_wrong_reference() {
    // context is E4
    let row = 4;
    let column = 5;
    let context = &CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row,
        column,
    };
    // Area is C2:G5
    let area = &Area {
        sheet: 0,
        row: 2,
        column: 3,
        width: 4,
        height: 5,
    };
    let worksheets = vec!["Sheet1".to_string()];
    let mut parser = new_parser(worksheets, vec![], HashMap::new());

    // Wrong formulas will NOT be displaced; a missing sheet prints as `#REF!` (T-2340)
    let node = parser.parse("Sheet3!AB31", context);
    let t = move_formula(
        &node,
        &MoveContext {
            source_sheet_name: "Sheet1",
            row,
            column,
            area,
            target_sheet_name: "Sheet1",
            row_delta: 10,
            column_delta: 10,
        },
    );
    assert_eq!(t, "#REF!AB31");
    let node = parser.parse("Sheet3!$X$9", context);
    let t = move_formula(
        &node,
        &MoveContext {
            source_sheet_name: "Sheet1",
            row,
            column,
            area,
            target_sheet_name: "Sheet1",
            row_delta: 10,
            column_delta: 10,
        },
    );
    assert_eq!(t, "#REF!$X$9");

    let node = parser.parse("SUM(Sheet3!D2:D3)", context);
    let t = move_formula(
        &node,
        &MoveContext {
            source_sheet_name: "Sheet1",
            row,
            column,
            area,
            target_sheet_name: "Sheet1",
            row_delta: 10,
            column_delta: 10,
        },
    );
    assert_eq!(t, "SUM(#REF!D2:D3)");
}

#[test]
fn test_move_formula_misc() {
    // context is E4
    let row = 4;
    let column = 5;
    let context = &CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row,
        column,
    };
    let worksheets = vec!["Sheet1".to_string()];
    let mut parser = new_parser(worksheets, vec![], HashMap::new());

    // Area is C2:F6
    let area = &Area {
        sheet: 0,
        row: 2,
        column: 3,
        width: 4,
        height: 5,
    };
    let node = parser.parse("X9^C2-F4*H2+SUM(F2:H4)+SUM(C2:F6)", context);
    let t = move_formula(
        &node,
        &MoveContext {
            source_sheet_name: "Sheet1",
            row,
            column,
            area,
            target_sheet_name: "Sheet1",
            row_delta: 10,
            column_delta: 10,
        },
    );
    assert_eq!(t, "X9^M12-P14*H2+SUM(F2:H4)+SUM(M12:P16)");

    let node = parser.parse("F5*(-D5)*SUM(A1, X9, $D$5)", context);
    let t = move_formula(
        &node,
        &MoveContext {
            source_sheet_name: "Sheet1",
            row,
            column,
            area,
            target_sheet_name: "Sheet1",
            row_delta: 10,
            column_delta: 10,
        },
    );
    assert_eq!(t, "P15*(-N15)*SUM(A1, X9, $N$15)");

    let node = parser.parse("IF(F5 < -D5, X9 & F5, FALSE)", context);
    let t = move_formula(
        &node,
        &MoveContext {
            source_sheet_name: "Sheet1",
            row,
            column,
            area,
            target_sheet_name: "Sheet1",
            row_delta: 10,
            column_delta: 10,
        },
    );
    assert_eq!(t, "IF(P15 < -N15, X9 & P15, FALSE)");
}

#[test]
fn test_move_formula_another_sheet() {
    // top left corner C2
    let row = 2;
    let column = 3;
    let context = &CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row,
        column,
    };
    // we add two sheets and we cut/paste from Sheet1 to Sheet2
    let worksheets = vec!["Sheet1".to_string(), "Sheet2".to_string()];
    let mut parser = new_parser(worksheets, vec![], HashMap::new());

    // Area is C2:F6
    let area = &Area {
        sheet: 0,
        row,
        column,
        width: 4,
        height: 5,
    };

    // Formula AB31 and JJ3:JJ4 refers to original Sheet1!AB31 and Sheet1!JJ3:JJ4
    let node = parser.parse("AB31*SUM(JJ3:JJ4)+SUM(Sheet2!C2:F6)*SUM(C2:F6)", context);
    let t = move_formula(
        &node,
        &MoveContext {
            source_sheet_name: "Sheet1",
            row,
            column,
            area,
            target_sheet_name: "Sheet2",
            row_delta: 10,
            column_delta: 10,
        },
    );
    assert_eq!(
        t,
        "Sheet1!AB31*SUM(Sheet1!JJ3:JJ4)+SUM(Sheet2!C2:F6)*SUM(M12:P16)"
    );
}

#[test]
fn move_formula_implicit_intersection() {
    // context is E4
    let row = 4;
    let column = 5;
    let context = &CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row,
        column,
    };
    let worksheets = vec!["Sheet1".to_string()];
    let mut parser = new_parser(worksheets, vec![], HashMap::new());

    // Area is C2:F6
    let area = &Area {
        sheet: 0,
        row: 2,
        column: 3,
        width: 4,
        height: 5,
    };
    let node = parser.parse("SUM(@F2:H4)+SUM(@C2:F6)", context);
    let t = move_formula(
        &node,
        &MoveContext {
            source_sheet_name: "Sheet1",
            row,
            column,
            area,
            target_sheet_name: "Sheet1",
            row_delta: 10,
            column_delta: 10,
        },
    );
    assert_eq!(t, "SUM(@F2:H4)+SUM(@M12:P16)");
}

#[test]
fn move_formula_implicit_intersection_with_ranges() {
    // context is E4
    let row = 4;
    let column = 5;
    let context = &CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row,
        column,
    };
    let worksheets = vec!["Sheet1".to_string()];
    let mut parser = new_parser(worksheets, vec![], HashMap::new());

    // Area is C2:F6
    let area = &Area {
        sheet: 0,
        row: 2,
        column: 3,
        width: 4,
        height: 5,
    };
    let node = parser.parse("SUM(@F2:H4)+SUM(@C2:F6)+SUM(@A1, @X9, @$D$5)", context);
    let t = move_formula(
        &node,
        &MoveContext {
            source_sheet_name: "Sheet1",
            row,
            column,
            area,
            target_sheet_name: "Sheet1",
            row_delta: 10,
            column_delta: 10,
        },
    );
    assert_eq!(t, "SUM(@F2:H4)+SUM(@M12:P16)+SUM(@A1, @X9, @$N$15)");
}

#[test]
fn test_move_formula_at_intersection_prints_as_excel() {
    // Same printing as `stringify`: probes/intersection-at-formulatext (T-1098). A move that
    // changes nothing leaves the text as written.
    let context = &CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 1,
        column: 1,
    };
    let mut parser = new_parser(vec!["Sheet1".to_string()], vec![], HashMap::new());
    let area = &Area {
        sheet: 0,
        row: 1,
        column: 1,
        width: 1,
        height: 1,
    };
    for formula in ["A1 @A1", "@A1:A3 A2:B2", "(A1 @A1) B1"] {
        let node = parser.parse(formula, context);
        let t = move_formula(
            &node,
            &MoveContext {
                source_sheet_name: "Sheet1",
                row: 1,
                column: 1,
                area,
                target_sheet_name: "Sheet1",
                row_delta: 0,
                column_delta: 0,
            },
        );
        assert_eq!(t, formula);
    }
}

/// Every formula of the probe corpus, as `(sheet names, cell address, formula)`.
fn probe_corpus_formulas() -> Vec<(Vec<String>, String, String)> {
    let cases =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../eval/suites/probes/cases");
    let mut formulas = Vec::new();
    let Ok(entries) = std::fs::read_dir(&cases) else {
        panic!("no probe corpus at {}", cases.display());
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.to_string_lossy().ends_with(".case.json") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            panic!("cannot read {}", path.display());
        };
        let Ok(case) = serde_json::from_str::<serde_json::Value>(&text) else {
            panic!("not JSON: {}", path.display());
        };
        let sheets = case["sheets"].as_array().cloned().unwrap_or_default();
        let names: Vec<String> = sheets
            .iter()
            .filter_map(|sheet| sheet["name"].as_str().map(str::to_string))
            .collect();
        for sheet in &sheets {
            let Some(cells) = sheet["cells"].as_object() else {
                continue;
            };
            for (address, value) in cells {
                if let Some(formula) = value.as_str().and_then(|v| v.strip_prefix('=')) {
                    formulas.push((names.clone(), address.clone(), formula.to_string()));
                }
            }
        }
    }
    formulas
}

/// A cut and paste prints the formula as FORMULATEXT does: moving it by (0, 0) changes no
/// text (SPEC-0007 step 5, T-1344). Before, the moved text had its own precedence table, so
/// `=SUM(A1)^2` printed differently in the two.
#[test]
fn moving_by_nothing_prints_the_formula_text_over_the_probe_corpus() {
    let formulas = probe_corpus_formulas();
    assert!(formulas.len() > 1000, "probe corpus: {}", formulas.len());
    for (sheets, address, formula) in formulas {
        let Some(cell) = crate::expressions::utils::parse_reference_a1(&address) else {
            continue;
        };
        let sheet = sheets.first().cloned().unwrap_or_default();
        let context = CellReferenceRC {
            sheet: sheet.clone(),
            row: cell.row,
            column: cell.column,
        };
        let mut parser = new_parser(sheets, vec![], HashMap::new());
        let node = parser.parse(&formula, &context);
        let area = Area {
            sheet: 0,
            row: cell.row,
            column: cell.column,
            width: 1,
            height: 1,
        };
        let moved = move_formula(
            &node,
            &MoveContext {
                source_sheet_name: &sheet,
                row: cell.row,
                column: cell.column,
                area: &area,
                target_sheet_name: &sheet,
                row_delta: 0,
                column_delta: 0,
            },
        );
        assert_eq!(moved, to_english_string(&node, &context), "={formula}");
    }
}
