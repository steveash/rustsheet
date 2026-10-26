use crate::cell::CellValue;
use crate::test::util::new_empty_model;

#[test]
fn fn_complex() {
    let mut model = new_empty_model();
    model._set("A1", r#"=COMPLEX(3, 4.5, "i")"#);
    model._set("A2", r#"=COMPLEX(3, -5)"#);
    model._set("A3", r#"=COMPLEX(0, 42, "j")"#);

    model._set("B1", "=COMPLEX()");
    model._set("B2", r#"=COMPLEX(1,2, "i", 1)"#);

    model.evaluate();

    assert_eq!(model._get_text("A1"), "3+4.5i");
    assert_eq!(model._get_text("A2"), "3-5i");
    assert_eq!(model._get_text("A3"), "42j");

    assert_eq!(model._get_text("B1"), *"#ERROR!");
    assert_eq!(model._get_text("B2"), *"#ERROR!");
}

#[test]
fn complex_suffix_error_wins_over_number_errors() {
    let mut model = new_empty_model();
    model._set("A1", r#"=COMPLEX(1/0,1,"x")"#);
    model._set("A2", "=COMPLEX(NA(),1,1/0)");
    model._set("A3", r#"=COMPLEX(1/0,1,"j")"#);
    model._set("A4", "=COMPLEX(1/0,1,1)");
    model._set("A5", r#"=COMPLEX(1/0,1,"J")"#);
    model.evaluate();
    assert_eq!(model._get_text("A4"), *"#VALUE!");
    assert_eq!(model._get_text("A5"), *"#VALUE!");
    assert_eq!(model._get_text("A1"), *"#VALUE!");
    assert_eq!(model._get_text("A2"), *"#DIV/0!");
    assert_eq!(model._get_text("A3"), *"#DIV/0!");
}

#[test]
fn fn_imabs() {
    let mut model = new_empty_model();
    model._set("A1", r#"=IMABS("3+4i")"#);

    model.evaluate();

    assert_eq!(model._get_text("A1"), "5");
}

#[test]
fn fn_imaginary() {
    let mut model = new_empty_model();
    model._set("A1", r#"=IMAGINARY("3+4i")"#);

    model.evaluate();

    assert_eq!(model._get_text("A1"), "4");
}

#[test]
fn fn_imreal() {
    let mut model = new_empty_model();
    model._set("A1", r#"=IMREAL("3+4i")"#);

    model.evaluate();

    assert_eq!(model._get_text("A1"), "3");
}

#[test]
fn fn_imargument() {
    let mut model = new_empty_model();
    model._set("A1", r#"=IMARGUMENT("4+3i")"#);

    model.evaluate();

    assert_eq!(model._get_text("A1"), "0.643501109");
}

#[test]
fn fn_imconjugate() {
    let mut model = new_empty_model();
    model._set("A1", r#"=IMCONJUGATE("3+4i")"#);
    model._set("A2", r#"=IMCONJUGATE("12.7-32j")"#);

    model.evaluate();

    assert_eq!(model._get_text("A1"), "3-4i");
    assert_eq!(model._get_text("A2"), "12.7+32j");
}

#[test]
fn fn_imcos() {
    let mut model = new_empty_model();
    model._set("A1", r#"=IMCOS("4+3i")"#);
    // In macos non intel this is "-6.58066304055116+7.58155274274655i"
    model._set("A2", r#"=COMPLEX(-6.58066304055116, 7.58155274274654)"#);
    model._set("A3", r#"=IMABS(IMSUB(A1, A2)) < G1"#);

    // small number
    model._set("G1", "0.0000001");

    model.evaluate();

    assert_eq!(model._get_text("A3"), "TRUE");
}

#[test]
fn fn_imsin() {
    let mut model = new_empty_model();
    model._set("A1", r#"=IMSIN("4+3i")"#);

    model.evaluate();

    assert_eq!(model._get_text("A1"), "-7.61923172032141-6.548120040911i");
}

#[test]
fn fn_imaginary_misc() {
    let mut model = new_empty_model();
    model._set("A1", r#"=IMAGINARY("3.4i")"#);
    model._set("A2", r#"=IMAGINARY("-3.4")"#);

    model.evaluate();

    assert_eq!(model._get_text("A1"), "3.4");
    assert_eq!(model._get_text("A2"), "0");
}

#[test]
fn fn_imcosh() {
    let mut model = new_empty_model();
    model._set("A1", r#"=IMCOSH("4+3i")"#);

    model.evaluate();

    assert_eq!(model._get_text("A1"), "-27.0349456030742+3.85115333481178i");
}

#[test]
fn fn_imcot() {
    let mut model = new_empty_model();
    model._set("A1", r#"=IMCOT("4+3i")"#);

    model.evaluate();

    assert_eq!(
        model._get_text("A1"),
        // Excel: ironcalc-calc/ENGINEERING/COMPLEXs.xlsx T25, U25 (T-0907).
        "0.00490118239430447-0.999266927805902i"
    );
}

// Past `cosh 2y`'s range IMTAN is `±0 ± i` and IMCOT `±0 ∓ i`, as tanh gives; at the double
// nearest π/2 IMTAN stays finite (T-0907). Not Excel evidence: these pin our fallbacks until
// T-1966 probes Excel there.
#[test]
fn fn_imtan_imcot_large_imaginary_part() {
    let mut model = new_empty_model();
    model._set("A1", r#"=IMTAN("1+400i")"#);
    model._set("A2", r#"=IMTAN("1-400i")"#);
    model._set("A3", r#"=IMCOT("1+400i")"#);
    model._set("A4", r#"=IMTAN("1.5707963267948966")"#);

    model.evaluate();

    assert_eq!(model._get_text("A1"), "i");
    assert_eq!(model._get_text("A2"), "-i");
    assert_eq!(model._get_text("A3"), "-i");
    // Near π/2 the double-angle denominator is (close to) 0: a finite number, never NaN.
    assert!(model
        ._get_text("A4")
        .parse::<f64>()
        .is_ok_and(f64::is_finite));
}

#[cfg_attr(target_os = "windows", ignore)]
#[test]
fn fn_imtan() {
    let mut model = new_empty_model();
    model._set("A1", r#"=IMTAN("4+3i")"#);

    model.evaluate();

    assert_eq!(
        model._get_text("A1"),
        // Excel: ironcalc-calc/ENGINEERING/COMPLEXs.xlsx N25, O25 (T-0907).
        "0.00490825806749606+1.00070953606723i"
    );
}

#[test]
fn fn_power() {
    let mut model = new_empty_model();
    model._set("A2", r#"=IMPOWER("4+3i", 3)"#);
    model._set("A3", r#"=IMABS(IMSUB(IMPOWER("-i", -3), "-1"))<G1"#);
    model._set("A3", r#"=IMABS(IMSUB(IMPOWER("-1", 0.5), "i"))<G1"#);

    model._set("A1", r#"=IMABS(IMSUB(B1, "-1"))<G1"#);
    model._set("B1", r#"=IMPOWER("i", 2)"#);

    // small number
    model._set("G1", "0.0000001");

    model.evaluate();

    assert_eq!(model._get_text("A1"), "TRUE");
    assert_eq!(model._get_text("A2"), "-44+117i");
    assert_eq!(model._get_text("A3"), "TRUE");
    assert_eq!(model._get_text("A3"), "TRUE");
}

#[test]
fn im_sub_div_power_edge_cases() {
    let mut model = new_empty_model();
    model._set("A1", r#"=IMDIV("1","1E-200")"#);
    model._set("A2", r#"=IMDIV("-3-4i","1E-200+0i")"#);
    model._set("A3", r#"=IMSUB("","1+i")"#);
    model._set("A4", r#"=IMSUB(B9,"1+i")"#);
    model._set("A5", r#"=IMDIV("3+4i","1+2j")"#);
    model._set("A6", r#"=IMPOWER("0",0)"#);
    model._set("A7", r#"=IMDIV("1E+308","1E-308")"#);
    model._set("A8", r#"=IMSUB("1E+308","-1E+308")"#);
    model._set("A9", r#"=IMPOWER("",2)"#);
    model._set("A10", r#"=IMPOWER("i",2)"#);
    model._set("A11", r#"=IMPOWER("1+i",100)"#);
    model._set("A12", r#"=IMPOWER("-j",3)"#);
    model.evaluate();

    assert_eq!(model._get_text("A1"), "1E+200");
    assert_eq!(model._get_text("A2"), "-3E+200-4E+200i");
    assert_eq!(model._get_text("A3"), "#NUM!");
    assert_eq!(model._get_text("A4"), "-1-i");
    assert_eq!(model._get_text("A5"), "#NUM!");
    assert_eq!(model._get_text("A6"), "#NUM!");
    assert_eq!(model._get_text("A7"), "#NUM!");
    assert_eq!(model._get_text("A8"), "#NUM!");
    assert_eq!(model._get_text("A9"), "#NUM!");
    assert_eq!(model._get_text("A10"), "-1+1.22514845490862E-16i");
    assert_eq!(
        model._get_text("A11"),
        "-1125899906842640-0.551513671875006i"
    );
    assert_eq!(model._get_text("A12"), "-1.83772268236293E-16+j");
}

/// Excel: empty text is `#NUM!` in every one-argument IM* function, a blank cell is 0
/// (probes/eng-gen-sample-edges D1:D32).
#[test]
fn empty_text_is_num_error_blank_is_zero() {
    let mut model = new_empty_model();
    model._set("B1", r#"="""#);
    model._set("A1", r#"=IMSQRT("")"#);
    model._set("A2", "=IMSQRT(B1)");
    model._set("A3", r#"=IMREAL("")"#);
    model._set("A4", r#"=IMLN("")"#);
    model._set("A5", "=IMSQRT(C1)");
    model._set("A6", "=IMCOS(C1)");
    model.evaluate();
    for cell in ["A1", "A2", "A3", "A4"] {
        assert_eq!(model._get_text(cell), *"#NUM!", "{cell}");
    }
    assert_eq!(model._get_text("A5"), *"0");
    assert_eq!(model._get_text("A6"), *"1");
}

#[test]
fn im_overflow_edges() {
    // probes/eng-im-overflow-edges (excel-graph): A2, A3, A8, A27, A55, A64.
    let mut model = new_empty_model();
    model._set("B1", r#"=IMPRODUCT(A1,"1E200","1E200")"#);
    model._set("B2", r#"=IMPRODUCT("1E200","1E200",A1)"#);
    model._set("B3", r#"=IMCOSH("710.4+0i")"#);
    model._set("B4", r#"=IMSIN("710.4i")"#);
    model._set("B5", r#"=IMSECH("711+1.5i")"#);
    model._set("B6", r#"=IMCSCH("710.4+0i")"#);
    model.evaluate();

    assert_eq!(model._get_text("B1"), *"0");
    assert_eq!(model._get_text("B2"), *"#VALUE!");
    assert_eq!(model._get_text("B3"), *"#NUM!");
    assert_eq!(model._get_text("B4"), *"#NUM!");
    assert_eq!(model._get_text("B5"), *"0");
    assert_eq!(model._get_text("B6"), *"0");
}

// Excel: probes-numeric/im-last-digit-sweep E55:I55, K42:L42 and gen-m9/b5aef3645cec1b95 A154,
// probes/eng-im-overflow-edges A82 (T-2335): where the reciprocal's denominator overflows, a normal
// tiny part is kept and a subnormal one prints as 0.
#[test]
fn im_reciprocal_overflow_keeps_normal_tiny_parts() {
    let cases = [
        (
            r#"=IMCSC("-724.227-562.884i")"#,
            "-6.94796703439417E-245-6.26263832046024E-246i",
        ),
        (
            r#"=IMSEC("-724.227-562.884i")"#,
            "-6.26263832046024E-246+6.94796703439417E-245i",
        ),
        (
            r#"=IMSECH("-524.561+5.3113i")"#,
            "1.73046857535442E-228-2.53533899849465E-228i",
        ),
        (r#"=IMSEC("1.5+711i")"#, "0"),
        (r#"=IMEXP("-724.227-562.884i")"#, "0"),
    ];
    let mut model = new_empty_model();
    for (row, (formula, _)) in cases.iter().enumerate() {
        model._set(&format!("A{}", row + 1), formula);
    }
    model.evaluate();
    for (row, (formula, expected)) in cases.iter().enumerate() {
        assert_eq!(
            model._get_text(&format!("A{}", row + 1)),
            *expected,
            "{formula}"
        );
    }
}

// Excel: probes-numeric/im-last-digit-sweep D21, M21, P21 (T-2334).
#[test]
fn fn_imcsc_imlog2_imcot_of_zero_are_num() {
    let mut model = new_empty_model();
    model._set("A1", r#"=IMCSC("0")"#);
    model._set("A2", r#"=IMLOG2("0")"#);
    model._set("A3", r#"=IMCOT("0")"#);

    model.evaluate();

    assert_eq!(model._get_text("A1"), "#NUM!");
    assert_eq!(model._get_text("A2"), "#NUM!");
    assert_eq!(model._get_text("A3"), "#NUM!");
}

// Excel: gen-m9 (excel-graph, T-2258): IMLN and IMLOG10 of 0 or 0i are #NUM!, not "-inf" (T-2389).
#[test]
fn fn_imln_imlog10_of_zero_are_num() {
    let mut model = new_empty_model();
    for (i, f) in [
        "IMLN(\"0\")",
        "IMLN(\"0i\")",
        "IMLOG10(\"0\")",
        "IMLOG10(\"0i\")",
    ]
    .iter()
    .enumerate()
    {
        model._set(&format!("A{}", i + 1), &format!("={f}"));
    }
    model.evaluate();
    for i in 1..=4 {
        assert_eq!(model._get_text(&format!("A{i}")), "#NUM!");
    }
}

// Excel: probes-numeric/im-last-digit-sweep (excel-graph; T-2044). The tiny-argument rows keep
// the digits Excel loses (D65, D70, P65, P70), IMSEC and IMSECH differ in the last digit of the
// same real part (G23, J58), and a large real part needs SIN's own reduction (D51).
#[test]
fn im_reciprocal_kernels_match_the_sweep() {
    let cases = [
        (
            r#"=IMCSC("1E-5+1E-5i")"#,
            "49999.9979339908-49999.9979306575i",
        ),
        (r#"=IMCSC("0.00001i")"#, "-99999.9917276303i"),
        (
            r#"=IMCSC("-9447.72-0.00486262i")"#,
            "1.22857263522781-0.00426406656289288i",
        ),
        (
            r#"=IMCOT("1E-5+1E-5i")"#,
            "49999.9958596485-49999.9958663151i",
        ),
        (r#"=IMCOT("0.00001i")"#, "-99999.9917326303i"),
        (r#"=IMCOT("219.098+3372.26i")"#, "-i"),
        (r#"=IMCOT("-724.227-562.884i")"#, "i"),
        (
            r#"=IMSEC("4+3i")"#,
            "-0.0652940278579471-0.0752249603027732i",
        ),
        (
            r#"=IMSECH("3+4i")"#,
            "-0.065294027857947+0.0752249603027732i",
        ),
        // Near a pole of the secant Excel's sinh²y = cosh²y - 1 loses digits (G29, G33; T-2743).
        (
            r#"=IMSEC("23.7944+0.00977722i")"#,
            "4.33329584706818-0.178960935839407i",
        ),
        (
            r#"=IMSEC("956.615+0.00387493i")"#,
            "-2.46276584105671+258.045027895372i",
        ),
        (r#"=IMSEC("1+0.5i")"#, "1.08126832311555+0.778194035118073i"),
        // IMSECH's t fuses sinh²x into the sum (J34; T-2750): unfused it gives ...306335i.
        (
            r#"=IMSECH("1.34384-77.6742i")"#,
            "-0.367433538630135+0.376675382306336i",
        ),
        // A real argument is 1/sin x and 1/cosh x (gen-m9/2f0d384b8ed0d392 A109, 01810fc89c5ec2c0 A318).
        (r#"=IMCSC("0.2")"#, "5.03348954767234"),
        (r#"=IMSECH("1.5")"#, "0.42509603494228"),
    ];
    let mut model = new_empty_model();
    for (row, (formula, _)) in cases.iter().enumerate() {
        model._set(&format!("A{}", row + 1), formula);
    }
    model.evaluate();
    for (row, (formula, expected)) in cases.iter().enumerate() {
        assert_eq!(
            model._get_text(&format!("A{}", row + 1)),
            *expected,
            "{formula}"
        );
    }
}

// Excel: probes-numeric/im-last-digit-sweep B22, C22 (T-2044): IMABS scales by the larger part
// (`hypot` gives 51.39464785909132) and IMARGUMENT is ATAN2's `atan(y/x) - π`
// (libm `atan2` gives -2.0374703428717544).
#[test]
fn imabs_and_imargument_kernels() {
    let mut model = new_empty_model();
    model._set("A1", r#"=IMABS("-23.1234-45.899i")"#);
    model._set("A2", r#"=IMARGUMENT("-23.1234-45.899i")"#);
    model.evaluate();
    let bits = |c: &str| match model.get_cell_value_by_ref(&format!("Sheet1!{c}")) {
        Ok(CellValue::Number(n)) => n.to_bits(),
        other => panic!("expected number, got {other:?}"),
    };
    assert_eq!(bits("A1"), 51.394_647_859_091_315_f64.to_bits());
    assert_eq!(bits("A2"), (-2.037_470_342_871_754_f64).to_bits());
}

// IMPOWER's angle is Excel's ATAN2 kernel `atan(y/x)`, one ulp above libm `atan2(1,1000)` here, so
// the imaginary part is 1.0000000000000002 and keeps its "1" (gen-m9/74779cff2810ccf4 Calc!A379;
// docs/excel-semantics/engineering.md "1i" when a part is not exactly 1; T-2610).
#[test]
fn impower_angle_is_excels_atan2_kernel() {
    let mut model = new_empty_model();
    model._set("A1", r#"=IMPOWER("1000+1i",1)"#);
    model.evaluate();
    assert_eq!(model._get_text("A1"), "1000+1i");
}

// IMSIN, IMCOS, IMSEC, IMCSC and IMCOT of a real number take SIN, COS and TAN's kernels and are
// `#NUM!` from 2^27, IMCOT "0"; IMTAN takes the limit only (probes/t2534-im-trig-real-limit, T-2534).
#[test]
fn im_trig_of_a_real_number_follows_sin_cos_tan() {
    let cases = [
        (r#"=IMSIN("710")"#, "0.0000602887066915739"),
        (r#"=IMSIN("123456789.123")"#, "0.99984293955201"),
        (r#"=IMCOS("123456789.123")"#, "0.0177227601686489"),
        (r#"=IMSEC("123456789.123")"#, "56.424619556099"),
        (r#"=IMCSC("710")"#, "16586.8544023646"),
        (r#"=IMCOT("710")"#, "16586.8543722202"),
        (r#"=IMCOT("123456789.123")"#, "0.0177255441505541"),
        (r#"=IMSIN("134217727")"#, "-0.955993328772671"),
        (r#"=IMSIN("134217728")"#, "#NUM!"),
        (r#"=IMCOS("-1E+200")"#, "#NUM!"),
        (r#"=IMTAN("1E+15")"#, "#NUM!"),
        (r#"=IMSEC("1E+10")"#, "#NUM!"),
        (r#"=IMCSC("1E+200")"#, "#NUM!"),
        (r#"=IMCOT("1E+200")"#, "0"),
        (r#"=IMCOT("-134217728")"#, "0"),
        // The desktop cache of COMPLEXs.xlsx N6; the web gives TAN's -3.38051500624659 (T-2774).
        (r#"=IMTAN("5")"#, "-3.38051500624658"),
    ];
    let mut model = new_empty_model();
    for (row, (formula, _)) in cases.iter().enumerate() {
        model._set(&format!("A{}", row + 1), formula);
    }
    model.evaluate();
    for (row, (formula, expected)) in cases.iter().enumerate() {
        assert_eq!(
            model._get_text(&format!("A{}", row + 1)),
            *expected,
            "{formula}"
        );
    }
}
