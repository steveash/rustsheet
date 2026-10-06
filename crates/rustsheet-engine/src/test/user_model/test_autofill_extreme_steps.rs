#![allow(clippy::unwrap_used)]
use crate::expressions::types::Area;
use crate::locale::get_locale;
use crate::test::util::new_empty_model;
use crate::UserModel;

/// Fills A1:A2 down to A4 and returns the numbers autofill wrote to A3 and A4.
fn fill_two(locale: &str, first: &str, second: &str) -> (f64, f64) {
    let mut model = new_empty_model();
    model.locale = get_locale(locale).unwrap();
    let mut model = UserModel::from_model(model);
    // `de` types the decimal separator as a comma
    let typed = |text: &str| {
        if locale == "de" {
            text.replace('.', ",")
        } else {
            text.to_string()
        }
    };
    model.set_user_input(0, 1, 1, &typed(first)).unwrap();
    model.set_user_input(0, 2, 1, &typed(second)).unwrap();
    model
        .auto_fill_rows(
            &Area {
                sheet: 0,
                row: 1,
                column: 1,
                width: 1,
                height: 2,
            },
            4,
        )
        .unwrap();
    let number = |row| {
        model
            .get_cell_content(0, row, 1)
            .unwrap()
            .replace(',', ".")
            .parse::<f64>()
            .unwrap()
    };
    (number(3), number(4))
}

#[test]
fn small_and_large_steps_keep_their_values() {
    for locale in ["en", "de"] {
        let (a, b) = fill_two(locale, "0.0001", "0.0002");
        assert_eq!((a, b), (0.0003, 0.0004), "{locale}");
        let (a, b) = fill_two(locale, "0.00001", "0.00002");
        assert_eq!((a, b), (0.00003, 0.00004), "{locale}");
        let (a, b) = fill_two(locale, "1000000000000", "2000000000000");
        assert_eq!((a, b), (3e12, 4e12), "{locale}");
        let (a, b) = fill_two(locale, "100000000000000", "200000000000000");
        assert_eq!((a, b), (3e14, 4e14), "{locale}");
    }
}

#[test]
fn exponent_notation_sources_are_copied_not_extended() {
    // Characterization only: no Excel evidence yet for fills over E-notation numbers (T-0171).
    assert_eq!(fill_two("en", "1E-10", "2E-10"), (1e-10, 2e-10));
    assert_eq!(fill_two("en", "1E+20", "2E+20"), (1e20, 2e20));
}
