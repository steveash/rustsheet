#![allow(clippy::unwrap_used)]

//! SUMPRODUCT's `block*mask*...` argument read from the cached block (T-3230) gives what the
//! operators' arrays give: values, errors in masked-out cells, mask errors and shapes that are no
//! masks.

use crate::expressions::types::CellReferenceIndex;
use crate::test::util::new_empty_model;
use crate::Model;

/// Sheet2: keys in A2:A41 (clean) and P2:P41 (an error at row 30), headers in B1:G1 (clean) and
/// I1:N1 (an error at L1), a block of numbers, booleans, blanks and numeric text in B2:G41, and
/// the same with a text, `#DIV/0!` and `#NUM!` in I2:N41. Sheet3!B2:G41 holds formulas that are
/// not evaluated yet when Sheet1's first call reads them.
fn fill(model: &mut Model) {
    model.new_sheet();
    model.new_sheet();
    for (index, column) in ["B", "C", "D", "E", "F", "G"].iter().enumerate() {
        model._set(&format!("Sheet2!{column}1"), &format!("h{}", index % 3));
    }
    for (index, column) in ["I", "J", "K", "L", "M", "N"].iter().enumerate() {
        let header = if *column == "L" {
            "=NA()".to_string()
        } else {
            format!("h{}", index % 3)
        };
        model._set(&format!("Sheet2!{column}1"), &header);
    }
    for row in 2..=41 {
        model._set(&format!("Sheet2!A{row}"), &format!("k{}", row % 5));
        let key = if row == 30 {
            "=1/0".to_string()
        } else {
            format!("k{}", row % 5)
        };
        model._set(&format!("Sheet2!P{row}"), &key);
        for (index, (clean, dirty)) in ["B", "C", "D", "E", "F", "G"]
            .iter()
            .zip(["I", "J", "K", "L", "M", "N"])
            .enumerate()
        {
            let value = match (row + index) % 11 {
                3 => String::new(),
                5 => "TRUE".to_string(),
                7 => "'7".to_string(),
                _ => format!("{}", (row as f64 * (index as f64 + 1.0) - 50.0) / 8.0),
            };
            model._set(&format!("Sheet2!{clean}{row}"), &value);
            let dirty_value = match (row, dirty) {
                (10, "J") => "abc".to_string(),
                (35, "M") => "=1/0".to_string(),
                (5, "N") => "=SQRT(-1)".to_string(),
                _ => value.clone(),
            };
            model._set(&format!("Sheet2!{dirty}{row}"), &dirty_value);
            model._set(
                &format!("Sheet3!{clean}{row}"),
                &format!("=Sheet2!{clean}{row}*2"),
            );
        }
    }
}

/// `fill`, and errors placed to meet at one position: Sheet2!Q2:V41 holds numbers but `#NUM!` at
/// Q30 (the row of P30's `#DIV/0!`), X2:AC41 numbers but `#NUM!` at Y10 (between row 30's first
/// cell and P30's error, counting row by row), Q1:V1 headers but `#N/A` at Q1, and O2:O41 keys but
/// `#DIV/0!` at O2.
fn fill_meeting_errors(model: &mut Model) {
    fill(model);
    let columns = ["Q", "R", "S", "T", "U", "V"];
    for (index, column) in columns.iter().enumerate() {
        let header = if index == 0 {
            "=NA()".to_string()
        } else {
            format!("h{}", index % 3)
        };
        model._set(&format!("Sheet2!{column}1"), &header);
    }
    for row in 2..=41 {
        let key = if row == 2 {
            "=1/0".to_string()
        } else {
            format!("k{}", row % 5)
        };
        model._set(&format!("Sheet2!O{row}"), &key);
        for (index, (q, x)) in columns
            .iter()
            .zip(["X", "Y", "Z", "AA", "AB", "AC"])
            .enumerate()
        {
            let number = format!("{}", row * 10 + index);
            let error = "=SQRT(-1)".to_string();
            let q_value = if (row, *q) == (30, "Q") {
                &error
            } else {
                &number
            };
            let x_value = if (row, x) == (10, "Y") {
                &error
            } else {
                &number
            };
            model._set(&format!("Sheet2!{q}{row}"), q_value);
            model._set(&format!("Sheet2!{x}{row}"), x_value);
        }
    }
}

/// Errors of the block and of masks that meet at one position: the block's error wins there,
/// then the masks' in operator order; a row mask's error is first met at the start of its row.
#[test]
fn errors_meeting_at_one_position() {
    let clean_h = "(Sheet2!$B$1:$G$1=\"h1\")";
    let error_h = "(Sheet2!$Q$1:$V$1=\"h1\")";
    let p_key = "(Sheet2!$P$2:$P$41=\"k2\")";
    let o_key = "(Sheet2!$O$2:$O$41=\"k2\")";
    // Evaluates the blocks' formulas, so that every call below reads them cached.
    let mut formulas = vec!["=COUNT(Sheet2!$Q$2:$V$41,Sheet2!$X$2:$AC$41)".to_string()];
    for block in ["Sheet2!$Q$2:$V$41", "Sheet2!$X$2:$AC$41"] {
        for (h, k) in [(clean_h, p_key), (error_h, o_key), (error_h, p_key)] {
            formulas.push(format!("=SUMPRODUCT({block}*{h}*{k})"));
            formulas.push(format!("=SUMPRODUCT({block}*{k}*{h})"));
        }
    }
    let masked = assert_alike(&fill_meeting_errors, &formulas);
    assert_eq!(masked, formulas.len() - 1);
}

/// Each formula's text, and whether it is marked circular, with the pass caches on and off, and
/// how many arguments the cached run read as masked blocks.
fn cached_and_walked(
    fill: &dyn Fn(&mut Model),
    formulas: &[String],
) -> (Vec<(String, String)>, usize) {
    let run = |no_pass_caches: bool| {
        let mut model = new_empty_model();
        model.evaluation.no_pass_caches = no_pass_caches;
        fill(&mut model);
        for (index, formula) in formulas.iter().enumerate() {
            model._set(&format!("A{}", index + 1), formula);
        }
        model.evaluate();
        let texts: Vec<String> = (1..=formulas.len())
            .map(|row| {
                let at = CellReferenceIndex {
                    sheet: 0,
                    row: i32::try_from(row).unwrap(),
                    column: 1,
                };
                let circular = model
                    .evaluation
                    .circular
                    .contains(&crate::evaluation::key(at));
                format!(
                    "{} circular={circular}",
                    model._get_text(&format!("A{row}"))
                )
            })
            .collect();
        (texts, model.evaluation.ifs_cache.masked)
    };
    let (cached, masked) = run(false);
    let (walked, _) = run(true);
    (cached.into_iter().zip(walked).collect(), masked)
}

fn assert_alike(fill: &dyn Fn(&mut Model), formulas: &[String]) -> usize {
    let (results, masked) = cached_and_walked(fill, formulas);
    for ((cached, walked), formula) in results.iter().zip(formulas) {
        assert_eq!(cached, walked, "{formula}");
    }
    masked
}

/// Every block, header row, key column, criterion and operand order: errors and text anywhere in
/// the block, errors in either mask, and their positions against each other.
#[test]
fn masked_blocks_match_the_operators() {
    let mut formulas = Vec::new();
    for block in [
        "Sheet2!$B$2:$G$41",
        "Sheet2!$I$2:$N$41",
        "Sheet3!$B$2:$G$41",
    ] {
        for header in ["Sheet2!$B$1:$G$1", "Sheet2!$I$1:$N$1"] {
            for key in ["Sheet2!$A$2:$A$41", "Sheet2!$P$2:$P$41"] {
                for x in ["\"h0\"", "\"h2\"", "\"zz\""] {
                    for y in ["\"k0\"", "\"k3\"", "\"none\""] {
                        let h = format!("({header}={x})");
                        let k = format!("({key}={y})");
                        for argument in [
                            format!("{block}*{h}*{k}"),
                            format!("{block}*{k}*{h}"),
                            format!("{block}*{h}"),
                            format!("({block}*{k})"),
                            format!("{block}*{k}*{h}*{k}"),
                        ] {
                            formulas.push(format!("=SUMPRODUCT({argument})"));
                        }
                    }
                }
            }
        }
    }
    let masked = assert_alike(&fill, &formulas);
    assert!(masked >= 400, "the masked path ran {masked} times");
}

/// Operands that are no masks (a mismatched width, a 2-D or one-cell range, a number, a mask
/// already multiplied out, a text comparison result), other arguments beside a masked one, and a
/// block too small to cache: the operators run as before.
#[test]
fn operands_that_are_no_masks() {
    let h = "(Sheet2!$B$1:$G$1=\"h1\")";
    let k = "(Sheet2!$A$2:$A$41=\"k2\")";
    let block = "Sheet2!$B$2:$G$41";
    let formulas: Vec<String> = [
        format!("=SUMPRODUCT({block}*(Sheet2!$B$1:$F$1=\"h1\")*{k})"),
        format!("=SUMPRODUCT({block}*{k}*(Sheet2!$A$2:$A$40=\"k2\"))"),
        format!("=SUMPRODUCT({block}*{h}*Sheet2!$I$2:$N$41)"),
        format!("=SUMPRODUCT({block}*{h}*Sheet2!$B$2)"),
        format!("=SUMPRODUCT({block}*{h}*2)"),
        format!("=SUMPRODUCT({block}*({h}*1)*{k})"),
        format!("=SUMPRODUCT({block}*{h}*(Sheet2!$A$2:$A$41&\"\"))"),
        format!("=SUMPRODUCT({block}*{h}*{{1,2,3,4,5,6}})"),
        format!("=SUMPRODUCT({block}*{h}*{k},{block})"),
        format!("=SUMPRODUCT({block},{block}*{h}*{k})"),
        format!("=SUMPRODUCT({block}*{h}*{k},Sheet2!$B$2:$G$40)"),
        format!("=SUMPRODUCT(Sheet2!$I$2:$N$41*{h},{block}*{k})"),
        format!("=IFERROR(SUMPRODUCT(Sheet2!$I$2:$N$41*{h}*{k}),\"\")"),
        format!("=SUMPRODUCT(Sheet2!$B$2:$C$9*(Sheet2!$B$1:$C$1=\"h1\"))"),
        format!("=SUMPRODUCT({h}*{block}*{k})"),
        format!("=SUMPRODUCT({block}/{h}*{k})"),
    ]
    .into_iter()
    .collect();
    let masked = assert_alike(&fill, &formulas);
    assert!(masked >= 6, "the masked path ran {masked} times");
}

/// A block cell that reads the call back: the cell is circular, so the block is never cached and
/// the walk reports the cycle as before.
#[test]
fn a_cycle_through_the_block() {
    let with_cycle = |model: &mut Model| {
        fill(model);
        model._set("Sheet2!D20", "=Sheet1!A1");
    };
    let formulas: Vec<String> = (0..3)
        .map(|_| {
            "=SUMPRODUCT(Sheet2!$B$2:$G$41*(Sheet2!$B$1:$G$1=\"h1\")*(Sheet2!$A$2:$A$41=\"k0\"))"
                .to_string()
        })
        .collect();
    assert_alike(&with_cycle, &formulas);
}

/// One value worked by hand: columns D and G (header h2) at the rows keyed k0 (5, 10, ..., 40),
/// each cell as `fill` writes it (a blank 0, TRUE 1, the text "7" 7).
#[test]
fn a_masked_sum_by_hand() {
    let mut model = new_empty_model();
    fill(&mut model);
    model._set(
        "A1",
        "=SUMPRODUCT(Sheet2!$B$2:$G$41*(Sheet2!$B$1:$G$1=\"h2\")*(Sheet2!$A$2:$A$41=\"k0\"))",
    );
    model.evaluate();
    let expected: f64 = (5..=40)
        .step_by(5)
        .flat_map(|row: i32| [2, 5].map(|index| (row, index)))
        .map(|(row, index)| match (row + index) % 11 {
            3 => 0.0,
            5 => 1.0,
            7 => 7.0,
            _ => f64::from(row * (index + 1) - 50) / 8.0,
        })
        .sum();
    assert_eq!(model._get_text("A1"), format!("{expected}"));
    assert_eq!(model.evaluation.ifs_cache.masked, 1);
}
