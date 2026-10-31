use super::*;

fn point(id: &str, standing: Standing) -> Point {
    Point {
        id: id.into(),
        table: "F".into(),
        batch: "s1".into(),
        formula: Some("F(1)".into()),
        stratum: Some(Stratum::Typical),
        engine: Some(Value::Number(1.0)),
        excel: None,
        math: Some(MathValue::Number(1.0)),
        class: if standing == Standing::Inside {
            Envelope::Math
        } else {
            Envelope::Outside
        },
        standing,
        r3: false,
        owner: None,
        note: None,
    }
}

fn known(lines: &[(&str, usize)]) -> KnownFailures {
    lines
        .iter()
        .map(|(id, line)| {
            (
                (*id).to_string(),
                lists::KnownFailure {
                    line: *line,
                    owner: "T-0001".into(),
                    note: "n".into(),
                },
            )
        })
        .collect()
}

#[test]
fn a_batch_is_a_function_directory_and_a_file() {
    assert_eq!(
        table_and_batch("accuracy/T.INV.2T/r4-T-1234.case.json"),
        Some(("T.INV.2T".into(), "r4-T-1234".into()))
    );
    assert_eq!(table_and_batch("accuracy/s1.case.json"), None);
    assert_eq!(table_and_batch("accuracy/F/sub/s1.case.json"), None);
    assert_eq!(table_and_batch("probes/F/s1.case.json"), None);
}

#[test]
fn known_failures_carry_failing_points_and_stale_lines_fail() {
    let mut points = vec![
        point("accuracy/F/s1!Calc!A1", Standing::Failing),
        point("accuracy/F/s1!Calc!A2", Standing::Inside),
        point("accuracy/F/s1!Calc!A3", Standing::Failing),
    ];
    let lines = known(&[
        ("accuracy/F/s1!Calc!A1", 2),
        ("accuracy/F/s1!Calc!A2", 3),
        ("accuracy/F/s9!Calc!A1", 4),
    ]);
    let stale = apply_known_failures(&mut points, &lines);
    assert_eq!(points[0].standing, Standing::KnownFailure);
    assert_eq!(points[0].owner.as_deref(), Some("T-0001"));
    assert_eq!(points[2].standing, Standing::Failing);
    let stale: Vec<(usize, &str)> = stale.iter().map(|s| (s.line, s.id.as_str())).collect();
    assert_eq!(
        stale,
        vec![(3, "accuracy/F/s1!Calc!A2"), (4, "accuracy/F/s9!Calc!A1")]
    );
}

#[test]
fn the_check_passes_only_without_failures_stale_lines_marks_or_defects() {
    let check = |points: Vec<Point>| Check {
        points,
        stale_known_failures: Vec::new(),
        bad_marks: Vec::new(),
        seconds: 0.0,
    };
    assert!(check(vec![point("a!S!A1", Standing::Inside)]).passes());
    assert!(check(vec![point("a!S!A1", Standing::KnownFailure)]).passes());
    assert!(!check(vec![point("a!S!A1", Standing::Failing)]).passes());
    assert!(!check(vec![point("a!S!A1", Standing::TableDefect)]).passes());
    let mut stale = check(Vec::new());
    stale.stale_known_failures.push(ListProblem {
        file: lists::KNOWN_FAILURES,
        line: 1,
        id: "a!S!A1".into(),
        why: "x".into(),
    });
    assert!(!stale.passes());
    assert!(check(Vec::new())
        .render()
        .starts_with("accuracy: 0 points in 0 tables (pending 0): 0 in envelope"));
}
