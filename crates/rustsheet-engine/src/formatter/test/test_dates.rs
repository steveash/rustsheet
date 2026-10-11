#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::{
    formatter::format::{format_number, FormatContext},
    locale::{get_locale, Locale},
    types::DateSystem,
};

fn get_default_locale() -> &'static Locale {
    get_locale("de").unwrap()
}

#[test]
fn simple_test() {
    let locale = get_default_locale();
    let b = format_number(46015.0, "m.d.yy", &FormatContext::new(locale));
    assert_eq!(b.text, "12.24.25");
}

/// `format_number` of `value` with `format` in the en locale and the given date system.
fn format_in(value: f64, format: &str, date_system: DateSystem) -> String {
    let ctx = FormatContext {
        locale: get_locale("en").unwrap(),
        date_system,
    };
    format_number(value, format, &ctx).text
}

// probes/negative-date-sections-1900 and probes/date-system-1900-negatives (T-1105, T-1268).
#[test]
fn negative_serials_1900() {
    let cases = [
        (-1.5, "[h]", "-36"),
        (-1.5, "[h]\" h\"", "-36 h"),
        (-1.5, "\"x\"[h]", "-x36"),
        (-1.5, "[Red][h]", "-36"),
        (-1.5, "[h];[h]", "36"),
        (-1.5, "0;[h]", "36"),
        (-1E-10, "[s]", "-0"),
        (-0.0, "[h]:mm", "0:00"),
        (-1.5, "[h]:mm", "#VALUE!"),
        (-1.5, "[h] d", "#VALUE!"),
        (-1.5, "[s].00", "#VALUE!"),
        (-1.5, "[h]:mm;-[h]:mm", "#VALUE!"),
        (-1.0, "d;d", "#VALUE!"),
        (-1.5, "[h];d", "#VALUE!"),
        (-1.5, "0.0;d", "#VALUE!"),
        (-0.5, "h:mm", "#VALUE!"),
    ];
    for (value, format, expected) in cases {
        assert_eq!(
            format_in(value, format, DateSystem::Excel1900),
            expected,
            "{value} {format}"
        );
    }
}

// probes/date-system-1904-negatives, negative-date-sections-1904 and poi ElapsedFormatTests A40.
#[test]
fn negative_serials_1904() {
    let cases = [
        (-1.5, "[h]:mm", "-36:00"),
        (-1.5, "[mm]:ss", "-2160:00"),
        (-1E-10, "[h]:mm", "-0:00"),
        (-0.5, "h:mm", "-12:00"),
        (-1.5, "[h]:mm;-[h]:mm", "-36:00"),
        (-1.5, "\"x\"[h]", "-x36"),
        (-1.5, "[h];[h]", "36"),
    ];
    for (value, format, expected) in cases {
        assert_eq!(
            format_in(value, format, DateSystem::Excel1904),
            expected,
            "{value} {format}"
        );
    }
}
