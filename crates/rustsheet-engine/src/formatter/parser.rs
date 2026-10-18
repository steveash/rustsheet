use super::lexer::{Compare, Lexer, Token};
use super::locale_tag::{self, DateNames, LocaleTag};
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
    AMPM,
    AP(char, char),
}
pub struct NumberPart {
    pub color: Option<i32>,
    pub condition: Option<(Compare, f64)>,
    pub use_thousands: bool,
    pub percent: i32, // multiply number by 100^percent
    pub comma: i32,   // divide number by 1000^comma
    pub tokens: Vec<TextToken>,
    pub digit_count: i32, // number of digit tokens (#, 0 or ?) to the left of the decimal point
    pub precision: i32,   // number of digits to the right of the decimal point
    pub is_scientific: bool,
    pub scientific_minus: bool,
    pub exponent_digit_count: i32,
    pub currency: Option<char>,
}

pub struct DatePart {
    pub color: Option<i32>,
    pub use_ampm: bool,
    pub tokens: Vec<TextToken>,
    /// The names of the section's `[$-...]` locale; `None` uses the workbook locale.
    pub names: Option<&'static DateNames>,
}

pub struct ErrorPart {}

pub struct GeneralPart {}

pub enum ParsePart {
    Number(NumberPart),
    Date(DatePart),
    Error(ErrorPart),
    General(GeneralPart),
}

pub struct Parser {
    pub parts: Vec<ParsePart>,
    lexer: Lexer,
}

#[derive(PartialEq, Copy, Clone)]
pub enum NumberState {
    Integer,
    Decimal,
    Exponent,
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
        Parser { parts, lexer }
    }
    pub fn parse(&mut self) {
        while self.lexer.peek_token() != Token::EOF {
            let part = self.parse_part();
            self.parts.push(part);
        }
    }

    fn parse_part(&mut self) -> ParsePart {
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
        let mut hijri = false;
        // Whether the previous token is an unescaped colon, which the next one joins.
        let mut after_colon = false;
        let mut locale = None;
        // Excel: a second locale tag in a section is #VALUE!, even the same one
        // (`TEXT(45000,"[$-40C]mmmm [$-40C]mmmm")`; probes/text-locale-name-tables A568-A575).
        let mut repeated_locale = false;

        while token != Token::EOF && token != Token::Separator {
            let next_token = self.lexer.next_token();
            let token_is_digit = token.is_digit();
            is_number = is_number || token_is_digit;
            let next_token_is_digit = next_token.is_digit();
            let is_colon = token == Token::Literal(':');
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
                        return ParsePart::General(GeneralPart {});
                    }
                    // Excel: literals around `General` print around the value (`TEXT(5,"General x")`
                    // = "5 x", `TEXT(5,"x General")` = "x 5"; probes/text-format-bare-literals
                    // A75, A76). TODO(T-1614): next to placeholders or date tokens it stays an error (unprobed).
                    tokens.push(TextToken::General);
                }
                Token::Comma => {
                    // If it is in between digit tokens then we use the thousand separator
                    if last_token_is_digit && next_token_is_digit {
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
                Token::Literal(value) => {
                    if value == ':' {
                        is_time = true;
                    }
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
                Token::Month => {
                    if is_time {
                        // minute
                        tokens.push(TextToken::Minute);
                    } else {
                        is_date = true;
                        tokens.push(TextToken::Month);
                    }
                }
                Token::MonthPadded => {
                    if is_time {
                        // minute padded
                        tokens.push(TextToken::MinutePadded);
                    } else {
                        is_date = true;
                        tokens.push(TextToken::MonthPadded);
                    }
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
                // The era prints nothing for the Gregorian calendar and `B1` selects it; both
                // still make the section a date one (`TEXT(100,"g")` and `TEXT(100,"B1")` are "").
                Token::Era | Token::Calendar('1') => {
                    is_date = true;
                }
                // TODO(T-1613): `B2` (Hijri) is not implemented: the section is an error once a date or time
                // token would print a Hijri date (`TEXT(100,"B2")` alone is "").
                Token::Calendar(_) => {
                    is_date = true;
                    hijri = true;
                }
                Token::Hour => {
                    is_date = true;
                    is_time = true;
                    tokens.push(TextToken::Hour);
                }
                Token::HourPadded => {
                    is_date = true;
                    is_time = true;
                    tokens.push(TextToken::HourPadded);
                }
                Token::Second => {
                    is_date = true;
                    is_time = true;
                    tokens.push(TextToken::Second);
                }
                Token::SecondPadded => {
                    is_date = true;
                    is_time = true;
                    tokens.push(TextToken::SecondPadded);
                }
                Token::AMPM => {
                    is_date = true;
                    use_ampm = true;
                    tokens.push(TextToken::AMPM);
                }
                Token::AP(am, pm) => {
                    is_date = true;
                    use_ampm = true;
                    tokens.push(TextToken::AP(am, pm));
                }
                Token::Locale(tag) => {
                    repeated_locale |= locale.is_some();
                    locale = Some(tag);
                }
                Token::Scientific => {
                    if !is_scientific {
                        index = 0;
                        number = NumberState::Exponent;
                    }
                    is_scientific = true;
                }
                Token::ScientificMinus => {
                    if !is_scientific {
                        index = 0;
                        number = NumberState::Exponent;
                    }
                    is_scientific = true;
                    scientific_minus = true;
                }
                Token::Separator => {}
                Token::Raw => {
                    tokens.push(TextToken::Raw);
                }
                Token::ILLEGAL => {
                    return ParsePart::Error(ErrorPart {});
                }
                Token::EOF => {}
                Token::ElapsedHour => {
                    is_date = true;
                    is_time = true;
                    tokens.push(TextToken::ElapsedHour);
                }
                Token::ElapsedMinute => {
                    is_date = true;
                    is_time = true;
                    tokens.push(TextToken::ElapsedMinute);
                }
                Token::ElapsedSecond => {
                    is_date = true;
                    is_time = true;
                    tokens.push(TextToken::ElapsedSecond);
                }
                Token::ElapsedHourPadded => {
                    is_date = true;
                    is_time = true;
                    tokens.push(TextToken::ElapsedHourPadded);
                }
                Token::ElapsedMinutePadded => {
                    is_date = true;
                    is_time = true;
                    tokens.push(TextToken::ElapsedMinutePadded);
                }
                Token::ElapsedSecondPadded => {
                    is_date = true;
                    is_time = true;
                    tokens.push(TextToken::ElapsedSecondPadded);
                }
            }
            last_token_is_digit = token_is_digit;
            after_colon = is_colon;
            token = next_token;
        }
        if repeated_locale {
            return ParsePart::Error(ErrorPart {});
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
        let names = match locale {
            Some(LocaleTag::Names(names)) => Some(names),
            _ => None,
        };
        let general_count = tokens
            .iter()
            .filter(|t| matches!(t, TextToken::General))
            .count();
        if general_count > 0 {
            let only_literals = tokens.iter().all(|t| {
                matches!(
                    t,
                    TextToken::General
                        | TextToken::Literal(_)
                        | TextToken::Text(_)
                        | TextToken::Ghost(_)
                        | TextToken::Spacer(_)
                )
            });
            if general_count > 1 || !only_literals || is_date || percent > 0 || comma > 0 {
                return ParsePart::Error(ErrorPart {});
            }
        }
        if is_date {
            let prints_date = tokens.iter().any(|t| {
                !matches!(
                    t,
                    TextToken::Literal(_)
                        | TextToken::Text(_)
                        | TextToken::Ghost(_)
                        | TextToken::Spacer(_)
                )
            });
            if is_number || (hijri && prints_date) {
                return ParsePart::Error(ErrorPart {});
            }
            // Excel prints a currency symbol in a date section where it is written
            // (`[$€-40C]d mmmm` is "€15 mars", `mmmm[$€-40C]` "mars€"; number-formats.md).
            for (position, symbol) in currency_positions.into_iter().rev() {
                tokens.insert(position, TextToken::Literal(symbol));
            }
            ParsePart::Date(DatePart {
                color,
                use_ampm,
                tokens,
                names,
            })
        } else {
            ParsePart::Number(NumberPart {
                color,
                condition,
                use_thousands,
                percent,
                comma,
                tokens,
                digit_count,
                precision,
                is_scientific,
                scientific_minus,
                exponent_digit_count,
                currency,
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
            });
        }
        let mut parser = Parser::new(format);
        parser.parse();
        match parser.parts.pop() {
            Some(ParsePart::Date(part)) => ParsePart::Date(DatePart {
                color,
                names: Some(EN_US),
                ..part
            }),
            _ => ParsePart::Error(ErrorPart {}),
        }
    }
}
