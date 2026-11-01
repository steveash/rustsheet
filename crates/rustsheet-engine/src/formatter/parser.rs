use super::elapsed::takes_second_fraction;
pub use super::fraction::{Denominator, Fraction};
use super::lexer::{Compare, Lexer, Token};
use super::locale_tag::{self, Calendar, DateNames, LocaleTag};
use super::numerals::Numerals;
use crate::locale::excel_date_names::EN_US;

pub struct Digit {
    pub kind: char, // '#' | '?' | '0'
    pub index: i32,
    pub number: NumberState, // 'i' | 'd' | 'e' (integer, decimal or exponent)
}

pub enum TextToken {
    Literal(char),
    Text(String),
    Ghost(char),
    Spacer(char),
    // Text
    Raw,
    /// `General` among literals (`General x`)
    General,
    Digit(Digit),
    Period,
    // Dates
    Day,
    DayPadded,
    DayNameShort,
    DayName,
    Month,
    MonthPadded,
    MonthNameShort,
    MonthName,
    MonthLetter,
    YearShort,
    Year,
    /// `e` (false) or `ee`+ (true).
    EraYear(bool),
    /// `g`, `gg` or `ggg`+: the era's letter, first character or name.
    Era(usize),
    BuddhistYearShort,
    BuddhistYear,
    Hour,
    HourPadded,
    Minute,
    MinutePadded,
    Second,
    SecondPadded,
    ElapsedHour,
    ElapsedHourPadded,
    ElapsedMinute,
    ElapsedMinutePadded,
    ElapsedSecond,
    ElapsedSecondPadded,
    /// `.0`, `.00` or `.000` right after a seconds token: that many digits of the second.
    SecondFraction(usize),
    AMPM,
    AP(char, char),
}

impl TextToken {
    /// Whether the token prints fixed text and no value (a literal, quoted text, a ghost or a fill).
    pub fn is_literal(&self) -> bool {
        matches!(
            self,
            TextToken::Literal(_) | TextToken::Text(_) | TextToken::Ghost(_) | TextToken::Spacer(_)
        )
    }
}
pub struct NumberPart {
    pub color: Option<i32>,
    pub condition: Option<(Compare, f64)>,
    pub use_thousands: bool,
    pub percent: i32, // multiply number by 100^percent
    /// Scaling commas: scale the number by 10^-3n, the exponent wrapping in a signed byte, or in a
    /// fraction section an unsigned one (`fraction::comma_exponent`).
    pub comma: i32,
    pub tokens: Vec<TextToken>,
    pub digit_count: i32, // number of digit tokens (#, 0 or ?) to the left of the decimal point
    pub precision: i32,   // number of digits to the right of the decimal point
    pub is_scientific: bool,
    pub scientific_minus: bool,
    pub exponent_digit_count: i32,
    pub currency: Option<char>,
    /// A fraction (`# ?/4`, `?/100`, `# ??/??`); `None` for any other section.
    pub fraction: Option<Fraction>,
    /// The digits of the section's locale tag (`locale_tag::numerals`).
    pub numerals: Numerals,
}

/// The most digits a fixed fraction denominator may have; more is `#VALUE!`.
const MAX_FIXED_DENOMINATOR_DIGITS: usize = 5;

pub struct DatePart {
    pub color: Option<i32>,
    pub condition: Option<(Compare, f64)>,
    pub use_ampm: bool,
    pub tokens: Vec<TextToken>,
    /// The names of the section's `[$-...]` locale; `None` uses the workbook locale.
    pub names: Option<&'static DateNames>,
    /// What `AM/PM` and `A/P` print under the section's locale tag.
    pub day_half: Option<&'static DateNames>,
    /// The calendar of the section's locale tag; `None` for a section with none.
    pub calendar: Option<Calendar>,
    /// Whether the section's locale tag names a Japanese LCID (`Localized::japanese_lcid`).
    pub japanese_lcid: bool,
    /// Whether a `B2` before the tag converts the years once more (`Localized::year_twice`).
    pub year_twice: bool,
    /// Whether a `B2` prefix before any tag puts the section in the Hijri calendar
    /// (`date_names::hijri_ymd`) and, with no tag or from its first `mmm` on, Excel's Arabic
    /// name path (`date_names::NameTokens`).
    pub hijri: bool,
    /// Whether a `B2` prefix came before a tag whose calendar wins over it: a weekday after a
    /// `mmm` then reads the month number (`date_names::NameTokens`).
    pub b2_overridden: bool,
    /// The names from the section's first `mmm` on (`Localized::names_after_mmm`).
    pub names_after_mmm: Option<&'static DateNames>,
    /// The digits of the section's locale tag (`locale_tag::numerals`).
    pub numerals: Numerals,
}

#[derive(Default)]
pub struct ErrorPart {
    /// Whether the error rejects the whole code, also for a value another section formats: a bare
    /// digit beside a date token does (`TEXT(3.14159,"0;4b")` is #VALUE!;
    /// probes/text-format-digits-date-token A42). Other section errors are not probed that way,
    /// and some come from tokens the engine does not implement (`[Cyan]`, T-1737).
    pub whole_code: bool,
}

/// A section that is `General` alone, after any locale tag.
pub struct GeneralPart {
    /// The digits of the section's locale tag (`[$-1B000409]General` on 12 is "十二").
    pub numerals: Numerals,
}

pub enum ParsePart {
    Number(NumberPart),
    Date(DatePart),
    Error(ErrorPart),
    General(GeneralPart),
}

pub struct Parser {
    pub parts: Vec<ParsePart>,
    /// Whether the format ends in a `;`, so an empty section follows the last part.
    ends_in_separator: bool,
    lexer: Lexer,
    /// The numeral system of the first section's locale tag, which every section prints in: a
    /// later section's tag changes no digits, even its own (`[$-2000409]0;(0)` on -5 is "(٥)",
    /// `0;[$-2000409]0;0` "5"; probes/t3532-numeral-digit-fields A17, t3532-numeral-digit-sections).
    pub numerals: Numerals,
}

/// The most digits of a second a format may show (`.000`, poi/ElapsedFormatTests A2).
const MAX_SECOND_FRACTION_DIGITS: usize = 3;

#[derive(PartialEq, Copy, Clone)]
pub enum NumberState {
    Integer,
    Decimal,
    Exponent,
}

/// The kind (`#`, `?` or `0`) of a digit placeholder token.
fn placeholder_kind(token: &Token) -> char {
    match token {
        Token::QuestionMark => '?',
        Token::Zero => '0',
        _ => '#',
    }
}

/// Whether `token` is a literal that may stand between a fraction's numerator, its `/` and its
/// denominator: any literal but a digit or a `/`, escaped or quoted text, or a `_x` space.
fn is_fraction_literal(token: &Token) -> bool {
    match token {
        Token::Literal(c) => !c.is_ascii_digit() && *c != '/',
        Token::Escaped(_) | Token::Text(_) | Token::Ghost(_) => true,
        _ => false,
    }
}

/// The text token a fraction literal (see [`is_fraction_literal`]) prints as.
fn fraction_literal(token: Token) -> Option<TextToken> {
    match token {
        Token::Literal(c) | Token::Escaped(c) => Some(TextToken::Literal(c)),
        Token::Text(text) => Some(TextToken::Text(text)),
        Token::Ghost(c) => Some(TextToken::Ghost(c)),
        _ => None,
    }
}

/// Whether `token` prints part of a date or a clock time: AM/PM alone does not count. A negative
/// list on purpose: every `TextToken` outside it is a date or time token.
fn is_calendar_or_clock(token: &TextToken) -> bool {
    !matches!(
        token,
        TextToken::Literal(_)
            | TextToken::Text(_)
            | TextToken::Ghost(_)
            | TextToken::Spacer(_)
            | TextToken::Raw
            | TextToken::General
            | TextToken::Digit(_)
            | TextToken::Period
            | TextToken::AMPM
            | TextToken::AP(..)
    )
}

/// Whether `token` starts a fraction's denominator: a digit placeholder or a digit 1-9.
fn starts_denominator(token: &Token) -> bool {
    token.is_digit() || matches!(token, Token::Literal('1'..='9'))
}

impl NumberState {
    pub fn is_integer(&self) -> bool {
        matches!(self, NumberState::Integer)
    }

    pub fn is_decimal(&self) -> bool {
        matches!(self, NumberState::Decimal)
    }

    pub fn is_exponent(&self) -> bool {
        matches!(self, NumberState::Exponent)
    }
}

impl ParsePart {
    pub fn is_error(&self) -> bool {
        match &self {
            ParsePart::Date(..) => false,
            ParsePart::Number(..) => false,
            ParsePart::Error(..) => true,
            ParsePart::General(..) => false,
        }
    }
    pub fn is_date(&self) -> bool {
        match &self {
            ParsePart::Date(..) => true,
            ParsePart::Number(..) => false,
            ParsePart::Error(..) => false,
            ParsePart::General(..) => false,
        }
    }
}

// Numbers:
// [integer section][decimal point][fractional section][optional exponent]
// So #,##0.00 is valid but 0.00#,## is not.

impl Parser {
    pub fn new(format: &str) -> Self {
        let lexer = Lexer::new(format);
        let parts = vec![];
        Parser {
            parts,
            ends_in_separator: false,
            lexer,
            numerals: Numerals::Ascii,
        }
    }
    /// Parses every section. A trailing `;` opens one more, empty section, as a `;` between two
    /// sections does: Excel prints nothing for a value that falls in it (`TEXT(-5,"0;")`,
    /// `TEXT(0,"0;0;")` and `TEXT("text",";;;")` are ""; probes/text-value-kind-sweep-b C2, D3,
    /// gen-m10/0e3d8eba9432a18d T!A24). After four sections it is left to the section-count check.
    pub fn parse(&mut self) {
        while self.lexer.peek_token() != Token::EOF {
            let part = self.parse_part();
            self.parts.push(part);
        }
        if self.ends_in_separator && self.parts.len() < 4 {
            let part = self.parse_part();
            self.parts.push(part);
        }
    }

    /// Whether `m`/`mm` read just before `next_token` is a minute because `:ss` follows with no
    /// hour ahead of it: `mm:ss` on 123456.789 is "56:10", not a month (gen-m10 text cells,
    /// T-1733; number-formats.md).
    fn minute_before_seconds(&mut self, next_token: &Token) -> bool {
        *next_token == Token::Literal(':')
            && matches!(self.lexer.peek_token(), Token::Second | Token::SecondPadded)
    }

    /// Whether `m`/`mm` before `next_token` is a minute because `.0` follows, wherever it stands
    /// (`mm.0 yyyy` is "16.2 1933"; probes/t3575-fraction-after-elapsed-b; number-formats.md).
    fn minute_before_fraction(&mut self, next_token: &Token) -> bool {
        *next_token == Token::Period && self.lexer.peek_token() == Token::Zero
    }

    fn parse_part(&mut self) -> ParsePart {
        // Set from the last part read; error parts never reach the empty-section check.
        self.ends_in_separator = false;
        let mut token = self.lexer.next_token();
        let mut digit_count = 0;
        let mut precision = 0;
        let mut is_date = false;
        let mut use_ampm = false;
        let mut is_number = false;
        let mut found_decimal_dot = false;
        let mut use_thousands = false;
        let mut comma = 0;
        let mut percent = 0;
        let mut last_token_is_digit = false;
        // Whether the previous token is a `%`, or ends a fraction's denominator (or is a scaling
        // comma right after one): a `,` there is a literal or a fraction's scaling comma.
        let mut after_percent = false;
        let mut after_denominator = false;
        // Just past the last digit placeholder while only literals follow it, which may stand
        // between a numerator and its `/` (`#\:#=/=#`, poi/NumberFormatTests A134).
        let mut after_digits: Option<usize> = None;
        let mut color = None;
        let mut condition = None;
        let mut tokens = vec![];
        let mut is_scientific = false;
        let mut scientific_minus = false;
        let mut exponent_digit_count = 0;
        let mut number = NumberState::Integer;
        let mut index = 0;
        let mut currency = None;
        // Where each currency symbol sits among `tokens`, for a date section to print it there.
        let mut currency_positions: Vec<(usize, char)> = Vec::new();
        let mut is_time = false;
        // Excel: after a minute, later `m`/`mm` are months (number-formats.md "`mm` after a minute").
        let mut minute_read = false;
        let mut hijri = false;
        let mut calendar_prefix = false;
        // Whether the previous token is an unescaped colon, which the next one joins.
        let mut after_colon = false;
        let mut locale = None;
        // Excel: a second locale tag in a section is #VALUE!, even the same one
        // (`TEXT(45000,"[$-40C]mmmm [$-40C]mmmm")`; probes/text-locale-name-tables A568-A575).
        let mut repeated_locale = false;
        let mut numerals = self.numerals;
        let mut fraction = None;
        // Excel: a plain `h` or `s`/`ss` after `[h]`/`[hh]` or `[s]`/`[ss]` shows the elapsed
        // count too ("It was 75 [yes, 75] hours", poi/ElapsedFormatTests A10, A11, A21;
        // number-formats.md "Elapsed-time tokens"). `m` after `[m]` is unprobed and stays a clock
        // minute.
        let mut elapsed_hour = false;
        // Bracketed `[h]`/`[m]`/`[s]` tokens so far (a bare `h` after `[h]` is not one).
        let mut bracketed_elapsed = 0_usize;
        let mut elapsed_second = false;
        // Whether a bare (unquoted, unescaped) digit 1-9 was read: it is a literal, but not beside
        // a date or time token (`is_calendar_or_clock`).
        let mut bare_digit = false;

        while token != Token::EOF && token != Token::Separator {
            let mut next_token = self.lexer.next_token();
            let token_is_digit = token.is_digit();
            let token_is_fraction_literal = is_fraction_literal(&token);
            is_number = is_number || token_is_digit;
            let next_token_is_digit = next_token.is_digit();
            let is_colon = token == Token::Literal(':');
            let is_percent = token == Token::Percent;
            let is_comma = token == Token::Comma;
            let mut ends_denominator = false;
            if token_is_digit {
                if is_scientific {
                    exponent_digit_count += 1;
                } else if found_decimal_dot {
                    precision += 1;
                } else {
                    digit_count += 1;
                }
            }
            match token {
                Token::General => {
                    if tokens.is_empty() && matches!(next_token, Token::EOF | Token::Separator) {
                        self.ends_in_separator = next_token == Token::Separator;
                        return ParsePart::General(GeneralPart { numerals });
                    }
                    // Excel: literals around `General` print around the value (`TEXT(5,"General x")`
                    // = "5 x", `TEXT(5,"x General")` = "x 5"; probes/text-format-bare-literals
                    // A75, A76). Next to placeholders or a second General it stays an error
                    // (probes/text-format-general-company).
                    tokens.push(TextToken::General);
                }
                Token::Comma => {
                    // Excel: in a fraction section only a `,` right after the denominator scales
                    // (`?/?,` on 1E-250 is "1000/1"); a `,` after a `%` or a literal prints, also
                    // outside a fraction (`?/?% ,` and `?/? ,` on 1E-250 are "0/1% ," and "0/1 ,",
                    // `0%,` on 0.5 "50%,"; probes/fraction-overflow-scaled; number-formats.md
                    // "Scaling commas in fraction formats").
                    if after_denominator {
                        comma += 1;
                    } else if after_percent || (fraction.is_some() && !last_token_is_digit) {
                        tokens.push(TextToken::Literal(','));
                    } else if last_token_is_digit && next_token_is_digit {
                        use_thousands = true;
                    } else if digit_count > 0 {
                        comma += 1;
                    } else {
                        // Before the number is just a literal.
                        tokens.push(TextToken::Literal(','));
                    }
                }
                Token::Percent => {
                    tokens.push(TextToken::Literal('%'));
                    percent += 1;
                }
                // `.0`, `.00`, `.000` after an h/m/s token are the fraction of the second, and
                // more zeros are an error (number-formats.md "Elapsed-time tokens").
                Token::Period
                    if next_token == Token::Zero
                        && tokens.last().is_some_and(takes_second_fraction) =>
                {
                    let mut digits = 0;
                    while next_token == Token::Zero {
                        digits += 1;
                        next_token = self.lexer.next_token();
                    }
                    if digits > MAX_SECOND_FRACTION_DIGITS {
                        return ParsePart::Error(ErrorPart::default());
                    }
                    tokens.push(TextToken::SecondFraction(digits));
                }
                Token::Period => {
                    if !is_number && !is_date && !found_decimal_dot {
                        // Excel: a decimal point with no digit placeholder before it shows the
                        // integer part right there, as if it were `#.` (`TEXT(12.3,"x.")` =
                        // "x12.", `TEXT(12.3,"1.0")` = "112.3", `TEXT(0,".")` = ".";
                        // probes/text-format-bare-literals A8-A14, A22-A25).
                        tokens.push(TextToken::Digit(Digit {
                            kind: '#',
                            index,
                            number,
                        }));
                        index += 1;
                        digit_count += 1;
                        is_number = true;
                    }
                    if is_number && !found_decimal_dot {
                        tokens.push(TextToken::Period);
                        found_decimal_dot = true;
                        if number.is_integer() {
                            number = NumberState::Decimal;
                            index = 0;
                        }
                    } else {
                        tokens.push(TextToken::Literal('.'));
                    }
                }
                Token::Color(index) => {
                    color = Some(index);
                }
                Token::Condition(cmp, value) => {
                    condition = Some((cmp, value));
                }
                Token::Currency(c) => {
                    currency = Some(c);
                    currency_positions.push((tokens.len(), c));
                }
                Token::QuestionMark => {
                    tokens.push(TextToken::Digit(Digit {
                        kind: '?',
                        index,
                        number,
                    }));
                    index += 1;
                }
                Token::Sharp => {
                    tokens.push(TextToken::Digit(Digit {
                        kind: '#',
                        index,
                        number,
                    }));
                    index += 1;
                }
                Token::Zero => {
                    tokens.push(TextToken::Digit(Digit {
                        kind: '0',
                        index,
                        number,
                    }));
                    index += 1;
                }
                // A `/` after digit placeholders and before a number or more placeholders makes a
                // fraction (`# ?/4`, `# ??/??`), also with literals on either side of it
                // (`#\:#=/=#`, `#_#/#`; number-formats.md "Fraction formats", poi/NumberFormatTests).
                Token::Literal('/')
                    if after_digits.is_some()
                        && !found_decimal_dot
                        && !is_scientific
                        && fraction.is_none()
                        && (starts_denominator(&next_token)
                            || (is_fraction_literal(&next_token)
                                && starts_denominator(
                                    &self.lexer.peek_token_past(is_fraction_literal),
                                ))) =>
                {
                    let numerator_end = after_digits.unwrap_or(tokens.len());
                    tokens.push(TextToken::Literal('/'));
                    while is_fraction_literal(&next_token) {
                        if let Some(literal) = fraction_literal(next_token) {
                            tokens.push(literal);
                        }
                        next_token = self.lexer.next_token();
                    }
                    let denominator = if let Token::Literal(first) = next_token {
                        let mut digits = String::from(first);
                        loop {
                            match self.lexer.peek_token() {
                                Token::Literal(c) if c.is_ascii_digit() => digits.push(c),
                                Token::Zero => digits.push('0'),
                                _ => break,
                            }
                            self.lexer.next_token();
                        }
                        // Excel: a fixed denominator of six or more digits is #VALUE! from TEXT
                        // (`TEXT(0.5,"?/100000")`, `"?/123456"`; probes/fraction-six-digit-
                        // denominator); a workbook that applies one is refused by Excel for the
                        // web (number-formats.md "Fixed denominators past 32768").
                        if digits.len() > MAX_FIXED_DENOMINATOR_DIGITS {
                            return ParsePart::Error(ErrorPart::default());
                        }
                        Denominator::Fixed(digits)
                    } else {
                        let mut kinds = vec![placeholder_kind(&next_token)];
                        while self.lexer.peek_token().is_digit() {
                            kinds.push(placeholder_kind(&self.lexer.next_token()));
                        }
                        Denominator::Placeholders(kinds)
                    };
                    next_token = self.lexer.next_token();
                    let numerator_len = tokens[..numerator_end]
                        .iter()
                        .rev()
                        .take_while(|t| matches!(t, TextToken::Digit(_)))
                        .count();
                    // Excel: a `,` before the fraction neither scales nor prints (`0, ?/4` on 2 is
                    // "2    ", on 1234567 "1234567    "; probes/fraction-overflow-scaled).
                    comma = 0;
                    ends_denominator = true;
                    fraction = Some(Fraction {
                        numerator_start: numerator_end - numerator_len,
                        numerator_end,
                        denominator_at: tokens.len(),
                        denominator,
                    });
                }
                Token::Literal(value) => {
                    if value == ':' {
                        is_time = true;
                    }
                    bare_digit |= value.is_ascii_digit();
                    // A run of colons prints as one ("0::a" → "1:a"; number-formats.md
                    // "Colon runs", probes/text-format-length-limit-f/g, T-1434).
                    if !(is_colon && after_colon) {
                        tokens.push(TextToken::Literal(value));
                    }
                }
                Token::Escaped(value) => {
                    if value == ':' {
                        is_time = true;
                    }
                    // Escaped colons do not join a run: `TEXT(1,"0\:\:")` is "1::"
                    // (probes/text-format-unquoted-letters A105).
                    tokens.push(TextToken::Literal(value));
                }
                Token::Text(value) => {
                    tokens.push(TextToken::Text(value));
                }
                Token::Ghost(value) => {
                    tokens.push(TextToken::Ghost(value));
                }
                Token::Spacer(value) => {
                    tokens.push(TextToken::Spacer(value));
                }
                Token::Day => {
                    is_date = true;
                    tokens.push(TextToken::Day);
                }
                Token::DayPadded => {
                    is_date = true;
                    tokens.push(TextToken::DayPadded);
                }
                Token::DayNameShort => {
                    is_date = true;
                    tokens.push(TextToken::DayNameShort);
                }
                Token::DayName => {
                    is_date = true;
                    tokens.push(TextToken::DayName);
                }
                Token::MonthNameShort => {
                    is_date = true;
                    tokens.push(TextToken::MonthNameShort);
                }
                Token::MonthName => {
                    is_date = true;
                    tokens.push(TextToken::MonthName);
                }
                Token::Month | Token::MonthPadded => {
                    // A `mm.0` minute leaves `minute_read` ("mm.0 h:mm" is "57.7 2:57"; t3575-b A30).
                    let fraction = self.minute_before_fraction(&next_token);
                    let clock =
                        (is_time && !minute_read) || self.minute_before_seconds(&next_token);
                    let minute = clock || fraction;
                    minute_read |= clock;
                    is_date |= !minute || fraction;
                    tokens.push(match (minute, token == Token::MonthPadded) {
                        (true, false) => TextToken::Minute,
                        (true, true) => TextToken::MinutePadded,
                        (false, false) => TextToken::Month,
                        (false, true) => TextToken::MonthPadded,
                    });
                }
                Token::MonthLetter => {
                    is_date = true;
                    tokens.push(TextToken::MonthLetter);
                }
                Token::YearShort => {
                    is_date = true;
                    tokens.push(TextToken::YearShort);
                }
                Token::Year => {
                    is_date = true;
                    tokens.push(TextToken::Year);
                }
                Token::BuddhistYearShort => {
                    is_date = true;
                    tokens.push(TextToken::BuddhistYearShort);
                }
                Token::BuddhistYear => {
                    is_date = true;
                    tokens.push(TextToken::BuddhistYear);
                }
                Token::EraYear(padded) => {
                    is_date = true;
                    tokens.push(TextToken::EraYear(padded));
                }
                // The era prints nothing for the Gregorian calendar (`TEXT(100,"g")` is "").
                Token::Era(length) => {
                    is_date = true;
                    tokens.push(TextToken::Era(length));
                }
                // `B1` selects the Gregorian calendar and still makes the section a date one
                // (`TEXT(100,"B1")` is ""). `B2` selects the Hijri calendar
                // (`TEXT(45000,"B2yyyy-mm-dd")` is "1444-08-23"; probes/t1613-hijri-b2), also
                // before a tag (the lexer resolves the tag in it). After a tag either changes
                // nothing (`[$-40C]B2mmmm` is "mars"; probes/text-locale-calendars A125-A134), but
                // is #VALUE! after another prefix (`B2[$-409]B1`, `[$-60409]B2B1`;
                // probes/t2729-hijri-locale-tag-names A278-A285).
                Token::Calendar(c) => {
                    if locale.is_some() && calendar_prefix {
                        return ParsePart::Error(ErrorPart::default());
                    }
                    is_date = true;
                    calendar_prefix = true;
                    hijri |= c == '2' && locale.is_none();
                }
                Token::Hour => {
                    is_date = true;
                    is_time = true;
                    tokens.push(if elapsed_hour {
                        TextToken::ElapsedHour
                    } else {
                        TextToken::Hour
                    });
                }
                Token::HourPadded => {
                    is_date = true;
                    is_time = true;
                    tokens.push(if elapsed_hour {
                        TextToken::ElapsedHourPadded
                    } else {
                        TextToken::HourPadded
                    });
                }
                Token::Second => {
                    is_date = true;
                    is_time = true;
                    tokens.push(if elapsed_second {
                        TextToken::ElapsedSecond
                    } else {
                        TextToken::Second
                    });
                }
                Token::SecondPadded => {
                    is_date = true;
                    is_time = true;
                    tokens.push(if elapsed_second {
                        TextToken::ElapsedSecondPadded
                    } else {
                        TextToken::SecondPadded
                    });
                }
                Token::AMPM => {
                    // Excel: a bracketed elapsed token before the marker is #VALUE! (`[ss] AM/PM`,
                    // `[h] AM/PM`, `[mm] AM/PM`, `[s] A/P`) but the marker first is valid
                    // (`AM/PM [h]` on 1.5 is "PM 12"; probes/text-elapsed-two-tokens A1, A9-A11, A13).
                    if bracketed_elapsed > 0 {
                        return ParsePart::Error(ErrorPart::default());
                    }
                    is_date = true;
                    use_ampm = true;
                    tokens.push(TextToken::AMPM);
                }
                Token::AP(am, pm) => {
                    // As for `AM/PM` above: an elapsed token before the marker is #VALUE!.
                    if bracketed_elapsed > 0 {
                        return ParsePart::Error(ErrorPart::default());
                    }
                    is_date = true;
                    use_ampm = true;
                    tokens.push(TextToken::AP(am, pm));
                }
                Token::Locale(tag, tag_numerals) => {
                    repeated_locale |= locale.is_some();
                    locale = Some(tag);
                    if self.parts.is_empty() {
                        (numerals, self.numerals) = (tag_numerals, tag_numerals);
                    }
                }
                Token::Scientific | Token::ScientificMinus => {
                    if !is_scientific {
                        index = 0;
                        number = NumberState::Exponent;
                        // The `E` stays where the format has it, ahead of any literal before the
                        // exponent digits; the exponent's sign goes next to its digits (`|#.#|E+|#|`
                        // on 123456.789 is "|1.2|E|+5|", poi/NumberFormatApproxTests.xlsx A41).
                        tokens.push(TextToken::Literal('E'));
                    }
                    is_scientific = true;
                    scientific_minus |= token == Token::ScientificMinus;
                }
                Token::Separator => {}
                Token::Raw => {
                    tokens.push(TextToken::Raw);
                }
                Token::ILLEGAL => {
                    return ParsePart::Error(ErrorPart::default());
                }
                Token::EOF => {}
                Token::ElapsedHour => {
                    is_date = true;
                    bracketed_elapsed += 1;
                    is_time = true;
                    elapsed_hour = true;
                    tokens.push(TextToken::ElapsedHour);
                }
                Token::ElapsedMinute => {
                    is_date = true;
                    bracketed_elapsed += 1;
                    is_time = true;
                    tokens.push(TextToken::ElapsedMinute);
                }
                Token::ElapsedSecond => {
                    is_date = true;
                    bracketed_elapsed += 1;
                    is_time = true;
                    elapsed_second = true;
                    tokens.push(TextToken::ElapsedSecond);
                }
                Token::ElapsedHourPadded => {
                    is_date = true;
                    bracketed_elapsed += 1;
                    is_time = true;
                    elapsed_hour = true;
                    tokens.push(TextToken::ElapsedHourPadded);
                }
                Token::ElapsedMinutePadded => {
                    is_date = true;
                    bracketed_elapsed += 1;
                    is_time = true;
                    tokens.push(TextToken::ElapsedMinutePadded);
                }
                Token::ElapsedSecondPadded => {
                    is_date = true;
                    bracketed_elapsed += 1;
                    is_time = true;
                    elapsed_second = true;
                    tokens.push(TextToken::ElapsedSecondPadded);
                }
            }
            if token_is_digit {
                after_digits = Some(tokens.len());
            } else if !token_is_fraction_literal {
                after_digits = None;
            }
            last_token_is_digit = token_is_digit;
            after_percent = is_percent;
            after_denominator = ends_denominator || (after_denominator && is_comma);
            after_colon = is_colon;
            token = next_token;
        }
        self.ends_in_separator = token == Token::Separator;
        // Excel: a bare digit 1-9 is a literal (`TEXT(5,"2")` is "2", `"4am/pm"` "4AM"), but beside a
        // date or time token anywhere in the section the code is #VALUE!: `"42abc"`, `"4b"`, `"b4"`,
        // `"4 d"`, `"4:mm"`, `"[h]4"`, `"4g"`; quoted or escaped it prints (`"\4b"` is "443";
        // probes/text-format-digits-date-token; number-formats.md "Bare digits beside a date token").
        if bare_digit && tokens.iter().any(is_calendar_or_clock) {
            return ParsePart::Error(ErrorPart { whole_code: true });
        }
        // Excel: a section with two bracketed elapsed tokens is #VALUE! for any value (`[h]:[mm]`,
        // `[mm]:[ss]`; probes/negative-date-sections-1900 and -1904 A9, A21, A22; dates.md "Negative
        // serials under an explicit negative section"). A bare `h` after `[h]` is not bracketed.
        if bracketed_elapsed > 1 || super::elapsed::elapsed_after_larger_clock(&tokens) {
            return ParsePart::Error(ErrorPart::default());
        }
        if repeated_locale {
            return ParsePart::Error(ErrorPart::default());
        }
        let system_format = match locale {
            Some(LocaleTag::SystemDate) => Some(locale_tag::SYSTEM_LONG_DATE),
            Some(LocaleTag::SystemTime) => Some(locale_tag::SYSTEM_LONG_TIME),
            _ => None,
        };
        if let Some(system_format) = system_format {
            if !tokens.iter().any(|t| matches!(t, TextToken::Raw)) {
                return self.system_part(system_format, color);
            }
        }
        let localized = match locale {
            Some(LocaleTag::Names(localized)) => Some(localized),
            _ => None,
        };
        // A calendar digit other than Hijri wins over a `B2` before the tag (`B2[$-10409]yyyy` is
        // "2023"); an unresolved tag leaves it a bare `B2` (`B2[$-zzz]`).
        let b2_overridden = hijri && localized.is_some_and(|l| l.calendar != Calendar::Hijri);
        hijri &= !b2_overridden;
        let general_count = tokens
            .iter()
            .filter(|t| matches!(t, TextToken::General))
            .count();
        if general_count > 0 {
            let only_literals = tokens
                .iter()
                .all(|t| t.is_literal() || matches!(t, TextToken::General));
            if general_count > 1 || !(only_literals || is_date) || comma > 0 {
                return ParsePart::Error(ErrorPart::default());
            }
            // Excel: `%` beside `General` prints, no scaling (probes/text-format-general-company A3).
            percent = 0;
        }
        if is_date {
            if is_number {
                return ParsePart::Error(ErrorPart::default());
            }
            // Excel prints a currency symbol in a date section where it is written
            // (`[$€-40C]d mmmm` is "€15 mars", `mmmm[$€-40C]` "mars€"; number-formats.md).
            for (position, symbol) in currency_positions.into_iter().rev() {
                tokens.insert(position, TextToken::Literal(symbol));
            }
            ParsePart::Date(DatePart {
                color,
                condition,
                use_ampm,
                tokens,
                names: localized.map(|l| l.names),
                day_half: localized.and_then(|l| l.day_half),
                calendar: localized.map(|l| l.calendar),
                japanese_lcid: localized.is_some_and(|l| l.japanese_lcid),
                year_twice: localized.is_some_and(|l| l.year_twice),
                hijri,
                b2_overridden,
                names_after_mmm: localized.and_then(|l| l.names_after_mmm),
                numerals,
            })
        } else {
            ParsePart::Number(NumberPart {
                color,
                condition,
                use_thousands,
                // A scientific code never scales: its `%` is shown as written and a trailing `,` is
                // dropped (`0.00E+00%` on 2.675 is "2.68E+00%", `0.00%E+00` "2.68%E+00", `0.00E+00,` on
                // 2675 "2.68E+03"; probes/t2720-scientific-percent, excel-graph 2026-10-25, T-2720).
                percent: if is_scientific { 0 } else { percent },
                comma: if is_scientific { 0 } else { comma },
                tokens,
                digit_count,
                precision,
                is_scientific,
                scientific_minus,
                exponent_digit_count,
                currency,
                fraction,
                numerals,
            })
        }
    }

    /// The section of a system tag (`[$-F800]`, `[$-F400]`), which replaces whatever else the
    /// section holds (`TEXT(45000.75,"x[$-F800]0")` is "Wednesday, March 15, 2023"). Only a first
    /// section shows it: in a later one the section prints nothing (`TEXT(-1,"0;[$-F800]")` and
    /// `TEXT(0,"0;0;[$-F800]")` are ""; probes/text-locale-name-tables A557-A570, T-1716).
    fn system_part(&self, format: &str, color: Option<i32>) -> ParsePart {
        if !self.parts.is_empty() {
            return ParsePart::Number(NumberPart {
                color,
                condition: None,
                use_thousands: false,
                percent: 0,
                comma: 0,
                tokens: vec![],
                digit_count: 0,
                precision: 0,
                is_scientific: false,
                scientific_minus: false,
                exponent_digit_count: 0,
                currency: None,
                fraction: None,
                numerals: Numerals::Ascii,
            });
        }
        let mut parser = Parser::new(format);
        parser.parse();
        match parser.parts.pop() {
            Some(ParsePart::Date(part)) => ParsePart::Date(DatePart {
                color,
                names: Some(EN_US),
                day_half: Some(EN_US),
                calendar: Some(Calendar::Gregorian),
                japanese_lcid: false,
                year_twice: false,
                ..part
            }),
            _ => ParsePart::Error(ErrorPart::default()),
        }
    }
}
