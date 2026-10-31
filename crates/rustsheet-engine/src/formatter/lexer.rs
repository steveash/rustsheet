use super::locale_tag::{self, LocaleTag};
use crate::number_format::{parse_excel_number, parse_finite_number};

pub struct Lexer {
    position: usize,
    len: usize,
    chars: Vec<char>,
    error_message: String,
    error_position: usize,
    /// The locale of a `[$€-40C]` currency tag, returned by the call after its symbol.
    pending_locale: Option<LocaleTag>,
    /// Whether the current section has a token before this one other than a color, condition,
    /// locale or calendar prefix: a `B1`/`B2` calendar prefix is invalid after one.
    section_started: bool,
    /// Whether the current section has had a `B2` prefix: a locale tag after it resolves with
    /// the Hijri calendar (`locale_tag::resolve_after_hijri_prefix`).
    hijri_prefix: bool,
}

#[derive(PartialEq, Debug)]
pub enum Token {
    Color(i32),              // [Red] or [Color 23]
    Condition(Compare, f64), // [<=100] (Comparator, number)
    Currency(char),          // [$€] ($ currency symbol)
    Literal(char), // €, £, ¥, |, $, (, ), /, :, +, -, ^, ', {, }, <, =, !, ~, >, space, digits 1-9 and literal letters
    Escaped(char), // \X
    Spacer(char),  // *X
    Ghost(char),   // _X
    Text(String),  // "Text"
    Separator,     // ;
    Raw,           // @
    Percent,       // %
    Comma,         // ,
    Period,        // .
    Sharp,         // #
    Zero,          // 0
    QuestionMark,  // ?
    Scientific,    // E+
    ScientificMinus, // E-
    General,       // General
    // Dates and time
    Day,            // d
    DayPadded,      // dd
    DayNameShort,   // ddd
    DayName,        // dddd+
    Month,          // m (or minute)
    MonthPadded,    // mm (or minute padded)
    MonthNameShort, // mmm
    MonthName,      // mmmm or mmmmmm+
    MonthLetter,    // mmmmm
    YearShort,      // y or yy
    Year,           // yyy+
    /// `e` (false) or `ee`+ (true, two digits at least): the era year, the four-digit year in a
    /// Gregorian calendar.
    EraYear(bool),
    BuddhistYearShort,   // b or bb
    BuddhistYear,        // bbb+
    Era(usize),          // g+ with its length (empty in en-US)
    Calendar(char),      // B1 (Gregorian) or B2 (Hijri)
    Hour,                // h
    HourPadded,          // hh
    Second,              // s
    SecondPadded,        // ss
    ElapsedHour,         // [h]
    ElapsedMinute,       // [m]
    ElapsedSecond,       // [s]
    ElapsedHourPadded,   // [hh]
    ElapsedMinutePadded, // [mm]
    ElapsedSecondPadded, // [ss]
    AMPM,                // AM/PM
    /// `A/P`: the letters printed for the morning and for the afternoon.
    AP(char, char),
    /// `[$-409]`: a locale with no currency symbol, which prints nothing.
    Locale(LocaleTag),
    ILLEGAL,
    EOF,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Compare {
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessOrEqualThan,
    GreaterOrEqualThan,
}

impl Compare {
    /// Whether `value` meets the condition `[<self><bound>]`.
    #[must_use]
    pub fn holds(self, value: f64, bound: f64) -> bool {
        match self {
            Compare::Equal => value == bound,
            Compare::NotEqual => value != bound,
            Compare::LessThan => value < bound,
            Compare::GreaterThan => value > bound,
            Compare::LessOrEqualThan => value <= bound,
            Compare::GreaterOrEqualThan => value >= bound,
        }
    }
}

impl Token {
    pub fn is_digit(&self) -> bool {
        (self == &Token::Zero) || (self == &Token::Sharp) || (self == &Token::QuestionMark)
    }

    pub fn is_date(&self) -> bool {
        self == &Token::Day
            || self == &Token::DayPadded
            || self == &Token::DayNameShort
            || self == &Token::DayName
            || self == &Token::MonthName
            || self == &Token::MonthNameShort
            || self == &Token::Month
            || self == &Token::MonthPadded
            || self == &Token::MonthLetter
            || self == &Token::YearShort
            || self == &Token::Year
            || matches!(self, Token::EraYear(_))
            || self == &Token::BuddhistYearShort
            || self == &Token::BuddhistYear
    }
}

/// Whether the last section of `format` is exactly `@`, with no color, locale or literal, not even
/// `""`. Excel shows a text whole under it however long, where any other text section shows 255 `#`
/// past 255 characters (probes/displayed-text-long-text-b A2, A3 vs A1, A4, A6; T-3282).
pub(crate) fn last_section_is_bare_at(format: &str) -> bool {
    let mut lexer = Lexer::new(format);
    let mut start = 0;
    loop {
        match lexer.next_token() {
            Token::EOF => break,
            Token::Separator => start = lexer.position,
            _ => {}
        }
    }
    lexer.chars.get(start..) == Some(&['@'][..])
}

impl Lexer {
    pub fn new(format: &str) -> Lexer {
        let chars: Vec<char> = format.chars().collect();
        let len = chars.len();
        Lexer {
            chars,
            position: 0,
            len,
            error_message: "".to_string(),
            error_position: 0,
            pending_locale: None,
            section_started: false,
            hijri_prefix: false,
        }
    }

    fn peek_char(&self) -> Option<char> {
        let position = self.position;
        if position < self.len {
            Some(self.chars[position])
        } else {
            None
        }
    }

    fn read_next_char(&mut self) -> Option<char> {
        let position = self.position;
        if position < self.len {
            self.position = position + 1;
            Some(self.chars[position])
        } else {
            None
        }
    }

    fn set_error(&mut self, error: &str) {
        self.error_message = error.to_string();
        self.error_position = self.position;
        self.position = self.len;
    }

    fn consume_string(&mut self) -> Option<String> {
        let mut position = self.position;
        let len = self.len;
        let mut chars = "".to_string();
        while position < len {
            let x = self.chars[position];
            position += 1;
            // Excel: `""` inside a literal is not an escaped quote but the end of one literal and
            // the start of the next, so `0 """Millions"""` shows `42 Millions`
            // (probes/format-doubled-quotes C1-C11; docs/excel-semantics/number-formats.md).
            if x != '"' {
                chars.push(x);
            } else {
                self.position = position;
                return Some(chars);
            }
        }
        None
    }

    fn consume_number(&mut self) -> Option<f64> {
        let mut position = self.position;
        let len = self.len;
        let mut chars = "".to_string();
        // first optional '-' sign
        if position < len && self.chars[position] == '-' {
            chars.push('-');
            position += 1;
        }
        // numbers before the '.'
        while position < len {
            let x = self.chars[position];
            if x.is_ascii_digit() {
                chars.push(x);
            } else {
                break;
            }
            position += 1;
        }
        if position < len && self.chars[position] == '.' {
            // numbers after the'.'
            chars.push('.');
            position += 1;
            while position < len {
                let x = self.chars[position];
                if x.is_ascii_digit() {
                    chars.push(x);
                } else {
                    break;
                }
                position += 1;
            }
        }
        if position + 1 < len && self.chars[position].eq_ignore_ascii_case(&'e') {
            // exponential side
            let x = self.chars[position + 1];
            if x == '-' || x == '+' || x.is_ascii_digit() {
                chars.push('e');
                chars.push(x);
                position += 2;
                while position < len {
                    let x = self.chars[position];
                    if x.is_ascii_digit() {
                        chars.push(x);
                    } else {
                        break;
                    }
                    position += 1;
                }
            }
        }
        self.position = position;
        // A condition number has to be one Excel reads from text at all: `[>1e-310]` and
        // `[>0e309]` make the format invalid like `[>1e400]` (probes/text-number-tiny-paths
        // C18, C19), but the value keeps every digit.
        parse_finite_number(&chars).filter(|_| parse_excel_number(&chars).is_some())
    }

    fn consume_condition(&mut self) -> Option<(Compare, f64)> {
        let cmp;
        match self.read_next_char() {
            Some('<') => match self.peek_char() {
                Some('=') => {
                    self.read_next_char();
                    cmp = Compare::LessOrEqualThan;
                }
                // Excel: `[<>10]` is "not equal" (poi FormatChoiceTests A17-A19).
                Some('>') => {
                    self.read_next_char();
                    cmp = Compare::NotEqual;
                }
                _ => cmp = Compare::LessThan,
            },
            Some('>') => {
                if let Some('=') = self.peek_char() {
                    self.read_next_char();
                    cmp = Compare::GreaterOrEqualThan;
                } else {
                    cmp = Compare::GreaterThan;
                }
            }
            Some('=') => {
                cmp = Compare::Equal;
            }
            _ => {
                return None;
            }
        }

        // Excel: spaces between the comparator and the number are allowed (`[< 10]`,
        // `[<      10]`; poi FormatChoiceTests A23-A26).
        while self.peek_char() == Some(' ') {
            self.read_next_char();
        }
        let start = self.position;
        let value = match self.consume_number() {
            Some(v) => v,
            // Without a number, as in `[>]`, the condition compares with 0
            None if self.position == start => 0.0,
            // Something that should be a number and is not one (`[>1e400]`, too large to hold)
            // is no condition: TEXT(5,"[>1e400]0") is #VALUE! (probes/text-number-words J1)
            None => return None,
        };
        if self.read_next_char() == Some(']') {
            return Some((cmp, value));
        }
        None
    }

    // Consume elapsed time like [h], [hh], [m], [mm], [s], [ss]
    fn consume_elapsed_time(&mut self) -> Option<Token> {
        // We are called right after '['
        let start_pos = self.position;

        // First char must be one of: h, m, s
        let unit = match self.read_next_char() {
            Some(c) if ['h', 'H', 'm', 'M', 's', 'S'].contains(&c) => c.to_ascii_lowercase(),
            _ => {
                // Not elapsed time → rewind
                self.position = start_pos;
                return None;
            }
        };

        // Count how many times the same unit repeats (so we detect h vs hh, etc.)
        let mut count = 1;
        while let Some(ch) = self.peek_char() {
            if ch.to_ascii_lowercase() == unit {
                count += 1;
                self.read_next_char();
            } else {
                break;
            }
        }

        // Must be closed by ']'
        if self.read_next_char() != Some(']') {
            // Not a well-formed [h], [hh], etc. → rewind
            self.position = start_pos;
            return None;
        }

        // Map (unit, count) to a token
        let token = match (unit, count) {
            ('h', 1) => Token::ElapsedHour,
            ('h', 2) => Token::ElapsedHourPadded,
            ('m', 1) => Token::ElapsedMinute,
            ('m', 2) => Token::ElapsedMinutePadded,
            ('s', 1) => Token::ElapsedSecond,
            ('s', 2) => Token::ElapsedSecondPadded,
            // Anything else like [hhh], [mmm], [sss] → not considered elapsed time
            _ => {
                self.position = start_pos;
                return None;
            }
        };

        Some(token)
    }

    fn consume_color(&mut self) -> Option<i32> {
        let colors = [
            "black", "white", "red", "green", "blue", "yellow", "magenta", "cyan",
        ];
        let mut chars = "".to_string();
        while let Some(ch) = self.read_next_char() {
            if ch == ']' {
                let lc = chars.to_lowercase();
                if let Some(index) = colors.iter().position(|&x| x == lc) {
                    return Some(index as i32);
                }
                if !chars.starts_with("Color") {
                    return None;
                }
                if let Ok(index) = chars[5..].trim().parse::<i32>() {
                    if index < 57 && index > 0 {
                        return Some(index);
                    } else {
                        return None;
                    }
                }
                return None;
            } else {
                chars.push(ch);
            }
        }
        None
    }

    /// Consumes the letters equal (ignoring case) to `letter` that follow one already read and
    /// returns how many there are in all.
    fn consume_run(&mut self, letter: char) -> usize {
        let mut count = 1;
        while let Some(c) = self.peek_char() {
            if c.eq_ignore_ascii_case(&letter) {
                count += 1;
                self.read_next_char();
            } else {
                break;
            }
        }
        count
    }

    /// How many characters from the current position are exactly `c`, without consuming them.
    /// Unlike [`Self::consume_run`] it is case-sensitive: `aAa` is not a weekday run (T-1880).
    fn same_char_run(&self, c: char) -> usize {
        self.chars
            .get(self.position..)
            .map_or(0, |rest| rest.iter().take_while(|&&next| next == c).count())
    }

    /// Whether the characters from the current position start with `text`, ignoring ASCII case.
    fn looks_at(&self, text: &str) -> bool {
        let mut position = self.position;
        for expected in text.chars() {
            match self.chars.get(position) {
                Some(c) if c.eq_ignore_ascii_case(&expected) => position += 1,
                _ => return false,
            }
        }
        true
    }

    /// Whether the next token is a currency tag's symbol: `[$€]` or `[$€-40C]` (`Token::Currency`),
    /// or `[$USD]` (`Token::Text`), not a `[$-409]` locale tag. The complexity limit prices it
    /// apart from quoted text (formatter/complexity.rs, T-2812).
    pub(crate) fn at_currency_tag(&self) -> bool {
        self.pending_locale.is_none()
            && self.chars.get(self.position) == Some(&'[')
            && self.chars.get(self.position + 1) == Some(&'$')
            && !matches!(self.chars.get(self.position + 2), Some('-' | ']') | None)
    }

    /// Whether the next token is the locale of the currency tag just read (`[$€-40C]`).
    pub(crate) fn has_pending_locale(&self) -> bool {
        self.pending_locale.is_some()
    }

    pub fn peek_token(&mut self) -> Token {
        self.peek_token_past(|_| false)
    }

    /// The first token after the run of tokens `skip` accepts, without consuming anything.
    pub fn peek_token_past(&mut self, skip: fn(&Token) -> bool) -> Token {
        let (position, pending) = (self.position, self.pending_locale);
        let mut token = self.next_token();
        while skip(&token) && token != Token::EOF {
            token = self.next_token();
        }
        (self.position, self.pending_locale) = (position, pending);
        token
    }

    pub fn next_token(&mut self) -> Token {
        let token = self.read_token();
        self.section_started = match token {
            Token::Separator => false,
            Token::Color(_) | Token::Condition(..) | Token::Locale(_) | Token::Calendar(_) => {
                self.section_started
            }
            _ => true,
        };
        match token {
            Token::Separator => self.hijri_prefix = false,
            Token::Calendar('2') => self.hijri_prefix = true,
            _ => {}
        }
        token
    }

    /// Resolves the locale tag `tag` of the current section.
    fn resolve_tag(&self, tag: &str) -> LocaleTag {
        if self.hijri_prefix {
            locale_tag::resolve_after_hijri_prefix(tag)
        } else {
            locale_tag::resolve(tag)
        }
    }

    fn read_token(&mut self) -> Token {
        if let Some(locale) = self.pending_locale.take() {
            return Token::Locale(locale);
        }
        let ch = self.read_next_char();
        match ch {
            Some(x) => match x {
                '$' | '€' | '£' | '¥' | '|' | '(' | ')' | '/' | ':' | '+' | '-' | '^' | '\''
                | '{' | '}' | '<' | '=' | '!' | '~' | '>' | ' ' => Token::Literal(x),
                '?' => Token::QuestionMark,
                ';' => Token::Separator,
                '#' => Token::Sharp,
                ',' => Token::Comma,
                '.' => Token::Period,
                '0' => Token::Zero,
                '@' => Token::Raw,
                '%' => Token::Percent,
                '[' => {
                    if let Some(c) = self.peek_char() {
                        if c == '<' || c == '>' || c == '=' {
                            // Condition
                            if let Some((cmp, value)) = self.consume_condition() {
                                Token::Condition(cmp, value)
                            } else {
                                self.set_error("Failed to parse condition");
                                Token::ILLEGAL
                            }
                        } else if c == '$' {
                            // currency: [$<symbol>] or [$<symbol>-<locale>]
                            // (`at_currency_tag` tells this branch's symbol tags apart; keep the two in step)
                            // IronCalc ignores the currency/locale modifiers for now
                            // They are of two types:
                            //  * BCP-47 locale tag (like [$€-fr-FR])
                            //  * hex LCID (old format)
                            //      Like [$$-409] (=1033 for en-US) or
                            //           [$$-40C] (=1036 for fr-FR)
                            self.read_next_char();
                            // Excel: `[$-409]` has a locale and no symbol and prints nothing
                            // (probes/text-ap-locale-bracket A11-A20; number-formats.md); the
                            // locale it names applies to date names (`locale_tag`, T-1716).
                            if self.peek_char() == Some('-') {
                                self.read_next_char();
                                let mut tag = String::new();
                                while let Some(c) = self.read_next_char() {
                                    if c == ']' {
                                        return Token::Locale(self.resolve_tag(&tag));
                                    }
                                    tag.push(c);
                                }
                                self.set_error("Failed to parse locale");
                                return Token::ILLEGAL;
                            }
                            if let Some(currency) = self.read_next_char() {
                                // Excel: currency text of several characters prints as it is
                                // (`#,##0.00 [$USD]` on -1.5 is "-1.50 USD", gen-m10 T-1611;
                                // number-formats.md).
                                let mut text = String::from(currency);
                                while let Some(c) = self.peek_char() {
                                    if c == ']' || c == '-' {
                                        break;
                                    }
                                    text.push(c);
                                    self.read_next_char();
                                }
                                // The locale part of `[$€-40C]` is read as a `[$-40C]` tag that
                                // follows the symbol (probes/text-currency-locale-date).
                                if Some('-') == self.peek_char() {
                                    self.read_next_char();
                                    let mut tag = String::new();
                                    while let Some(c) = self.peek_char() {
                                        if c == ']' {
                                            break;
                                        }
                                        tag.push(c);
                                        self.read_next_char();
                                    }
                                    self.pending_locale = Some(self.resolve_tag(&tag));
                                }
                                if self.read_next_char() == Some(']') {
                                    return if text.chars().count() == 1 {
                                        Token::Currency(currency)
                                    } else {
                                        Token::Text(text)
                                    };
                                }
                            }
                            self.set_error("Failed to parse currency");
                            Token::ILLEGAL
                        } else {
                            // try bracketed (elapsed) time
                            if let Some(token) = self.consume_elapsed_time() {
                                return token;
                            }
                            // Color
                            if let Some(index) = self.consume_color() {
                                return Token::Color(index);
                            }
                            self.set_error("Failed to parse color");
                            Token::ILLEGAL
                        }
                    } else {
                        self.set_error("Unexpected end of input");
                        Token::ILLEGAL
                    }
                }
                '_' => {
                    if let Some(y) = self.read_next_char() {
                        Token::Ghost(y)
                    } else {
                        self.set_error("Unexpected end of input");
                        Token::ILLEGAL
                    }
                }
                '*' => {
                    if let Some(y) = self.read_next_char() {
                        Token::Spacer(y)
                    } else {
                        self.set_error("Unexpected end of input");
                        Token::ILLEGAL
                    }
                }
                '\\' => {
                    if let Some(y) = self.read_next_char() {
                        Token::Escaped(y)
                    } else {
                        self.set_error("Unexpected end of input");
                        Token::ILLEGAL
                    }
                }
                '"' => {
                    if let Some(s) = self.consume_string() {
                        Token::Text(s)
                    } else {
                        self.set_error("Did not find end of text string");
                        Token::ILLEGAL
                    }
                }
                'E' => {
                    if let Some(s) = self.read_next_char() {
                        if s == '+' {
                            Token::Scientific
                        } else if s == '-' {
                            Token::ScientificMinus
                        } else {
                            self.set_error(&format!("Unexpected char: {s}. Expected + or -"));
                            Token::ILLEGAL
                        }
                    } else {
                        self.set_error("Unexpected end of input");
                        Token::ILLEGAL
                    }
                }
                'd' | 'D' => {
                    let d = self.consume_run('d');
                    match d {
                        1 => Token::Day,
                        2 => Token::DayPadded,
                        3 => Token::DayNameShort,
                        _ => Token::DayName,
                    }
                }
                'm' | 'M' => {
                    let m = self.consume_run('m');
                    match m {
                        1 => Token::Month,       // (or minute)
                        2 => Token::MonthPadded, // (or minute padded)
                        3 => Token::MonthNameShort,
                        4 => Token::MonthName,
                        5 => Token::MonthLetter,
                        _ => Token::MonthName,
                    }
                }
                'y' | 'Y' => {
                    let y = self.consume_run('y');
                    if y == 1 || y == 2 {
                        Token::YearShort
                    } else {
                        Token::Year
                    }
                }
                'h' | 'H' => {
                    let h = self.consume_run('h');
                    if h == 1 {
                        Token::Hour
                    } else if h == 2 {
                        Token::HourPadded
                    } else {
                        self.set_error("Unexpected character after 'h'");
                        Token::ILLEGAL
                    }
                }
                's' | 'S' => {
                    let s = self.consume_run('s');
                    if s == 1 {
                        Token::Second
                    } else if s == 2 {
                        Token::SecondPadded
                    } else {
                        self.set_error("Unexpected character after 's'");
                        Token::ILLEGAL
                    }
                }
                'A' | 'a' => {
                    if self.looks_at("m/pm") {
                        self.position += 4;
                        Token::AMPM
                    } else if self.looks_at("/p") {
                        // Excel: `A/P` prints the first letter in the morning and the second
                        // in the afternoon, each as typed (`h A/p` is "6 A" and "6 p";
                        // probes/text-ap-locale-bracket A1-A6, A28).
                        self.read_next_char();
                        let pm = self.read_next_char().unwrap_or('p');
                        Token::AP(x, pm)
                    } else if let first @ 3.. = 1 + self.same_char_run(x) {
                        // Excel: three or more of one `a` (all lowercase or all uppercase) are the
                        // weekday, `aaa` short and `aaaa`+ full like `ddd`/`dddd`; mixed case
                        // prints (`TEXT(1,"aaa")` = "Sun", `"AAAA"` "Sunday", `"aAa"` "aAa").
                        // Such runs of the other case right after it join it (`"aaaAAA"` is
                        // "Sunday"), shorter ones do not (`"aaaA"` is "SunA"; probes/
                        // text-format-aaa-weekday, -b, -c; T-1880).
                        self.position += first - 1;
                        let mut run = first;
                        while let Some(next @ ('a' | 'A')) = self.peek_char() {
                            match self.same_char_run(next) {
                                more @ 3.. => {
                                    self.position += more;
                                    run += more;
                                }
                                _ => break,
                            }
                        }
                        if run == 3 {
                            Token::DayNameShort
                        } else {
                            Token::DayName
                        }
                    } else {
                        // Excel: an unquoted `a` that is not AM/PM prints as itself
                        // (probes/text-format-unquoted-letters A1, A27).
                        Token::Literal(x)
                    }
                }
                // Excel: `b`/`bb` is the two-digit Buddhist-era year and `bbb`+ the four-digit one,
                // `B1`/`B2` a calendar prefix that prints nothing, and a lone `B` closing the code is
                // invalid (`TEXT(100,"ABC")` = "A43C", `"B"` and `"dB"` are #VALUE!;
                // probes/text-format-bare-literals A40-A50, probes/text-format-unquoted-letters).
                // The prefix is invalid after anything that prints in its section: `"b1"` is
                // accepted, `"ab1"`, `"*xb1"` and `"a*?b1"` are #VALUE! (probes/
                // t2202-text-code-check-on-text; number-formats.md "TEXT checks the code before it
                // looks at a text or boolean value").
                'b' | 'B' => {
                    if let Some(c @ ('1' | '2')) = self.peek_char() {
                        self.read_next_char();
                        if self.section_started {
                            self.set_error("Calendar prefix after the start of a section");
                            return Token::ILLEGAL;
                        }
                        return Token::Calendar(c);
                    }
                    let count = self.consume_run('b');
                    if x == 'B' && count == 1 && matches!(self.peek_char(), None | Some(';')) {
                        self.set_error("Unexpected end of input after 'B'");
                        return Token::ILLEGAL;
                    }
                    if count <= 2 {
                        Token::BuddhistYearShort
                    } else {
                        Token::BuddhistYear
                    }
                }
                // Excel: a lowercase `e` (any run) is the era year, which is the four-digit year in
                // the Gregorian calendar (`TEXT(100,"e")` = "1900", `"e+"` = "1900+";
                // probes/text-format-bare-literals A51, A52, A63); under Japanese eras `e` is "5" and
                // `ee` "05" (probes/text-locale-calendars A13).
                'e' => Token::EraYear(self.consume_run('e') > 1),
                'g' | 'G' => {
                    // Excel: the word is case-insensitive (`TEXT(5,"GENERAL x")` = "5 x";
                    // probes/text-format-general-company A16).
                    if self
                        .chars
                        .get(self.position..self.position + 6)
                        .is_some_and(|rest| {
                            rest.iter()
                                .collect::<String>()
                                .eq_ignore_ascii_case("eneral")
                        })
                    {
                        self.position += 6;
                        return Token::General;
                    }
                    // Excel: any other run of `g` is the era, empty for the Gregorian calendar
                    // (`TEXT(100,"g")` = "", `TEXT(1,"dG")` = "1"; probes/text-format-bare-literals
                    // A53-A56, probes/text-format-unquoted-letters).
                    Token::Era(self.consume_run('g'))
                }
                // Excel: `n`/`N` are invalid (cause unknown, not an en-US date token;
                // probes/text-format-unquoted-letters A14, A40).
                'n' | 'N' => {
                    self.set_error("Unexpected character: n");
                    Token::ILLEGAL
                }
                // Excel prints every other letter (the format-token letters have their own arms
                // above) and the digits 1-9 as they are: `TEXT(1,"0x")` = "1x", `TEXT(PI(),"ü")` =
                // "ü", `TEXT(5,"2")` = "2" (probes/text-format-unquoted-letters,
                // probes/text-format-bare-literals A1-A7).
                // Control characters print as themselves too: `TEXT(100,CHAR(10))` is a line feed
                // and `TEXT(-2.5,CHAR(1))` is "-" and CHAR(1) (gen-m5/d586a1e0c9896942 Calc!A38,
                // gen-m5/c0651f0e0fb544d0 Calc!A336; number-formats.md, T-2158).
                _ if x.is_alphabetic() || ('1'..='9').contains(&x) || x.is_control() => {
                    Token::Literal(x)
                }
                _ => {
                    self.set_error(&format!("Unexpected character: {x}"));
                    Token::ILLEGAL
                }
            },
            None => Token::EOF,
        }
    }
}

pub fn is_likely_date_number_format(format: &str) -> bool {
    let mut lexer = Lexer::new(format);
    loop {
        let token = lexer.next_token();
        if token == Token::EOF {
            return false;
        }
        if token.is_date() {
            return true;
        }
    }
}
