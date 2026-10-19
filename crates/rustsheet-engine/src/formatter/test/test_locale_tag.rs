#![allow(clippy::unwrap_used)]

use crate::{
    formatter::{
        format::{format_number, format_text_value, FormatContext},
        locale_tag::{resolve, LocaleTag},
    },
    locale::get_locale,
};

fn text(value: f64, format: &str) -> String {
    let ctx = FormatContext::new(get_locale("en").unwrap());
    format_number(value, format, &ctx).text
}

// Values from probes/text-locale-date-names, text-locale-name-tables and text-locale-am-neutral.

#[test]
fn names_follow_the_tag() {
    assert_eq!(
        text(45000.0, "[$-40C]dddd d mmmm yyyy"),
        "mercredi 15 mars 2023"
    );
    assert_eq!(text(45000.0, "[$-407]ddd mmm mmmmm"), "Mi Mrz M");
    assert_eq!(text(45000.0, "[$-807]ddd mmm"), "Mi. Mär");
    assert_eq!(text(45261.0, "[$-804]mmmmm"), "十");
    assert_eq!(text(0.0, "[$-40C]dddd mmmm"), "samedi janvier");
    assert_eq!(text(45000.0, "mmmm[$-40C]"), "mars");
    // A tag applies to its own section only.
    assert_eq!(text(45000.0, "[$-407]mmmm;0"), "März");
}

#[test]
fn day_halves_follow_the_tag() {
    assert_eq!(text(0.75, "[$-409]h A/P"), "6 PM");
    assert_eq!(text(0.25, "[$-40C]h a/p"), "6 AM");
    assert_eq!(text(0.75, "[$-411]h:mm am/pm"), "6:00 午後");
    assert_eq!(text(0.25, "[$-411]AM/PM h"), "午前 6");
    // Unresolved tags change nothing.
    assert_eq!(text(0.75, "[$-zzz]h A/P"), "6 P");
    assert_eq!(text(0.25, "[$-FFFF]h a/p"), "6 a");
}

#[test]
fn system_tags_replace_the_section() {
    assert_eq!(text(45000.75, "x[$-F800]dddd"), "Wednesday, March 15, 2023");
    assert_eq!(text(45000.75, "[$-F800]0"), "Wednesday, March 15, 2023");
    assert_eq!(text(45000.75, "[$-F400]h:mm"), "6:00:00 PM");
    assert_eq!(text(0.75, "[$-x-sysdate]"), "Saturday, January 0, 1900");
    assert_eq!(text(-1.0, "[$-F800]"), "#VALUE!");
    assert_eq!(text(-1.0, "0;[$-F800]"), "");
    assert_eq!(text(0.0, "0;0;[$-F800]"), "");
    assert_eq!(text(45000.75, "[$-F800]@"), "45000.75");
    assert_eq!(format_text_value("abc", "0;0;0;[$-F800]").text, "");
}

#[test]
fn a_second_tag_in_a_section_is_an_error() {
    assert_eq!(text(45000.0, "[$-40C]mmmm [$-40C]mmmm"), "#VALUE!");
    assert_eq!(text(45000.0, "[$-40C]mmmm [$-zzz]mmmm"), "#VALUE!");
    assert_eq!(text(45000.75, "[$-F400][$-F800]"), "#VALUE!");
    assert_eq!(text(45000.0, "[$-40C]0;[$-407]mmmm"), "45000");
}

#[test]
fn tags_resolve_like_excel() {
    let names = |tag: &str| match resolve(tag) {
        LocaleTag::Names(n) => Some(n.names.months[2]),
        _ => None,
    };
    assert_eq!(names("40c"), Some("mars"));
    assert_eq!(names("1040C"), Some("mars"));
    assert_eq!(names("0040C"), Some("mars"));
    assert_eq!(names("80C"), Some("mars"));
    assert_eq!(names("C"), Some("mars"));
    assert_eq!(names("4"), Some("三月"));
    assert_eq!(names("zh"), Some("三月"));
    assert_eq!(names("FR-fr"), Some("mars"));
    assert_eq!(names("fr-XX"), Some("mars"));
    assert_eq!(names("de"), Some("März"));
    assert_eq!(names("400"), Some("March"));
    for silent in [
        "", "zzz", "xx", "FFFF", "40C0", "0", "2000000", "11040C", "40C,1", "+40C",
    ] {
        assert_eq!(resolve(silent), LocaleTag::Silent, "{silent}");
    }
    assert_eq!(resolve("F800"), LocaleTag::SystemDate);
    assert_eq!(resolve("X-SYSTIME"), LocaleTag::SystemTime);
}

// Values from probes/text-locale-calendars, text-locale-calendars-detail and
// text-locale-japanese-year (T-1859).

#[test]
fn japanese_eras() {
    assert_eq!(text(45000.0, "[$-411]g|gg|ggg|e|ee"), "R|令|令和|5|05");
    assert_eq!(text(45000.0, "[$-ja-JP]g|gg|ggg|e|ee"), "R|令|令和|5|05");
    assert_eq!(text(43585.0, "[$-411]g|e|yyyy-mm-dd"), "H|31|31-04-30");
    assert_eq!(
        text(43586.0, "[$-411]ggge\"年\"m\"月\"d\"日\""),
        "令和1年5月1日"
    );
    assert_eq!(text(4595.0, "[$-411]g|ee"), "T|01");
    assert_eq!(text(9855.0, "[$-411]g|e"), "T|15");
    assert_eq!(text(9856.0, "[$-411]g|e"), "S|1");
    assert_eq!(text(0.0, "[$-411]g|e|yyyy-mm-dd"), "M|33|33-01-00");
    assert_eq!(text(2_958_465.0, "[$-411]g|e|yyyy"), "R|7981|7981");
    assert_eq!(text(45000.0, "[$-411]gggg|ggggg|eee"), "令和|令和|05");
    // The locale's own calendar counts `yy`/`yyyy` in eras only after a `g`; calendar 3 always.
    assert_eq!(text(45000.0, "[$-411]yy|g|yy|yyyy"), "23|R|05|05");
    assert_eq!(text(45000.0, "[$-411]e|yy|yyyy"), "5|23|2023");
    assert_eq!(text(45000.0, "[$-30411]yy|b|e|ggg"), "05|23|5|令和");
    assert_eq!(text(45000.0, "[$-30409]yyyy|mmmm|g"), "05|March|R");
    assert_eq!(text(45000.0, "[$-10411]g|e|yy"), "|2023|2023");
    assert_eq!(text(45000.0, "[$-70411]g|yyyy|mmmm"), "|2023|มีนาคม");
    // Other LCIDs keep their calendar's year after a `g` (probes/text-locale-era-gregorian).
    assert_eq!(text(45000.0, "[$-409]g|yy|yyyy"), "|23|2023");
    assert_eq!(text(45000.0, "[$-70409]g|yy|yyyy"), "|66|2566");
    assert_eq!(text(45000.0, "[$-50409]g|yy|yyyy"), "|56|4356");
}

#[test]
fn calendar_digits() {
    assert_eq!(text(45000.0, "[$-20411]mmmm|e|g"), "March|2023|");
    assert_eq!(text(45000.0, "[$-40409]ddd|mmm|yy|b"), "週三|三月|2023|23");
    assert_eq!(
        text(45000.0, "[$-50409]yyyy|yy|b|e|mmmm"),
        "4356|56|23|2023|March"
    );
    assert_eq!(text(45000.0, "[$-50412]mmmm"), "3월");
    assert_eq!(
        text(45000.0, "[$-70409]yyyy|yy|bbbb|e|mmmm"),
        "2566|66|2566|2023|มีนาคม"
    );
    assert_eq!(text(45000.0, "[$-90409]ddd|mmmm"), "mer.|mars");
    assert_eq!(text(45000.0, "[$-D0409]mmmm"), "March");
    assert_eq!(text(45000.0, "[$-FF040C]mmmm"), "mars");
    // Day halves follow the LCID, not the calendar.
    assert_eq!(text(45000.75, "[$-30409]h AM/PM"), "6 PM");
    assert_eq!(text(45000.75, "[$-30411]h A/P"), "6 午後");
    assert_eq!(text(45000.75, "[$-40409]h AM/PM"), "6 PM");
}

#[test]
fn buddhist_and_calendar_tokens_under_a_tag() {
    assert_eq!(text(45000.0, "b|bbbb"), "66|2566");
    assert_eq!(text(45000.0, "[$-40C]b|bb|bbb|bbbb"), "23|23|2023|2023");
    assert_eq!(text(45000.0, "[$-411]b|bbbb"), "23|2023");
    assert_eq!(text(45000.0, "[$-40C]B2mmmm"), "mars");
    assert_eq!(text(45000.0, "[$-401]B2yyyy-mm-dd"), "2023-03-15");
    assert_eq!(text(45000.0, "B1[$-411]g|e"), "R|5");
}
