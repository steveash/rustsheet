//! Timings for SPEC-0009's aggregate-argument walker: SUM and AVERAGE over a numeric column of
//! 1,000,000 cells, SUM over the same column with an error in A2 (the walk ends there), and SUM
//! over a block three columns wide reaching past the used area (T-0405: cut at the used area),
//! and SUM over the whole sheet with a far cell in XFD1048576 (T-1690: walked over its cells).
//! Then a 3-D TEXTJOIN over whole columns beside a far cell, and the same SUM repeated by a
//! LAMBDA (T-2710: blank runs counted, not read).
//! Then criteria aggregates over shared ranges (T-0449): 2,000 SUMIFS and 2,000 COUNTIFS over
//! 50,000 rows, with five text criteria and 100 numeric thresholds. Then aggregates over
//! whole-column expressions (T-0469): 400 `MAX(IF(F:F=A1,G:G,""))` over 2,000 rows. Then criteria
//! aggregates with a new criterion per call (T-0465): 10,500 ranks, each two COUNTIFS (one over a
//! growing prefix), and 12,000 per-row SUMIFS over 1,500 rows. Then SUMPRODUCT over whole-column
//! expressions (T-0471): 400 `SUMPRODUCT((F:F=A1)*G:G)` over 2,000 rows. Last, the three
//! consumers the SPEC-0020 amendment tails next (T-0488), each over 2,000 rows: UNIQUE under
//! TEXTJOIN, LOOKUP's last-non-empty idiom, and SMALL over a 70,000-row range with `ROW()`, and
//! a count through VLOOKUP over a whole-column key (T-3221) and INDEX of MATCH over one (T-3222).
//! Then calipers `pv23_042`'s shape (T-0498): 1,000 four-criteria SUMIFS over formula columns of
//! 20,000 rows that the calls themselves evaluate.
//! `cargo bench -p rustsheet --bench aggregate` (`-- criteria`, `-- whole_column`, `-- rank`,
//! `-- sumproduct` and `-- chained` run only the T-0449, T-0469, T-0465, T-0471 or T-0498 case,
//! `-- tailed_consumers` only the T-0488 cases and T-3221's lookup over a tailed key)
//!
//! One warm-up run, then `RUNS` measured recalculations; each case prints the fastest, the median
//! and the spread.

mod common;

use std::error::Error;
use std::io::Write;

use rustsheet::{Input, Workbook};

const ROWS: u32 = 1_000_000;
const RUNS: usize = 5;

/// A workbook with 1..=ROWS in column A (A2 an error when `error_in_a2`, and 1 in XFD1048576
/// when `far_cell`) and `formula` in B1.
fn workbook(formula: &str, error_in_a2: bool, far_cell: bool) -> Result<Workbook, Box<dyn Error>> {
    common::filled_workbook(|tx| {
        for r in 1..=ROWS {
            tx.set(format!("A{r}").as_str(), f64::from(r))?;
        }
        if error_in_a2 {
            tx.set("A2", Input::Formula("=NA()".to_string()))?;
        }
        if far_cell {
            tx.set("XFD1048576", 1.0)?;
        }
        tx.set("B1", Input::Formula(formula.to_string()))?;
        Ok(())
    })
}

/// Two sheets each holding only 1 in J1048576, with `formula` in Sheet1!Z1: every whole-column
/// walk of `A:J` spans the sheet height over one stored cell.
fn far_cell_workbook(formula: &str) -> Result<Workbook, Box<dyn Error>> {
    common::filled_workbook(|tx| {
        tx.add_sheet("Two")?;
        tx.set("Sheet1!J1048576", 1.0)?;
        tx.set("Two!J1048576", 1.0)?;
        tx.set("Sheet1!Z1", Input::Formula(formula.to_string()))?;
        Ok(())
    })
}

/// Rows of the criteria ranges of the T-0449 case.
const CRITERIA_ROWS: u32 = 50_000;
/// SUMIFS calls, and as many COUNTIFS.
const CRITERIA_CALLS: u32 = 2_000;
const REGIONS: [&str; 5] = ["North", "South", "East", "West", "Central"];

/// A region in A, a tag in B (half before "t", half after) and an amount in C, with
/// `CRITERIA_CALLS` SUMIFS in E and as many COUNTIFS in F.
fn criteria_workbook() -> Result<Workbook, Box<dyn Error>> {
    common::filled_workbook(|tx| {
        for r in 1..=CRITERIA_ROWS {
            let region = REGIONS[(r % 5) as usize];
            let tag = format!("{}{r}", if r % 2 == 0 { "alpha" } else { "zeta" });
            tx.set(format!("A{r}").as_str(), region)?;
            tx.set(format!("B{r}").as_str(), tag.as_str())?;
            tx.set(format!("C{r}").as_str(), f64::from((r * 37) % 1000))?;
        }
        let n = CRITERIA_ROWS;
        for r in 1..=CRITERIA_CALLS {
            let region = REGIONS[(r % 5) as usize];
            let threshold = (r % 100) * 10;
            let sumifs =
                format!("=SUMIFS($C$1:$C${n},$A$1:$A${n},\"{region}\",$B$1:$B${n},\">t\")");
            let countifs =
                format!("=COUNTIFS($A$1:$A${n},\"{region}\",$C$1:$C${n},\">{threshold}\")");
            tx.set(format!("E{r}").as_str(), Input::Formula(sumifs))?;
            tx.set(format!("F{r}").as_str(), Input::Formula(countifs))?;
        }
        Ok(())
    })
}

/// Rows of data in the T-0469 case.
const WHOLE_COLUMN_ROWS: u32 = 2_000;
/// Whole-column MAX(IF(..)) calls.
const WHOLE_COLUMN_CALLS: u32 = 400;

/// A key in F (one of 50) and an amount in G, with `WHOLE_COLUMN_CALLS` keys in A and
/// `MAX(IF(F:F=A1,G:G,""))` beside each in B, as in corpus-bench 17049.
fn whole_column_workbook() -> Result<Workbook, Box<dyn Error>> {
    common::filled_workbook(|tx| {
        for r in 1..=WHOLE_COLUMN_ROWS {
            tx.set(format!("F{r}").as_str(), format!("k{}", r % 50).as_str())?;
            tx.set(format!("G{r}").as_str(), f64::from((r * 37) % 1000))?;
        }
        for r in 1..=WHOLE_COLUMN_CALLS {
            tx.set(format!("A{r}").as_str(), format!("k{}", r % 50).as_str())?;
            let formula = format!("=MAX(IF(F:F=A{r},G:G,\"\"))");
            tx.set(format!("B{r}").as_str(), Input::Formula(formula))?;
        }
        Ok(())
    })
}

/// The T-0469 data with `SUMPRODUCT((F:F=A1)*G:G)` beside each key in B instead, as in
/// corpus-bench 55049.
fn sumproduct_workbook() -> Result<Workbook, Box<dyn Error>> {
    common::filled_workbook(|tx| {
        for r in 1..=WHOLE_COLUMN_ROWS {
            tx.set(format!("F{r}").as_str(), format!("k{}", r % 50).as_str())?;
            tx.set(format!("G{r}").as_str(), f64::from((r * 37) % 1000))?;
        }
        for r in 1..=WHOLE_COLUMN_CALLS {
            tx.set(format!("A{r}").as_str(), format!("k{}", r % 50).as_str())?;
            let formula = format!("=SUMPRODUCT((F:F=A{r})*G:G)");
            tx.set(format!("B{r}").as_str(), Input::Formula(formula))?;
        }
        Ok(())
    })
}

/// Calls of each T-0488 shape: each one materializes 1,048,576-row arrays today.
const TAILED_CONSUMER_CALLS: u32 = 20;
/// Measured recalculations of each T-0488 shape (each takes seconds today).
const TAILED_CONSUMER_RUNS: usize = 3;

/// Corpus-bench 45896's shape: a key in Zord!A (one of 50) and a name in Zord!C (one of 20), and
/// `TAILED_CONSUMER_CALLS` keys in A with `TEXTJOIN(",",TRUE,UNIQUE(IF(A1=Zord!A:A,Zord!C:C,"")))`
/// beside each in B.
fn unique_textjoin_workbook() -> Result<Workbook, Box<dyn Error>> {
    common::filled_workbook(|tx| {
        tx.add_sheet("Zord")?;
        for r in 1..=WHOLE_COLUMN_ROWS {
            tx.set(
                format!("Zord!A{r}").as_str(),
                format!("k{}", r % 50).as_str(),
            )?;
            tx.set(
                format!("Zord!C{r}").as_str(),
                format!("n{}", r % 20).as_str(),
            )?;
        }
        for r in 1..=TAILED_CONSUMER_CALLS {
            tx.set(format!("A{r}").as_str(), format!("k{}", r % 50).as_str())?;
            let formula = format!("=TEXTJOIN(\",\",TRUE,UNIQUE(IF(A{r}=Zord!A:A,Zord!C:C,\"\")))");
            tx.set(format!("B{r}").as_str(), Input::Formula(formula))?;
        }
        Ok(())
    })
}

/// Corpus-bench 33935's shape: an amount in J, 2 in L1, and `TAILED_CONSUMER_CALLS` copies of
/// the last-non-empty idiom `IF($L$1>1,LOOKUP(2,1/(J:J<>""),J:J))` in N.
fn lookup_last_workbook() -> Result<Workbook, Box<dyn Error>> {
    common::filled_workbook(|tx| {
        for r in 1..=WHOLE_COLUMN_ROWS {
            tx.set(format!("J{r}").as_str(), f64::from((r * 37) % 1000))?;
        }
        tx.set("L1", 2.0)?;
        let formula = "=IF($L$1>1,LOOKUP(2,1/(J:J<>\"\"),J:J))";
        for r in 1..=TAILED_CONSUMER_CALLS {
            tx.set(
                format!("N{r}").as_str(),
                Input::Formula(formula.to_string()),
            )?;
        }
        Ok(())
    })
}

/// SPEC-0020 Amendment 2's lookup over a tailed key (T-3221): a key in A (one of 50), a table of
/// the 50 keys and 0 (which a blank key finds) in Data!A1:B51, and `TAILED_CONSUMER_CALLS` copies
/// of `SUM(--(VLOOKUP($A:$A,Data!$A$1:$B$51,2,FALSE)=x))` in C, each counting the keys of one
/// value x.
fn vlookup_count_workbook() -> Result<Workbook, Box<dyn Error>> {
    common::filled_workbook(|tx| {
        tx.add_sheet("Data")?;
        for r in 1..=50 {
            tx.set(format!("Data!A{r}").as_str(), format!("k{r}").as_str())?;
            tx.set(format!("Data!B{r}").as_str(), f64::from(r))?;
        }
        tx.set("Data!A51", 0.0)?;
        tx.set("Data!B51", 0.0)?;
        for r in 1..=WHOLE_COLUMN_ROWS {
            tx.set(
                format!("A{r}").as_str(),
                format!("k{}", r % 50 + 1).as_str(),
            )?;
        }
        for r in 1..=TAILED_CONSUMER_CALLS {
            let formula = format!(
                "=SUM(--(VLOOKUP($A:$A,Data!$A$1:$B$51,2,FALSE)={}))",
                r % 50 + 1
            );
            tx.set(format!("C{r}").as_str(), Input::Formula(formula))?;
        }
        Ok(())
    })
}

/// Corpus-bench 59969's shape: two keys in Report!A and Report!B (one of 5 and one of 4) and an
/// amount in Report!G, the two keys wanted in X1 and X2, and in Z the k-th matching amount for
/// k = 1 to `TAILED_CONSUMER_CALLS`, through SMALL over `ROW()` of 70,000-row ranges.
fn small_row_workbook() -> Result<Workbook, Box<dyn Error>> {
    common::filled_workbook(|tx| {
        tx.add_sheet("Report")?;
        for r in 2..=WHOLE_COLUMN_ROWS + 1 {
            tx.set(
                format!("Report!A{r}").as_str(),
                format!("a{}", r % 5).as_str(),
            )?;
            tx.set(
                format!("Report!B{r}").as_str(),
                format!("b{}", r % 4).as_str(),
            )?;
            tx.set(format!("Report!G{r}").as_str(), f64::from((r * 37) % 1000))?;
        }
        tx.set("X1", "a1")?;
        tx.set("X2", "b1")?;
        for r in 1..=TAILED_CONSUMER_CALLS {
            let formula = format!(
                "=IFERROR(INDEX(Report!G:G,SMALL(IF((Report!$A$2:$A$70000=$X$1)*(Report!$B$2:$B$70000=$X$2),ROW(Report!$A$2:$A$70000)),ROWS($Y$1:Y{r}))),\"\")"
            );
            tx.set(format!("Z{r}").as_str(), Input::Formula(formula))?;
        }
        Ok(())
    })
}

/// Rows of the formula columns of the T-0498 case.
const CHAINED_ROWS: u32 = 20_000;
/// Rows and columns of its SUMIFS calls (1,000).
const CHAINED_CALL_ROWS: u32 = 100;
const MONTHS: [&str; 10] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct",
];

/// Calipers `pv23_042`'s shape: on Data, a customer in A (one of 50, a constant) and a number in X,
/// and formulas of X: a person in B (all but 1% the same), a year in C (one of 10), a month in D
/// (one of 12) and an amount in E. On Sheet1, which the pass reaches first, 100 rows x 10 columns
/// of `SUMIFS(E,A,"c1",B,"p1",C,<the row's year>,D,<the column's month>)` over whole columns:
/// the calls evaluate B, C, D and E where the criteria before them match, and nothing else does.
fn chained_workbook() -> Result<Workbook, Box<dyn Error>> {
    common::filled_workbook(|tx| {
        tx.add_sheet("Data")?;
        for r in 1..=CHAINED_ROWS {
            tx.set(
                format!("Data!A{r}").as_str(),
                format!("c{}", r % 50).as_str(),
            )?;
            tx.set(format!("Data!X{r}").as_str(), f64::from(r))?;
            let formulas = [
                ("B", format!("=IF(MOD(X{r},100)=0,\"p2\",\"p1\")")),
                ("C", format!("=2015+MOD(X{r}*7,10)")),
                ("D", format!("=TEXT(DATE(2024,MOD(X{r},12)+1,1),\"mmm\")")),
                ("E", format!("=ROUND(X{r}*1.5,5)")),
            ];
            for (column, formula) in formulas {
                tx.set(
                    format!("Data!{column}{r}").as_str(),
                    Input::Formula(formula),
                )?;
            }
        }
        for r in 1..=CHAINED_CALL_ROWS {
            let year = 2015 + r % 10;
            for (column, month) in ["A", "B", "C", "D", "E", "F", "G", "H", "I", "J"]
                .iter()
                .zip(MONTHS)
            {
                let formula = format!(
                    "=SUMIFS(Data!$E:$E,Data!$A:$A,\"c1\",Data!$B:$B,\"p1\",Data!$C:$C,{year},Data!$D:$D,\"{month}\")"
                );
                tx.set(format!("{column}{r}").as_str(), Input::Formula(formula))?;
            }
        }
        Ok(())
    })
}

/// A bench workbook's builder.
type Build = fn() -> Result<Workbook, Box<dyn Error>>;

/// Times each T-0488 shape.
fn tailed_consumer_cases() -> Result<(), Box<dyn Error>> {
    let cases: [(&str, Build); 5] = [
        (
            "TEXTJOIN(\",\",TRUE,UNIQUE(IF(A1=Zord!A:A,Zord!C:C,\"\")))",
            unique_textjoin_workbook,
        ),
        (
            "IF($L$1>1,LOOKUP(2,1/(J:J<>\"\"),J:J))",
            lookup_last_workbook,
        ),
        (
            "SMALL(IF((A2:A70000=x)*(B2:B70000=y),ROW(A2:A70000)),k)",
            small_row_workbook,
        ),
        (
            "SUM(--(VLOOKUP($A:$A,Data!$A$1:$B$51,2,FALSE)=x))",
            vlookup_count_workbook,
        ),
        (
            "INDEX(MATCH($A:$A,Data!$A$1:$A$50,0),n)",
            index_match_workbook,
        ),
    ];
    for (name, build) in cases {
        let mut wb = build()?;
        let t = common::timings(TAILED_CONSUMER_RUNS, || wb.recalculate());
        writeln!(
            std::io::stdout(),
            "{name}: {TAILED_CONSUMER_CALLS} calls over {WHOLE_COLUMN_ROWS} rows, runs={TAILED_CONSUMER_RUNS} {t}"
        )?;
    }
    Ok(())
}

/// Rows of data in the T-0465 case.
const RANK_ROWS: u32 = 1_500;
/// Score columns in the T-0465 case, A to G.
const RANK_COLUMNS: [&str; 7] = ["A", "B", "C", "D", "E", "F", "G"];
/// The rank formulas' columns, one per score column.
const RANK_OUTPUTS: [&str; 7] = ["N", "O", "P", "Q", "R", "S", "T"];
/// The per-row SUMIFS formulas' columns, one per score column.
const SUM_OUTPUTS: [&str; 7] = ["V", "W", "X", "Y", "Z", "AA", "AB"];

/// Seven score columns (each a permutation of 0..1,500, so no criterion repeats on its range),
/// a text key in K (one per row) and an amount in L. Beside each row, as calipers `pv26_002` does:
/// the rank of each score (`COUNTIFS(all,">"&x)+COUNTIFS(prefix,x)`, the prefix growing to the
/// row), the SUMIFS of the amounts of rows with that score, and the SUMIFS of the row's key.
fn rank_workbook() -> Result<Workbook, Box<dyn Error>> {
    common::filled_workbook(|tx| {
        let n = RANK_ROWS;
        for r in 1..=n {
            for (c, column) in (0u32..).zip(RANK_COLUMNS) {
                let score = f64::from((r * 7919 + c * 13) % n);
                tx.set(format!("{column}{r}").as_str(), score)?;
            }
            tx.set(format!("K{r}").as_str(), format!("key{r}").as_str())?;
            tx.set(format!("L{r}").as_str(), f64::from((r * 37) % 1000))?;
        }
        for r in 1..=n {
            for ((x, rank), sum) in RANK_COLUMNS.iter().zip(RANK_OUTPUTS).zip(SUM_OUTPUTS) {
                let rank_formula =
                    format!("=COUNTIFS({x}$1:{x}${n},\">\"&{x}{r})+COUNTIFS({x}$1:{x}{r},{x}{r})");
                let sum_formula = format!("=SUMIFS($L$1:$L${n},{x}$1:{x}${n},{x}{r})");
                tx.set(format!("{rank}{r}").as_str(), Input::Formula(rank_formula))?;
                tx.set(format!("{sum}{r}").as_str(), Input::Formula(sum_formula))?;
            }
            let key_formula = format!("=SUMIFS($L$1:$L${n},$K$1:$K${n},$K{r})");
            tx.set(format!("AD{r}").as_str(), Input::Formula(key_formula))?;
        }
        Ok(())
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    if std::env::args().any(|arg| arg == "tailed_consumers") {
        return tailed_consumer_cases();
    }
    let mut wb = sumproduct_workbook()?;
    let t = common::timings(RUNS, || wb.recalculate());
    writeln!(
        std::io::stdout(),
        "SUMPRODUCT((F:F=A1)*G:G): {WHOLE_COLUMN_CALLS} calls over {WHOLE_COLUMN_ROWS} rows, runs={RUNS} {t}"
    )?;
    if std::env::args().any(|arg| arg == "sumproduct") {
        return Ok(());
    }
    let mut wb = chained_workbook()?;
    let t = common::timings(RUNS, || wb.recalculate());
    writeln!(
        std::io::stdout(),
        "chained SUMIFS over formula columns: 1,000 calls over {CHAINED_ROWS} rows, runs={RUNS} {t}"
    )?;
    if std::env::args().any(|arg| arg == "chained") {
        return Ok(());
    }
    let mut wb = rank_workbook()?;
    let t = common::timings(RUNS, || wb.recalculate());
    writeln!(
        std::io::stdout(),
        "rank COUNTIFS + per-row SUMIFS: 21,000 COUNTIFS and 12,000 SUMIFS over {RANK_ROWS} rows, runs={RUNS} {t}"
    )?;
    if std::env::args().any(|arg| arg == "rank") {
        return Ok(());
    }
    let mut wb = whole_column_workbook()?;
    let t = common::timings(RUNS, || wb.recalculate());
    writeln!(
        std::io::stdout(),
        "MAX(IF(F:F=A1,G:G,\"\")): {WHOLE_COLUMN_CALLS} calls over {WHOLE_COLUMN_ROWS} rows, runs={RUNS} {t}"
    )?;
    if std::env::args().any(|arg| arg == "whole_column") {
        return Ok(());
    }
    let mut wb = criteria_workbook()?;
    let t = common::timings(RUNS, || wb.recalculate());
    writeln!(
        std::io::stdout(),
        "SUMIFS + COUNTIFS: {CRITERIA_CALLS} each over {CRITERIA_ROWS} rows, runs={RUNS} {t}"
    )?;
    if std::env::args().any(|arg| arg == "criteria") {
        return Ok(());
    }
    let far_cell_cases = [
        (
            "TEXTJOIN 3-D, whole columns beside a far cell",
            "=LEN(TEXTJOIN(\",\",FALSE,Sheet1:Two!A:J))",
        ),
        (
            "SUM(A:J) beside a far cell, 100 times in a LAMBDA",
            "=SUM(BYROW(SEQUENCE(100),LAMBDA(r,SUM(Two!A:J))))",
        ),
    ];
    for (name, formula) in far_cell_cases {
        let mut wb = far_cell_workbook(formula)?;
        let t = common::timings(RUNS, || wb.recalculate());
        writeln!(std::io::stdout(), "{name}: runs={RUNS} {t}")?;
    }
    let cases = [
        ("SUM", "=SUM(A1:A1000000)", false, false),
        ("AVERAGE", "=AVERAGE(A1:A1000000)", false, false),
        ("SUM, error in A2", "=SUM(A1:A1000000)", true, false),
        (
            "SUM, block past the used area",
            "=SUM(A1:C1048000)",
            false,
            false,
        ),
        (
            "SUM, whole sheet with a far cell",
            "=SUM(A:XFD)",
            false,
            true,
        ),
    ];
    for (name, formula, error_in_a2, far_cell) in cases {
        let mut wb = workbook(formula, error_in_a2, far_cell)?;
        let t = common::timings(RUNS, || wb.recalculate());
        writeln!(std::io::stdout(), "{name}: rows={ROWS} runs={RUNS} {t}")?;
    }
    tailed_consumer_cases()
}
