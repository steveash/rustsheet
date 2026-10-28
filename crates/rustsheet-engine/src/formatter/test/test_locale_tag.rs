use super::text;
use crate::formatter::{
    format::format_text_value,
    locale_tag::{resolve, LocaleTag},
};

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
    // Lunar calendar digits (probes/text-locale-lunar-edges, text-locale-lunar-tokens A61-A74).
    assert_eq!(names("11040C"), Some("三月"));
    assert_eq!(names("E0412"), Some("삼월"));
    assert_eq!(names("120411"), Some("March"));
    assert_eq!(names("6040C"), Some("Rabi' Aoual"));
    for silent in [
        "", "zzz", "xx", "FFFF", "40C0", "0", "2000000", "40C,1", "+40C",
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

// probes/t1924-locale-tag-extended-hex and text-locale-calendars A136-A142 (T-1924).
#[test]
fn a_tag_neither_hex_nor_bcp47_reads_as_extended_hex() {
    // Letters count from 10 in either case, `_` from `0` (47), `~` and `{` from `a`.
    assert_eq!(text(45000.0, "[$-g]mmmm"), "marzo");
    assert_eq!(text(45000.0, "[$-h]mmmm|e"), "3月|5");
    assert_eq!(text(45000.0, "[$-3gC]mmmm"), "mars");
    assert_eq!(text(45000.0, "[$-2wC]mmmm"), "mars");
    assert_eq!(text(45000.0, "[$-6g40C]mmmm|yyyy"), "มีนาคม|2566");
    assert_eq!(text(45000.0, "[$-r]mmmm|dddd"), "marec|streda");
    // An LCID with a sublanguage and no table has no locale: 0x2412 is not Korean.
    assert_eq!(text(45000.0, "[$-x_y]mmmm|yyyy-mm-dd"), "March|2023-03-15");
    assert_eq!(text(0.75, "[$-Gg]h A/P"), "6 P");
    assert_eq!(text(0.75, "[$-zzzz]h A/P"), "6 P");
    // A BCP-47 language still comes first (`fr-FR_x` is French).
    assert_eq!(text(45000.0, "[$-fr-FR_x]mmmm"), "mars");
}

#[test]
fn a_calendar_digit_over_an_lcid_without_a_locale_keeps_its_date() {
    // de_DE is 0xE0FDE and en_US 0xFA0FC (calendars E and F): lunar dates with en-US names.
    assert_eq!(
        text(45000.0, "[$-de_DE]mmmm|yyyy-mm-dd|dddd|e"),
        "February|2023-02-24|Wednesday|2023"
    );
    assert_eq!(
        text(45000.0, "[$-en_US]mmmm|yyyy-mm-dd"),
        "February|2023-02-24"
    );
    assert_eq!(
        text(45000.0, "[$-E08DE]mmmm|yyyy-mm-dd"),
        "February|2023-02-24"
    );
    // Thai and Arabic calendars keep their own names; A/P stays as written.
    assert_eq!(text(45000.0, "[$-708DE]mmmm|yyyy"), "มีนาคม|2566");
    assert_eq!(text(45000.0, "[$-a_b_c]mmmm|dddd"), "مارس|الأربعاء");
    assert_eq!(text(0.75, "[$-E08DE]h A/P"), "6 P");
    assert_eq!(text(0.75, "[$-708DE]h AM/PM"), "6 PM");
    // LCID 0 is one too (T-2764's cells).
    assert_eq!(text(45000.0, "[$-110000]mmmm|d"), "February|24");
    assert_eq!(text(45000.0, "[$-120000]mmmm|d"), "February|24");
    assert_eq!(text(45000.0, "[$-E0000]mmmm|dddd"), "February|Wednesday");
    // A lunar digit wins over a `B2` before the tag (probes/text-locale-lunar-tokens A79).
    assert_eq!(text(45300.0, "B2[$-110409]yyyy|m"), "2023|12");
}

#[test]
fn hijri_under_a_tag() {
    // probes/t2729-hijri-locale-tag and t2729-hijri-locale-tag-names (excel-graph, 2026-10-27).
    assert_eq!(
        text(45000.0, "[$-60409]yyyy-mm-dd|mmm|dddd"),
        "1444-08-23|Sha'ban|AlArbia'a"
    );
    assert_eq!(text(45000.0, "[$-6040C]mmmm|ddd"), "Chaaban|mer.");
    assert_eq!(text(45000.0, "[$-6041F]mmmm|ddd"), "Şaban|Çar");
    assert_eq!(text(45000.0, "[$-60401]mmmm|dddd"), "شعبان|الأربعاء");
    assert_eq!(text(45000.0, "[$-608DE]mmmm|dddd"), "شعبان|الأربعاء");
    assert_eq!(text(45000.0, "[$-60409]yy|e|g|b|bbbb"), "44|1444||44|1444");
    assert_eq!(text(45000.0, "B2[$-40C]mmmm|yyyy"), "Chaaban|1444");
    assert_eq!(text(45000.0, "B2[$-0]mmmm|dddd"), "شعبان|الأربعاء");
    assert_eq!(text(45000.0, "B2[$-FFFF]mmmm|dddd"), "شعبان|الأربعاء");
    assert_eq!(text(45000.0, "B2mmmm[$-40C]"), "Chaaban");
    // Other calendar digits win over the prefix; an unresolved tag leaves a bare `B2`.
    assert_eq!(text(45000.0, "B2[$-10409]yyyy|mmmm"), "2023|March");
    assert_eq!(text(45000.0, "B2[$-zzz]mmmm|dddd"), "شعبان|السبت");
    // A second calendar prefix after a tag is #VALUE!; before it, it is not.
    assert_eq!(text(45000.0, "B2[$-409]B1mmmm"), "#VALUE!");
    assert_eq!(text(45000.0, "[$-60409]B2B1mmmm"), "#VALUE!");
    assert_eq!(text(45000.0, "B2B2[$-409]mmmm"), "Sha'ban");
}

#[test]
fn mmm_takes_a_b2_section_to_the_arabic_path() {
    // probes/t2729-hijri-locale-tag-names A83-A121, A239-A277.
    assert_eq!(
        text(45000.0, "B2[$-409]mmmm|mmm|mmmm"),
        "Sha'ban|شعبان|شعبان"
    );
    assert_eq!(
        text(45000.0, "B2[$-409]dddd|mmm|dddd"),
        "AlArbia'a|شعبان|السبت"
    );
    assert_eq!(
        text(45000.0, "B2[$-409]mmmmm|mmmm|dddd"),
        "S|Sha'ban|AlArbia'a"
    );
    assert_eq!(text(45000.0, "[$-60409]B2mmm|dddd"), "Sha'ban|AlArbia'a");
    assert_eq!(text(45000.0, "B2dddd|mmm|dddd"), "الأربعاء|شعبان|السبت");
    assert_eq!(text(45000.0, "B2dddd|mmmmm"), "الأربعاء|الأربعاء");
    // A number token first turns the first-name quirk off (probes/text-locale-calendars A124).
    assert_eq!(
        text(45000.0, "B2yyyy-mm-dd|mmmm|dddd"),
        "1444-08-23|شعبان|الأربعاء"
    );
}

#[test]
fn b2_before_calendars_3_5_and_7_converts_the_years_twice() {
    // probes/t2932-b2-calendar-twice A1-A68, B1-B5.
    assert_eq!(
        text(45000.0, "B2[$-70409]yyyy|yy|e|ee|b|bb|bbbb|g"),
        "3109|09|2566|2566|09|09|3109|"
    );
    assert_eq!(
        text(2_958_465.0, "B2[$-70409]yyyy|yy|e|b|bbbb"),
        "11085|85|10542|08|11085"
    );
    assert_eq!(
        text(45000.0, "B2[$-50409]yyyy|yy|e|b|bbbb"),
        "6689|89|4356|56|4356"
    );
    assert_eq!(
        text(45000.0, "B2[$-30411]yyyy|yy|e|ee|g|gg|ggg"),
        "4294965434|4294965434|4294965434|4294965434|M|明|明治"
    );
    assert_eq!(text(32515.0, "B2[$-30409]e|g"), "4294965493|M");
    assert_eq!(text(2_958_465.0, "B2[$-30411]yyyy|e|g"), "5963|5963|R");
    // The era read back changes on its own day: 3930-07-29 and -30 are Reiwa 1912.
    assert_eq!(text(741_653.0, "B2[$-30411]e|g"), "45|M");
    assert_eq!(text(741_654.0, "B2[$-30411]e|g"), "1|T");
    assert_eq!(text(45000.0, "B2[$-70411]g|yyyy"), "|2566");
    assert_eq!(text(45000.0, "B2\"x\"[$-70409]yyyy|e"), "x3109|2566");
    // `B1`, or a `B2` after the tag, changes nothing.
    assert_eq!(text(45000.0, "B1[$-70409]yyyy|e"), "2566|2023");
    assert_eq!(text(45000.0, "[$-70409]B2yyyy|e"), "2566|2023");
    // `b` prints the year's third and fourth digits, also with no `B2`.
    assert_eq!(text(2_958_465.0, "b|bbbb|yy"), "54|10542|99");
    assert_eq!(text(2_958_465.0, "[$-70409]b|yy"), "54|42");
    // Under a tag a number token before the `mmm` switch leaves the first-name quirk on.
    assert_eq!(text(45000.0, "B2[$-409]yyyy|mmm|dddd"), "1444|شعبان|السبت");
}

// An empty text section prints nothing (gen-m10/0e3d8eba9432a18d T!A24; `TRUE` and `0;0;0;` in
// probes/text-format-text-sections, T-2975); fewer than four sections leave the text as it is
// (docs/excel-semantics/number-formats.md; three sections probed there, one or two unprobed).
#[test]
fn empty_text_section_is_empty() {
    assert_eq!(format_text_value("text", ";;;").text, "");
    assert_eq!(format_text_value("TRUE", ";;;").text, "");
    assert_eq!(format_text_value("x", "4;b").text, "x");
    assert_eq!(format_text_value("x", "0;0;General").text, "x");
    assert_eq!(format_text_value("x", "0;0;0;").text, "");
}
