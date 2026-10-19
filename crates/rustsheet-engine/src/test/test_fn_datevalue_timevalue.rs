#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn datevalue_timevalue_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=DATEVALUE()");
    model._set("A2", "=TIMEVALUE()");
    model._set("A3", "=DATEVALUE(\"2000-01-01\")");
    model._set("A4", "=TIMEVALUE(\"12:00:00\")");
    model._set("A5", "=DATEVALUE(1,2)");
    model._set("A6", "=TIMEVALUE(1,2)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"36526");
    assert_eq!(model._get_text("A4"), *"0.5");
    assert_eq!(model._get_text("A5"), *"#ERROR!");
    assert_eq!(model._get_text("A6"), *"#ERROR!");
}

// Excel keeps the float error of serial - floor(serial) for "date time" text
// (TIMEVALUE.xlsx B18 cached 0.69097222221898846).
#[test]
fn timevalue_of_datetime_text_is_serial_fraction() {
    let mut model = new_empty_model();
    model._set("A1", "=TIMEVALUE(\"7/7/2026 4:35 PM\")");
    model._set("A2", "=TIMEVALUE(\"4:35 PM\")");
    model.evaluate();

    let value = |cell: &str| match model.get_cell_value_by_ref(cell).unwrap() {
        crate::cell::CellValue::Number(n) => n,
        other => panic!("expected number, got {other:?}"),
    };
    assert_eq!(value("Sheet1!A1"), 0.690_972_222_218_988_5);
    assert_eq!(value("Sheet1!A2"), 16.583_333_333_333_332 / 24.0);
}

// Clock text grammar, oracle values from probes/timevalue-*.case.json (excel-graph).
#[test]
fn timevalue_clock_text_grammar() {
    let cases: [(&str, Option<f64>); 13] = [
        ("10:11:12.5", Some(0.424_450_231_481_481_5)),
        ("10:11:12.4995", Some(0.424_450_231_481_481_5)), // half-up on the decimal digits
        ("11:12.5", Some(0.007_783_564_814_814_815)),     // two parts + fraction = M:S
        ("10:11:12.5 P", Some(0.924_450_231_481_481_5)),
        ("10:11:12 . 5", Some(0.424_450_231_481_481_5)),
        ("24:00:00.5", Some(5.787_037_037_130_46e-6)), // frac of 1.0000057870370371
        ("23:59:59.9995", Some(0.0)),
        ("1/2/2020 10:11:12.5", Some(0.424_450_231_483_206_15)),
        ("1/2/2020 25:00", Some(0.041_666_666_664_241_35)), // date + clock past 23 h
        ("10:61 PM", None),
        ("010:11:12", None),
        ("10000:00:00", None),
        ("10:11:12.5PM", None),
    ];
    let mut model = new_empty_model();
    for (i, (text, _)) in cases.iter().enumerate() {
        model._set(&format!("A{}", i + 1), &format!("=TIMEVALUE(\"{text}\")"));
    }
    model.evaluate();
    for (i, (text, expected)) in cases.iter().enumerate() {
        let actual = model
            .get_cell_value_by_ref(&format!("Sheet1!A{}", i + 1))
            .unwrap();
        match expected {
            Some(v) => assert_eq!(actual, crate::cell::CellValue::Number(*v), "{text}"),
            None => assert_eq!(model._get_text(&format!("A{}", i + 1)), "#VALUE!", "{text}"),
        }
    }
}

// `H:M.f:S` clock text reads the fraction digits as whole seconds and as the fraction, ignores
// `S`; at most one component may be out of range (hours from 24, minutes or whole seconds from
// 60); one trailing `:` is ignored. Oracle values (0+text) from probes/clock-text-oddities*.case.json (excel-graph).
#[test]
fn coercion_of_minutes_fraction_clock_text() {
    let cases: [(&str, Option<f64>); 22] = [
        ("10:11.5:12", Some(0.424_369_212_962_963)),
        ("10:11.5:99", Some(0.424_369_212_962_963)),
        ("10:11.0123:12", Some(0.425_729_305_555_555_6)),
        ("10:11.12345:12", Some(0.567_188_923_611_111_1)),
        ("0:11.9999:1", Some(0.123_379_629_629_629_63)),
        ("0:9999.5:1", Some(6.943_813_657_407_407)),
        ("10:11.59:12 PM", Some(0.924_995_254_629_629_6)),
        ("1/2/2020 10:11.5:12", Some(43_832.424_369_212_96)),
        ("10:11.5:", Some(0.007_077_546_296_296_296)),
        ("10:11:", Some(0.424_305_555_555_555_55)),
        ("10:11.5:12.5", None),
        ("10:11.:12", None),
        ("10:11.5:012", None),
        ("10:11.60:12 PM", None),
        ("0:9998.9999:1", None),
        ("0:9999:9999.9999", None),
        ("0:9999.9998:1", None),
        ("0:9999:59.9999", Some(6.944_444_444_444_445)),
        ("0:24:60", Some(0.017_361_111_111_111_112)),
        ("24:60", None),
        ("24:0:60", None),
        ("60:60.5", None),
    ];
    let mut model = new_empty_model();
    for (i, (text, _)) in cases.iter().enumerate() {
        model._set(&format!("A{}", i + 1), &format!("=0+\"{text}\""));
    }
    model.evaluate();
    for (i, (text, expected)) in cases.iter().enumerate() {
        let actual = model
            .get_cell_value_by_ref(&format!("Sheet1!A{}", i + 1))
            .unwrap();
        match expected {
            Some(v) => assert_eq!(actual, crate::cell::CellValue::Number(*v), "{text}"),
            None => assert_eq!(model._get_text(&format!("A{}", i + 1)), "#VALUE!", "{text}"),
        }
    }
}

// VALUE of clock text is the coercion serial (`0+text`) with surrounding spaces trimmed;
// coercion itself allows trailing but not leading spaces, and only the space character counts:
// a tab or a no-break space is never trimmed (probes/value-clock-text, excel-graph).
#[test]
fn value_and_coercion_of_clock_text() {
    // (text, VALUE(text), 0+text)
    let cases: [(&str, Option<f64>, Option<f64>); 19] = [
        ("10:00", Some(10.0 / 24.0), Some(10.0 / 24.0)),
        (
            "25:00",
            Some(1.041_666_666_666_666_7),
            Some(1.041_666_666_666_666_7),
        ),
        (
            "10:11.5",
            Some(0.007_077_546_296_296_296),
            Some(0.007_077_546_296_296_296),
        ),
        (" 10:00 ", Some(10.0 / 24.0), None),
        ("10:00 ", Some(10.0 / 24.0), Some(10.0 / 24.0)),
        ("10:00\u{a0}", None, None),
        ("10:00\u{a0}AM", None, None),
        ("10:11.\u{a0}5", None, None),
        ("10:11.\u{3000}5", None, None),
        ("10:11.5\u{a0}", None, None),
        ("10:11.\u{a0}5:12", None, None),
        ("10:00\t", None, None),
        ("1/2/2020\u{a0}", None, None),
        ("6 P.", Some(0.75), Some(0.75)),
        ("6 a", Some(0.25), Some(0.25)),
        ("0 PM", Some(0.5), Some(0.5)),
        ("0 AM", Some(0.0), Some(0.0)),
        ("13 PM", None, None),
        ("1/2/2020 6 PM", Some(43_832.75), Some(43_832.75)),
    ];
    let mut model = new_empty_model();
    for (i, (text, _, _)) in cases.iter().enumerate() {
        model._set(&format!("A{}", i + 1), &format!("=VALUE(\"{text}\")"));
        model._set(&format!("B{}", i + 1), &format!("=0+\"{text}\""));
    }
    model.evaluate();
    for (i, (text, value, coerced)) in cases.iter().enumerate() {
        for (column, expected) in [("A", value), ("B", coerced)] {
            let cell = format!("{column}{}", i + 1);
            match expected {
                Some(v) => assert_eq!(
                    model
                        .get_cell_value_by_ref(&format!("Sheet1!{cell}"))
                        .unwrap(),
                    crate::cell::CellValue::Number(*v),
                    "{column} {text:?}"
                ),
                None => assert_eq!(model._get_text(&cell), "#VALUE!", "{column} {text:?}"),
            }
        }
    }
}

// Excel ignores spaces around the date separators (ironcalc-calc DATEVALUE_issue_1380.xlsx;
// probes/datevalue-separator-spaces). It does not accept the ISO "T" separator, so the
// upstream T24:00:00 rollover (IronCalc #1379) is not ported: the oracle gives #VALUE!.
#[test]
fn datevalue_spaces_around_separators() {
    let mut model = new_empty_model();
    model._set("A1", "=DATEVALUE(\"2026 - 01 - 01\")");
    model._set("A2", "=DATEVALUE(\"1 / 2 / 2026\")");
    model._set("A3", "=DATEVALUE(\"4 - Feb - 2026\")");
    model._set("A4", "=DATEVALUE(\"2026 - 01 - 01 12:00\")");
    model._set("A5", "=VALUE(\"2026 - 01 - 01 12:00\")");
    model._set("A6", "=DATEVALUE(\"2026-01-01T24:00:00\")");
    model._set("A7", "=DATEVALUE(\"2026-01-01 24:00:00\")");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"46023");
    assert_eq!(model._get_text("A2"), *"46024");
    assert_eq!(model._get_text("A3"), *"46057");
    assert_eq!(model._get_text("A4"), *"46023");
    assert_eq!(model._get_text("A5"), *"46023.5");
    assert_eq!(model._get_text("A6"), *"#VALUE!");
    assert_eq!(model._get_text("A7"), *"46024");
}

// TIMEVALUE of date-only text is 0, not #VALUE! (probes/date-text-glued-month-2 column C rows 6-8,
// 10-11, 13-15); typed date+time text is a number formatted `m/d/yyyy h:mm` (N8, T-1907).
#[test]
fn timevalue_of_date_only_text_is_zero() {
    let mut model = new_empty_model();
    model._set("A1", "=TIMEVALUE(\"Jun31\")");
    model._set("A2", "=TIMEVALUE(\"Jul1 , 2026\")");
    model._set("A3", "=TIMEVALUE(\"not a date\")");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0");
    assert_eq!(model._get_text("A2"), *"0");
    assert_eq!(model._get_text("A3"), *"#VALUE!");
}

#[test]
fn typed_date_clock_text_is_a_number_with_a_date_time_format() {
    let mut model = new_empty_model();
    model._set("A1", "July1 10:00");
    model._set("A2", "=ISNUMBER(A1)");
    model._set("A3", "=MOD(A1,1)*24");
    model.evaluate();

    assert_eq!(model._get_text("A2"), *"TRUE");
    assert_eq!(model._get_text("A3"), *"10");
    assert_eq!(
        model.get_style_for_cell(0, 1, 1).unwrap().num_fmt,
        "m/d/yyyy h:mm"
    );
}

#[test]
fn typed_date_clock_text_with_seconds_or_am_pm_is_a_number() {
    // probes/date-text-non-glued-timevalue rows 16-17 (excel-graph, T-1952).
    let mut model = new_empty_model();
    model._set("A1", "1/2/2020 10:00:30");
    model._set("A2", "1/2/2020 10:00 AM");
    model._set("A3", "=TEXT(MOD(A1,1)*86400,\"0\")");
    model._set("A4", "=TEXT(MOD(A2,1)*24,\"0\")");
    model.evaluate();

    assert_eq!(model._get_text("A3"), *"36030");
    assert_eq!(model._get_text("A4"), *"10");
    assert_eq!(
        model.get_style_for_cell(0, 1, 1).unwrap().num_fmt,
        "m/d/yyyy h:mm"
    );
}

#[test]
fn typed_date_clock_text_shapes_pick_their_formats() {
    // probes/date-text-typed-clock-shapes (excel-graph, T-1979; formats from the ENTERED log in
    // docs/research/typed-clock-shapes-2026-10-19.md).
    let cases = [
        ("1/2/2020 10:00:30 PM", Some("m/d/yyyy h:mm")),
        ("1/2/2020 10:00 pm", Some("m/d/yyyy h:mm")),
        ("1/2/2020 10 AM", Some("m/d/yyyy h:mm")),
        ("1/2/2020 10:00  AM", Some("m/d/yyyy h:mm")),
        ("1/2/2020 10:00:30.5", Some("mm:ss.0")),
        ("1/2/2020 10:00.5", Some("mm:ss.0")),
        ("1/2/2020 24:00", Some("general")),
        ("1/2/2020 10:60", Some("general")),
        ("1/2/2020 13:00 PM", None),
        ("1/2/2020 10:00AM", None),
    ];
    let mut model = new_empty_model();
    for (i, (text, _)) in cases.iter().enumerate() {
        model._set(&format!("A{}", i + 1), text);
    }
    model.evaluate();
    for (i, (text, format)) in cases.iter().enumerate() {
        let row = i32::try_from(i + 1).unwrap();
        match format {
            Some(format) => assert_eq!(
                model.get_style_for_cell(0, row, 1).unwrap().num_fmt,
                *format,
                "{text}"
            ),
            None => assert_eq!(model._get_text(&format!("A{row}")), *text),
        }
    }
}

#[test]
fn typed_date_text_with_a_rejected_clock_stays_text_and_keeps_the_date_format() {
    // probes/date-text-typed-clock-shapes N14-N15, date-text-rejected-clock-formats N1-N16
    // (excel-graph, T-1986; formats from docs/research/rejected-clock-text-formats-2026-10-19.md).
    let cases = [
        ("1/2/2020 13:00 PM", "m/d/yyyy"),
        ("1/2/2020 13:00 AM", "m/d/yyyy"),
        ("1/2/2020 24:00 PM", "m/d/yyyy"),
        ("1/2/2020 13 PM", "m/d/yyyy"),
        ("1/2/2020 25:00 PM", "m/d/yyyy"),
        ("1/2/2020 13:00:00 PM", "m/d/yyyy"),
        ("1/2/2020 99:00 PM", "m/d/yyyy"),
        ("1/2/2020 1300 PM", "m/d/yyyy"),
        ("2020-01-02 13:00 PM", "m/d/yyyy"),
        ("July 1, 2026 13:00 PM", "d-mmm-yy"),
        ("1/2/2020 10:00AM", "general"),
        ("1/2/2020 10:00 XM", "general"),
        ("1/2/2020 abc", "general"),
        ("1/2/2020 10:00 PM PM", "general"),
        ("13:00 PM", "general"),
        // Unprobed: a non-date number before the rejected clock keeps General.
        ("$5 13:00 PM", "general"),
        ("10% 13:00 PM", "general"),
        ("1,000 13:00 PM", "general"),
    ];
    let mut model = new_empty_model();
    for (i, (text, _)) in cases.iter().enumerate() {
        model._set(&format!("A{}", i + 1), text);
    }
    model.evaluate();
    for (i, (text, format)) in cases.iter().enumerate() {
        let row = i32::try_from(i + 1).unwrap();
        assert_eq!(model._get_text(&format!("A{row}")), *text, "{text}");
        assert_eq!(
            model.get_style_for_cell(0, row, 1).unwrap().num_fmt,
            *format,
            "{text}"
        );
    }
}
