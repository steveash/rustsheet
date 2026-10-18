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
        LocaleTag::Names(n) => Some(n.months[2]),
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
