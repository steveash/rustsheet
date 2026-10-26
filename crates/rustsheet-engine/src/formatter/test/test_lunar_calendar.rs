//! Excel's lunar dates of calendar digits E, F, 11, 12 and 13 (T-1922).

use crate::formatter::locale_tag::LunarTable;
use crate::formatter::lunar_calendar::lunar_ymd;

use super::text;

#[test]
fn lunar_dates_of_the_probes() {
    use LunarTable::{Chinese, Eastern};
    // probes/text-locale-lunar-edges A1-A29, text-locale-calendars A98-A115,
    // text-locale-lunar-tokens A1-A9, A61-A85; serials 21915-21943 are 1959-12-31 to
    // 1960-01-28, where the year turns on 1 January.
    let cases = [
        (Chinese, 0, Some((1899, 12, 16))),
        (Chinese, 1, Some((1899, 12, 16))),
        (Chinese, 30, Some((1899, 13, 15))),
        (Chinese, 59, Some((1900, 1, 14))),
        (Chinese, 61, Some((1900, 1, 15))),
        (Chinese, 415, Some((1901, 1, 15))),
        (Chinese, 17940, Some((1948, 14, 1))),
        (Chinese, 17941, Some((1949, 1, 1))),
        (Chinese, 21915, Some((1959, 12, 18))),
        (Chinese, 21916, Some((1960, 12, 19))),
        (Chinese, 21943, Some((1960, 1, 1))),
        (Chinese, 44948, Some((2023, 1, 1))),
        (Chinese, 45000, Some((2023, 2, 24))),
        (Chinese, 45007, Some((2023, 3, 1))),
        (Chinese, 45065, Some((2023, 5, 1))),
        (Eastern, 45065, Some((2023, 4, 30))),
        (Eastern, 45066, Some((2023, 5, 1))),
        (Chinese, 45302, Some((2023, 13, 1))),
        (Chinese, 45332, Some((2024, 1, 1))),
        (Chinese, 73415, Some((2100, 12, 16))),
        (Chinese, 60, None),
        (Chinese, -1, None),
        (Chinese, 73416, None),
    ];
    for (table, serial, ymd) in cases {
        assert_eq!(lunar_ymd(serial, table), ymd, "{table:?} {serial}");
    }
}

#[test]
fn lunar_tags_print_excels_text() {
    // probes/text-locale-name-tables A589, text-locale-calendars A71-A85, A113,
    // text-locale-lunar-edges A26, A55, text-locale-lunar-tokens A10, A74.
    for (value, format, expected) in [
        (45000.0, "[$-11040C]mmmm", "二月"),
        (45000.0, "[$-11040C]d", "24"),
        (
            45000.0,
            "[$-E0409]yyyy-mm-dd|mmm|dddd|e|g",
            "2023-02-24|二月|水曜日|2023|",
        ),
        (45000.0, "[$-F0411]mmmm|dddd", "二月|星期三"),
        (
            45000.0,
            "[$-130409]yyyy-mm-dd|mmmm|dddd",
            "2023-02-24|February|Wednesday",
        ),
        (
            45300.0,
            "[$-110409]yyyy|m|mmmm|d|e",
            "2023|12|十二月|28|2023",
        ),
        (45302.0, "[$-110409]m|mmmm|mmmmm", "13|十二月|十"),
        (45066.0, "[$-E0412]m|mmmm|mmmmm", "5|오월|오"),
        (
            45000.5,
            "[$-110409]yyyy-mm-dd hh:mm:ss",
            "2023-02-24 12:00:00",
        ),
        (45000.75, "[$-E0411]h AM/PM", "6 午後"),
    ] {
        assert_eq!(text(value, format), expected, "{format}");
    }
}
