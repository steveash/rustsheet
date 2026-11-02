use crate::functions::engineering::complex::Suffix;

use super::reading::{split_inumber, Inumber};

fn complex<'a>(
    real: Option<&'a str>,
    sign: f64,
    coefficient: &'a str,
    suffix: Suffix,
) -> Option<Inumber<'a>> {
    Some(Inumber::Complex {
        real,
        sign,
        coefficient,
        suffix,
    })
}

#[test]
fn inner_sign_without_suffix_is_not_a_number() {
    // Excel: probes/eng-inumber-reading A4 ("2024-01-01"), probes/eng-inumber-date-boundary A4,
    // A8, A13, A20, A23 and probes/eng-inumber-grammar A47:A50 refused; A3, A5, A6, A10, A21
    // read. "Dec-2020" and ".5e-1" are the rule's own reading, unprobed.
    for text in [
        "2024-01-01",
        "1-2-2020",
        "$-5",
        "2020-1-2",
        "$1,000-",
        "1-1-1",
        "Dec-2020",
        "1+2",
        "1i+2",
        "1+2i+3",
    ] {
        assert_eq!(split_inumber(text), None, "{text}");
    }
    for text in [
        " -5", "$5", "1E-3%", "1,000E-2", "- 5", "-5", ".5e-1", "1E+5",
    ] {
        assert_eq!(split_inumber(text), Some(Inumber::Real(text)), "{text}");
    }
}

#[test]
fn split_at_the_first_inner_sign() {
    // Excel: probes/eng-inumber-grammar A3:A8, A15, probes/eng-inumber-grammar-parts A1, A6,
    // A9, A13, A43, A45.
    assert_eq!(
        split_inumber("1+-2i"),
        complex(Some("1"), 1.0, "-2", Suffix::I)
    );
    assert_eq!(
        split_inumber("1--2i"),
        complex(Some("1"), -1.0, "-2", Suffix::I)
    );
    assert_eq!(
        split_inumber("-1+-2i"),
        complex(Some("-1"), 1.0, "-2", Suffix::I)
    );
    assert_eq!(
        split_inumber("1+-i"),
        complex(Some("1"), 1.0, "-", Suffix::I)
    );
    assert_eq!(
        split_inumber("1E+5+1E-5j"),
        complex(Some("1E+5"), 1.0, "1E-5", Suffix::J)
    );
    assert_eq!(
        split_inumber(" 1 + 2 i "),
        complex(Some(" 1 "), 1.0, " 2 ", Suffix::I)
    );
    assert_eq!(split_inumber("-i"), complex(None, -1.0, "", Suffix::I));
    assert_eq!(split_inumber("+i"), complex(None, 1.0, "", Suffix::I));
    assert_eq!(split_inumber("i"), complex(None, 1.0, "", Suffix::I));
    assert_eq!(split_inumber("-5i"), complex(None, 1.0, "-5", Suffix::I));
    assert_eq!(
        split_inumber("+-2i"),
        complex(Some("+"), -1.0, "2", Suffix::I)
    );
    assert_eq!(split_inumber("1+2I"), None);
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
