use crate::functions::engineering::complex::Suffix;

use super::has_inner_minus;
use super::parse_complex_number as parse;

#[test]
fn inner_minus_spares_a_leading_sign_and_an_exponent() {
    // Excel: probes/eng-inumber-reading A4 ("2024-01-01") and probes/eng-inumber-date-boundary
    // A4, A8, A13, A20, A23 refused; A3, A5, A6, A10, A21 read. "Dec-2020", "June-2020", "$5"
    // and ".5e-1" are the rule's own reading, unprobed.
    for text in [
        "2024-01-01",
        "1-2-2020",
        "$-5",
        "2020-1-2",
        "$1,000-",
        "1-1-1",
        "Dec-2020",
        "June-2020",
    ] {
        assert!(has_inner_minus(text), "{text}");
    }
    for text in [" -5", "$5", "1E-3%", "1,000E-2", "- 5", "-5", ".5e-1"] {
        assert!(!has_inner_minus(text), "{text}");
    }
}

#[test]
fn coefficient_one_written_unless_exactly_one() {
    let text = |x: f64, y: f64| {
        super::Complex {
            x,
            y,
            suffix: Suffix::I,
        }
        .to_string()
    };
    assert_eq!(text(0.1, 1.0 + 2f64.powi(-52)), "0.1+1i");
    assert_eq!(text(0.5, 1.0 - 2f64.powi(-53)), "0.5+1i");
    assert_eq!(text(0.5, -1.0 - 2f64.powi(-52)), "0.5-1i");
    assert_eq!(text(0.5, 1.0), "0.5+i");
    assert_eq!(text(0.5, -1.0), "0.5-i");
    assert_eq!(text(0.0, 1.0), "i");
}

#[test]
fn suffix_join() {
    assert_eq!(Suffix::I.join(Suffix::J), None);
    assert_eq!(Suffix::Real.join(Suffix::J), Some(Suffix::J));
    assert_eq!(Suffix::I.join(Suffix::Real), Some(Suffix::I));
    assert_eq!(Suffix::Real.join(Suffix::Real), Some(Suffix::Real));
}

#[test]
fn test_parse_complex() {
    assert_eq!(parse("1+2i"), Ok((1.0, 2.0, Suffix::I)));
    assert_eq!(parse("2i"), Ok((0.0, 2.0, Suffix::I)));
    assert_eq!(parse("7.5"), Ok((7.5, 0.0, Suffix::Real)));
    assert_eq!(parse("-7.5"), Ok((-7.5, 0.0, Suffix::Real)));
    assert_eq!(parse("7-5i"), Ok((7.0, -5.0, Suffix::I)));
    assert_eq!(parse("i"), Ok((0.0, 1.0, Suffix::I)));
    assert_eq!(parse("7+i"), Ok((7.0, 1.0, Suffix::I)));
    assert_eq!(parse("7-i"), Ok((7.0, -1.0, Suffix::I)));
    assert_eq!(parse("-i"), Ok((0.0, -1.0, Suffix::I)));
    assert_eq!(parse("0"), Ok((0.0, 0.0, Suffix::Real)));
}
