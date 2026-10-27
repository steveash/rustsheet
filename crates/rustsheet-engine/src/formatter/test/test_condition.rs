#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::formatter::lexer::{Compare, Lexer, Token};

#[test]
fn condition() {
    let mut lexer = Lexer::new("[<100]");
    let token = lexer.next_token();
    assert!(matches!(token, Token::Condition(Compare::LessThan, 100.0)));
}

#[test]
fn condition_negative() {
    let mut lexer = Lexer::new("[>-100]");
    let token = lexer.next_token();
    assert!(matches!(
        token,
        Token::Condition(Compare::GreaterThan, -100.0)
    ));
    assert!(matches!(lexer.next_token(), Token::EOF));
}

#[test]
fn condition_period() {
    let mut lexer = Lexer::new("[<=.5]");
    let token = lexer.next_token();
    assert!(matches!(
        token,
        Token::Condition(Compare::LessOrEqualThan, 0.5)
    ));
    assert!(matches!(lexer.next_token(), Token::EOF));
}

#[test]
fn condition_scientific() {
    let mut lexer = Lexer::new("[<=-1.5E2]");
    let token = lexer.next_token();
    assert!(matches!(
        token,
        Token::Condition(Compare::LessOrEqualThan, -150.0)
    ));
    assert!(matches!(lexer.next_token(), Token::EOF));
}

#[test]
fn condition_without_number() {
    let mut lexer = Lexer::new("[>]");
    let token = lexer.next_token();
    assert!(matches!(token, Token::Condition(Compare::GreaterThan, 0.0)));
    assert!(matches!(lexer.next_token(), Token::EOF));
}

#[test]
fn condition_largest_number() {
    let mut lexer = Lexer::new("[>9.99999999999999e307]");
    let token = lexer.next_token();
    assert!(matches!(
        token,
        Token::Condition(Compare::GreaterThan, 9.99999999999999e307)
    ));
    assert!(matches!(lexer.next_token(), Token::EOF));
}

// A number too large to hold is not a number, and it is not a missing number
// either: the condition is not `[>0]`, it is wrong.
#[test]
fn condition_number_too_large() {
    for format in ["[>1e400]", "[<-1e400]", "[=1E+999]", "[>=123e400]0"] {
        let mut lexer = Lexer::new(format);
        assert!(matches!(lexer.next_token(), Token::ILLEGAL), "{format}");
    }
}

#[test]
fn condition_number_unfinished() {
    for format in ["[>-]", "[<.]", "[>-.]"] {
        let mut lexer = Lexer::new(format);
        assert!(matches!(lexer.next_token(), Token::ILLEGAL), "{format}");
    }
}

#[test]
fn condition_invalid() {
    let mut lexer = Lexer::new("[<abc]");
    let token = lexer.next_token();
    assert!(matches!(token, Token::ILLEGAL));
    assert!(matches!(lexer.next_token(), Token::EOF));
}

#[test]
fn elapsed_time() {
    let mut lexer = Lexer::new("[hh]:mm:ss");
    let token = lexer.next_token();
    assert!(matches!(token, Token::ElapsedHourPadded));
    let token = lexer.next_token();
    assert!(matches!(token, Token::Literal(':')));
    let token = lexer.next_token();
    // We can't distinguish between minute and month here
    assert!(matches!(token, Token::MonthPadded));
    let token = lexer.next_token();
    assert!(matches!(token, Token::Literal(':')));
    let token = lexer.next_token();
    assert!(matches!(token, Token::SecondPadded));
    assert!(matches!(lexer.next_token(), Token::EOF));
}

#[test]
fn elapsed_hour() {
    let mut lexer = Lexer::new("[H]");
    let token = lexer.next_token();
    assert!(matches!(token, Token::ElapsedHour));
    assert!(matches!(lexer.next_token(), Token::EOF));
}

#[test]
fn elapsed_minute() {
    let mut lexer = Lexer::new("[M]");
    let token = lexer.next_token();
    assert!(matches!(token, Token::ElapsedMinute));
    assert!(matches!(lexer.next_token(), Token::EOF));
}

#[test]
fn elapsed_second() {
    let mut lexer = Lexer::new("[S]");
    let token = lexer.next_token();
    assert!(matches!(token, Token::ElapsedSecond));
    assert!(matches!(lexer.next_token(), Token::EOF));
}

#[test]
fn elapsed_minute_padded() {
    let mut lexer = Lexer::new("[MM]");
    let token = lexer.next_token();
    assert!(matches!(token, Token::ElapsedMinutePadded));
    assert!(matches!(lexer.next_token(), Token::EOF));
}

#[test]
fn elapsed_second_padded() {
    let mut lexer = Lexer::new("[SS]");
    let token = lexer.next_token();
    assert!(matches!(token, Token::ElapsedSecondPadded));
    assert!(matches!(lexer.next_token(), Token::EOF));
}

#[test]
fn too_many_h() {
    let mut lexer = Lexer::new("[HHH]");
    let token = lexer.next_token();
    assert!(matches!(token, Token::ILLEGAL));
    assert!(matches!(lexer.next_token(), Token::EOF));
}

#[test]
fn condition_not_equal_and_spaces_after_the_comparator() {
    // Excel (poi FormatChoiceTests A17-A19, A23-A26).
    let mut lexer = Lexer::new("[<>10]");
    assert!(matches!(
        lexer.next_token(),
        Token::Condition(Compare::NotEqual, 10.0)
    ));
    let mut lexer = Lexer::new("[<      10]");
    assert!(matches!(
        lexer.next_token(),
        Token::Condition(Compare::LessThan, 10.0)
    ));
}

#[test]
fn an_unmatched_single_condition_prints_general() {
    use crate::formatter::format::{format_number, FormatContext};
    let ctx = FormatContext::new(crate::locale::get_locale("en").unwrap());
    let text = |v: f64, f: &str| format_number(v, f, &ctx).text;
    assert_eq!(text(10.0, "[<10]#\" Wow\""), "10");
    assert_eq!(text(1.0, "[<10]#\" Wow\""), "1 Wow");
    assert_eq!(text(1.0, "[<>10]#\" Wow\""), "1 Wow");
    assert_eq!(text(10.0, "[<>10]#\" Wow\""), "10");
    assert_eq!(text(10.0, "[< 10]#\" Wow\""), "10");
    assert_eq!(text(1.0, "[>5]d"), "1");
    // The signed value is compared and keeps its minus (poi FormatConditionTests A32, A35).
    assert_eq!(text(-11.0, "[=-10]#\" Wow\""), "-11");
    assert_eq!(text(-11.0, "[<>-10]#\" Wow\""), "-11 Wow");
}

#[test]
fn a_condition_on_the_first_section_chooses_among_the_sections() {
    use crate::formatter::format::{format_number, FormatContext};
    let ctx = FormatContext::new(crate::locale::get_locale("en").unwrap());
    let text = |v: f64, f: &str| format_number(v, f, &ctx).text;
    // gen-m10 039accabcfe1e6c7 F!Q (excel-graph): the third section takes the rest.
    assert_eq!(text(1.0, "[<0]\"neg\";[>0]\"pos\";\"zero\""), "pos");
    assert_eq!(text(999.9999, "[<100]0.0;[>=1000]#,##0;0"), "1000");
    assert_eq!(
        text(1_234_567_890.0, "[<=-1]0;[<1]0.0000;0.0"),
        "1234567890.0"
    );
    // The sign goes only for a condition that holds for negatives alone.
    assert_eq!(text(-1.5, "[<=-1]0;[<1]0.0000;0.0"), "2");
    assert_eq!(text(-0.5, "[<=-1]0;[<1]0.0000;0.0"), "-0.5000");
    assert_eq!(text(-0.5, "[<100]0.0;[>=1000]#,##0;0"), "-0.5");
    assert_eq!(text(-0.5, "[>=1]0.00;0.000"), "-0.500");
    assert_eq!(text(0.5, "[>=1]0.00;0.000"), "0.500");
    assert_eq!(text(-123_456.789, "[=0]\"nil\";General"), "-123456.789");
    assert_eq!(text(0.0, "[=0]\"nil\";General"), "nil");
}

#[test]
fn cyan_color_is_accepted() {
    use crate::formatter::format::{format_number, FormatContext};
    let ctx = FormatContext::new(crate::locale::get_locale("en").unwrap());
    let text = |v: f64, f: &str| format_number(v, f, &ctx).text;
    // Excel (gen-m10, T-1737): `[Cyan]` is one of the eight color names, not #VALUE!.
    assert_eq!(text(0.0, "[Green]#,##0;[Magenta](#,##0);[Cyan]0"), "0");
    assert_eq!(text(5.0, "[cyan]0"), "5");
}
