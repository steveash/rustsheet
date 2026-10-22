//! The PCRE2 pattern grammar (SPEC-0013 section 1): hand-written, iterative over groups (an
//! explicit frame stack), and failing closed: every construct it does not know is
//! [`Invalid`]. Brackets nest and recurse, but under the same [`NEST_LIMIT`] as groups.

use regex_syntax::hir::{ClassUnicode, ClassUnicodeRange};

use super::{
    ast::{fixed_length, Anchor, Ast, Condition, LookKind, Reference, RepeatKind},
    class::{class_of, is_extended_space, posix_class_text, BLANK},
};

/// The deepest a pattern may nest: groups, brackets and stacked quantifiers each count one
/// level. PCRE2's default parentheses nest limit; T-1972 probes Excel's.
pub(super) const NEST_LIMIT: u32 = 250;

/// PCRE2's largest repeat count (`a{65536}` is a compile error).
const MAX_REPEAT: u32 = 65_535;

/// Compile-budget units per code point a case fold walks: regex-syntax's simple folding takes
/// about 5 ns a code point in a release build (T-2138), so a call's 128 MiB compile budget
/// folds at most 16 M code points, under 0.1 s, before the parser stops folding.
const FOLD_UNITS: usize = 8;

/// PCRE2's longest group name.
const MAX_NAME: usize = 32;

/// A pattern PCRE2 rejects, or one using a construct we do not know (`#VALUE!`).
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Invalid;

/// A parsed pattern.
pub(super) struct Parsed {
    pub(super) ast: Ast,
    /// Code points the case folding of its classes walks (the compile cost of `(?i)`, T-2138).
    pub(super) fold_cost: usize,
}

/// The options in force at one point of the pattern.
#[derive(Debug, Clone, Copy, Default)]
struct Flags {
    /// `i`
    caseless: bool,
    /// `m`
    multi_line: bool,
    /// `s`
    dot_all: bool,
    /// `x`
    extended: bool,
    /// `n`
    no_auto_capture: bool,
    /// `U`
    ungreedy: bool,
}

/// What an open group becomes when it closes.
enum GroupKind {
    Root,
    Capture { index: u32, name: Option<Box<str>> },
    NonCapture,
    LookAround(LookKind),
    Atomic,
    Conditional(Condition),
}

/// One open group: its alternatives so far, the items of the current one (each with its depth)
/// and the flags to restore when it closes.
struct Frame {
    kind: GroupKind,
    alternatives: Vec<(Ast, u32)>,
    items: Vec<(Ast, u32)>,
    outer: Flags,
}

/// One escape sequence, read outside or inside a bracket.
enum Escape {
    Char(char),
    Class(ClassUnicode),
    Node(Ast),
    /// `\Q`: literal text up to `\E`.
    Quote,
    /// A lone `\E`: nothing.
    Nothing,
}

/// Parses `pattern`; `caseless` is the `case_sensitivity` argument's choice, which `(?i)` and
/// `(?-i)` override. Case folding stops once it would walk more than `fold_budget` code points:
/// `fold_cost` is then `usize::MAX`, and the classes are left unfolded (the caller refuses the
/// pattern), so a pattern of many folded ranges costs no more than its budget (directive 5).
pub(super) fn parse(pattern: &str, caseless: bool, fold_budget: usize) -> Result<Parsed, Invalid> {
    Parser {
        chars: pattern.chars().collect(),
        pos: 0,
        flags: Flags {
            caseless,
            ..Flags::default()
        },
        groups: 0,
        names: Vec::new(),
        fold_cost: 0,
        fold_budget,
        quoting: false,
    }
    .parse()
}

struct Parser {
    chars: Vec<char>,
    pos: usize,
    flags: Flags,
    groups: u32,
    names: Vec<Box<str>>,
    fold_cost: usize,
    fold_budget: usize,
    /// Inside `\Q..\E`.
    quoting: bool,
}

impl Parser {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_at(&self, offset: usize) -> Option<char> {
        self.chars.get(self.pos + offset).copied()
    }

    fn next(&mut self) -> Result<char, Invalid> {
        let c = self.peek().ok_or(Invalid)?;
        self.pos += 1;
        Ok(c)
    }

    fn eat(&mut self, c: char) -> bool {
        let found = self.peek() == Some(c);
        if found {
            self.pos += 1;
        }
        found
    }

    fn expect(&mut self, c: char) -> Result<(), Invalid> {
        if self.eat(c) {
            Ok(())
        } else {
            Err(Invalid)
        }
    }

    fn parse(mut self) -> Result<Parsed, Invalid> {
        // PCRE2 reads start-of-pattern verbs; `(*UCP)` is a no-op, UCP being always on in Excel
        // (probes/regex-ascii-classes A17). Any other verb is unprobed, so invalid.
        while self.chars[self.pos..].starts_with(&['(', '*', 'U', 'C', 'P', ')']) {
            self.pos += 6;
        }
        let mut stack = vec![Frame {
            kind: GroupKind::Root,
            alternatives: Vec::new(),
            items: Vec::new(),
            outer: self.flags,
        }];
        while let Some(c) = self.peek() {
            let frame = stack.last_mut().ok_or(Invalid)?;
            if self.quoting {
                self.pos += 1;
                if c == '\\' && self.eat('E') {
                    self.quoting = false;
                } else {
                    frame.items.push((self.literal(c), 0));
                }
                continue;
            }
            if self.flags.extended && is_extended_space(c) {
                self.pos += 1;
                continue;
            }
            if self.flags.extended && c == '#' {
                while self.peek().is_some_and(|c| c != '\n') {
                    self.pos += 1;
                }
                continue;
            }
            self.pos += 1;
            match c {
                '\\' => match self.escape(false)? {
                    Escape::Char(c) => frame.items.push((self.literal(c), 0)),
                    Escape::Class(class) => frame.items.push((Ast::Class(class), 0)),
                    Escape::Node(ast) => frame.items.push((ast, 0)),
                    Escape::Quote => self.quoting = true,
                    Escape::Nothing => {}
                },
                '(' => {
                    if let Some(frame) = self.open_group()? {
                        if stack.len() > NEST_LIMIT as usize {
                            return Err(Invalid);
                        }
                        stack.push(frame);
                    }
                }
                ')' => {
                    let frame = stack.pop().ok_or(Invalid)?;
                    if matches!(frame.kind, GroupKind::Root) {
                        return Err(Invalid);
                    }
                    self.flags = frame.outer;
                    let (ast, depth) = close(frame)?;
                    stack.last_mut().ok_or(Invalid)?.items.push((ast, depth));
                }
                '|' => {
                    let items = std::mem::take(&mut frame.items);
                    frame.alternatives.push(concat(items));
                }
                '[' => {
                    // A POSIX class, collating element or equivalence class outside a bracket
                    // (`[:alpha:]`, `[.a.]`, `[..]`) is a PCRE2 error (probes/regex-posix-classes
                    // Z1, regex-posix-syntax-edges A5-A7; T-1985, T-2010).
                    if self.posix_class()?.is_some() {
                        return Err(Invalid);
                    }
                    let class = self.bracket(1)?;
                    stack
                        .last_mut()
                        .ok_or(Invalid)?
                        .items
                        .push((Ast::Class(class), 0));
                }
                '.' => frame.items.push((
                    Ast::Dot {
                        dot_all: self.flags.dot_all,
                    },
                    0,
                )),
                '^' => frame.items.push((
                    Ast::Anchor(if self.flags.multi_line {
                        Anchor::StartLine
                    } else {
                        Anchor::Start
                    }),
                    0,
                )),
                '$' => frame.items.push((
                    Ast::Anchor(if self.flags.multi_line {
                        Anchor::EndLine
                    } else {
                        Anchor::EndOrFinalLf
                    }),
                    0,
                )),
                '*' => self.quantify(frame, 0, None)?,
                '+' => self.quantify(frame, 1, None)?,
                '?' => self.quantify(frame, 0, Some(1))?,
                '{' => match self.counted()? {
                    Some((min, max)) => self.quantify(frame, min, max)?,
                    None => frame.items.push((self.literal('{'), 0)),
                },
                c => frame.items.push((self.literal(c), 0)),
            }
        }
        let frame = stack.pop().ok_or(Invalid)?;
        if !stack.is_empty() || !matches!(frame.kind, GroupKind::Root) {
            return Err(Invalid);
        }
        let (ast, _) = alternation(frame.alternatives, frame.items);
        self.check_tree(&ast)?;
        Ok(Parsed {
            ast,
            fold_cost: self.fold_cost,
        })
    }

    /// A reference to a group the pattern does not have is a PCRE2 compile error (`(a)\\2` is
    /// `#VALUE!`; text.md "REGEX* patterns and match iteration are PCRE2 (T-1688)").
    fn check_tree(&self, ast: &Ast) -> Result<(), Invalid> {
        let mut stack = vec![ast];
        while let Some(ast) = stack.pop() {
            let reference = match ast {
                Ast::Backref { reference, .. }
                | Ast::Conditional {
                    condition: Condition::Group(reference),
                    ..
                } => Some(reference),
                _ => None,
            };
            let known = match reference {
                Some(Reference::Number(n)) => (1..=self.groups).contains(n),
                Some(Reference::Name(name)) => self.names.contains(name),
                None => true,
            };
            if !known {
                return Err(Invalid);
            }
            if let Ast::LookAround {
                kind: LookKind::Behind | LookKind::NotBehind,
                ast,
            } = ast
            {
                let alternatives = match ast.as_ref() {
                    Ast::Alternation(asts) => asts.iter().collect(),
                    one => vec![one],
                };
                if alternatives
                    .into_iter()
                    .any(|ast| fixed_length(ast).is_none())
                {
                    return Err(Invalid);
                }
            }
            ast.push_children(&mut stack);
        }
        Ok(())
    }

    /// A literal character under the current flags.
    fn literal(&mut self, c: char) -> Ast {
        let fold = self.flags.caseless && self.charge(1);
        Ast::Literal { c, fold }
    }

    /// Applies a quantifier to the last item of `frame`, reading its lazy (`?`) or possessive
    /// (`+`) suffix.
    fn quantify(&mut self, frame: &mut Frame, min: u32, max: Option<u32>) -> Result<(), Invalid> {
        let (ast, depth) = frame.items.pop().ok_or(Invalid)?;
        if matches!(ast, Ast::Anchor(_) | Ast::KeepOut) {
            return Err(Invalid);
        }
        let kind = if self.eat('+') {
            RepeatKind::Possessive
        } else if self.eat('?') != self.flags.ungreedy {
            RepeatKind::Lazy
        } else {
            RepeatKind::Greedy
        };
        let depth = depth + 1;
        if depth > NEST_LIMIT {
            return Err(Invalid);
        }
        frame.items.push((
            Ast::Repeat {
                min,
                max,
                kind,
                ast: Box::new(ast),
            },
            depth,
        ));
        Ok(())
    }

    /// Reads `{n}`, `{n,}` or `{n,m}` after its `{`; `None` (and nothing read) when the brace
    /// starts no quantifier, which PCRE2 then reads as a literal.
    fn counted(&mut self) -> Result<Option<(u32, Option<u32>)>, Invalid> {
        let start = self.pos;
        // Spaces inside a quantifier's braces (`a{1, 2}`) are allowed from PCRE2 10.43 only, as
        // `{,m}` is: unprobed, so invalid rather than literal text (T-1972 probes both).
        let body: Vec<char> = self.chars[start..]
            .iter()
            .copied()
            .take_while(|&c| c.is_ascii_digit() || c == ',' || c == ' ')
            .take(32)
            .collect();
        if body.contains(&' ')
            && body.iter().any(char::is_ascii_digit)
            && self.chars.get(start + body.len()) == Some(&'}')
        {
            return Err(Invalid);
        }
        let Some(min) = self.number()? else {
            // `{,m}` is a quantifier from PCRE2 10.43 only: unprobed, so invalid.
            if self.peek() == Some(',') && self.peek_at(1).is_some_and(|c| c.is_ascii_digit()) {
                return Err(Invalid);
            }
            return Ok(None);
        };
        let max = if self.eat(',') {
            self.number()?
        } else {
            Some(min)
        };
        if !self.eat('}') {
            self.pos = start;
            return Ok(None);
        }
        if max.is_some_and(|max| max < min) {
            return Err(Invalid);
        }
        Ok(Some((min, max)))
    }

    /// A decimal repeat count, at most [`MAX_REPEAT`].
    fn number(&mut self) -> Result<Option<u32>, Invalid> {
        let mut value: Option<u32> = None;
        while let Some(digit) = self.peek().and_then(|c| c.to_digit(10)) {
            self.pos += 1;
            let next = value
                .unwrap_or(0)
                .checked_mul(10)
                .and_then(|v| v.checked_add(digit))
                .filter(|&v| v <= MAX_REPEAT)
                .ok_or(Invalid)?;
            value = Some(next);
        }
        Ok(value)
    }

    /// Reads a group opening after its `(`: the frame to push, or `None` for a comment or a
    /// flag setting, which open nothing.
    fn open_group(&mut self) -> Result<Option<Frame>, Invalid> {
        let outer = self.flags;
        let frame = |kind| {
            Some(Frame {
                kind,
                alternatives: Vec::new(),
                items: Vec::new(),
                outer,
            })
        };
        if self.peek() == Some('*') {
            // Verbs other than a leading `(*UCP)` are unprobed.
            return Err(Invalid);
        }
        if !self.eat('?') {
            if self.flags.no_auto_capture {
                return Ok(frame(GroupKind::NonCapture));
            }
            return Ok(frame(self.capture(None)?));
        }
        match self.next()? {
            '#' => {
                while self.next()? != ')' {}
                Ok(None)
            }
            ':' => Ok(frame(GroupKind::NonCapture)),
            '=' => Ok(frame(GroupKind::LookAround(LookKind::Ahead))),
            '!' => Ok(frame(GroupKind::LookAround(LookKind::NotAhead))),
            '>' => Ok(frame(GroupKind::Atomic)),
            '<' if self.eat('=') => Ok(frame(GroupKind::LookAround(LookKind::Behind))),
            '<' if self.eat('!') => Ok(frame(GroupKind::LookAround(LookKind::NotBehind))),
            '<' => {
                let name = self.name('>')?;
                Ok(frame(self.capture(Some(name))?))
            }
            '\'' => {
                let name = self.name('\'')?;
                Ok(frame(self.capture(Some(name))?))
            }
            'P' if self.eat('<') => {
                let name = self.name('>')?;
                Ok(frame(self.capture(Some(name))?))
            }
            '(' => {
                let reference = if self.peek().is_some_and(|c| c.is_ascii_digit()) {
                    let number = self.number()?.ok_or(Invalid)?;
                    Reference::Number(number)
                } else if self.eat('<') {
                    Reference::Name(self.name('>')?)
                } else if self.eat('\'') {
                    Reference::Name(self.name('\'')?)
                } else {
                    // Assertion conditions are left to T-2354; recursion (`R`) and `DEFINE` stay
                    // invalid (SPEC-0013 rollout step 6).
                    return Err(Invalid);
                };
                self.expect(')')?;
                Ok(frame(GroupKind::Conditional(Condition::Group(reference))))
            }
            _ => {
                self.pos -= 1;
                self.flag_group(outer)
            }
        }
    }

    /// A capture group, numbered in order of its opening parenthesis.
    fn capture(&mut self, name: Option<Box<str>>) -> Result<GroupKind, Invalid> {
        if let Some(name) = &name {
            if self.names.contains(name) {
                return Err(Invalid);
            }
            self.names.push(name.clone());
        }
        self.groups = self.groups.checked_add(1).ok_or(Invalid)?;
        if self.groups > MAX_REPEAT {
            return Err(Invalid);
        }
        Ok(GroupKind::Capture {
            index: self.groups,
            name,
        })
    }

    /// A group name up to `close`: word characters, not starting with a digit.
    fn name(&mut self, close: char) -> Result<Box<str>, Invalid> {
        let mut name = String::new();
        loop {
            let c = self.next()?;
            if c == close {
                break;
            }
            if !(c.is_ascii_alphanumeric() || c == '_') || name.len() >= MAX_NAME {
                return Err(Invalid);
            }
            name.push(c);
        }
        if name.is_empty() || name.starts_with(|c: char| c.is_ascii_digit()) {
            return Err(Invalid);
        }
        Ok(name.into())
    }

    /// `(?imnsxU-imnsxU)`, which sets the flags for the rest of the enclosing group, or
    /// `(?imnsxU-imnsxU:..)`, a group with them.
    fn flag_group(&mut self, outer: Flags) -> Result<Option<Frame>, Invalid> {
        let mut flags = self.flags;
        let mut on = true;
        loop {
            match self.next()? {
                '-' if on => on = false,
                'i' => flags.caseless = on,
                'm' => flags.multi_line = on,
                's' => flags.dot_all = on,
                'x' => flags.extended = on,
                'n' => flags.no_auto_capture = on,
                'U' => flags.ungreedy = on,
                ')' => {
                    self.flags = flags;
                    return Ok(None);
                }
                ':' => {
                    self.flags = flags;
                    return Ok(Some(Frame {
                        kind: GroupKind::NonCapture,
                        alternatives: Vec::new(),
                        items: Vec::new(),
                        outer,
                    }));
                }
                _ => return Err(Invalid),
            }
        }
    }

    /// Reads one escape after its `\`; `in_class` inside a bracket, where `\b` is a backspace
    /// and assertions are invalid.
    fn escape(&mut self, in_class: bool) -> Result<Escape, Invalid> {
        let c = self.next()?;
        let node = |ast| {
            if in_class {
                Err(Invalid)
            } else {
                Ok(Escape::Node(ast))
            }
        };
        Ok(match c {
            'd' | 'D' | 'w' | 'W' | 's' | 'S' => {
                Escape::Class(self.known_class(&format!("\\{c}"))?)
            }
            // PCRE2's `\h`, as `[:blank:]` (probes/regex-ascii-classes A41).
            'h' | 'H' => {
                let mut class = self.known_class(BLANK)?;
                if c == 'H' {
                    class.negate();
                }
                Escape::Class(class)
            }
            'p' | 'P' => Escape::Class(self.property(c == 'P')?),
            'b' if in_class => Escape::Char('\u{8}'),
            'b' => return node(Ast::Anchor(Anchor::WordBoundary)),
            'B' => return node(Ast::Anchor(Anchor::NotWordBoundary)),
            'A' => return node(Ast::Anchor(Anchor::Start)),
            'z' => return node(Ast::Anchor(Anchor::End)),
            'Z' => return node(Ast::Anchor(Anchor::EndOrFinalLf)),
            'G' => return node(Ast::Anchor(Anchor::SearchStart)),
            'K' => return node(Ast::KeepOut),
            'R' => return node(Ast::line_break()),
            'X' => return node(Ast::Grapheme),
            'Q' => Escape::Quote,
            'E' => Escape::Nothing,
            'g' => {
                let reference = self.g_reference()?;
                return node(self.backref(reference));
            }
            'k' => {
                let close = match self.next()? {
                    '<' => '>',
                    '\'' => '\'',
                    '{' => '}',
                    _ => return Err(Invalid),
                };
                let name = self.name(close)?;
                return node(self.backref(Reference::Name(name)));
            }
            '1'..='9' => {
                // A backreference (PCRE2 reads `\12` as octal when there are fewer groups; such
                // a pattern stays #VALUE!, as a reference to a missing group).
                self.pos -= 1;
                let number = self.number()?.ok_or(Invalid)?;
                return node(self.backref(Reference::Number(number)));
            }
            'a' => Escape::Char('\u{7}'),
            'e' => Escape::Char('\u{1B}'),
            'f' => Escape::Char('\u{C}'),
            'n' => Escape::Char('\n'),
            'r' => Escape::Char('\r'),
            't' => Escape::Char('\t'),
            '0' => {
                let mut value = 0;
                for _ in 0..2 {
                    match self.peek().and_then(|c| c.to_digit(8)) {
                        Some(d) => {
                            self.pos += 1;
                            value = value * 8 + d;
                        }
                        None => break,
                    }
                }
                Escape::Char(char::from_u32(value).ok_or(Invalid)?)
            }
            'o' => {
                self.expect('{')?;
                Escape::Char(self.braced_code(8)?)
            }
            'x' => {
                if self.eat('{') {
                    Escape::Char(self.braced_code(16)?)
                } else {
                    let mut value = 0;
                    for _ in 0..2 {
                        match self.peek().and_then(|c| c.to_digit(16)) {
                            Some(d) => {
                                self.pos += 1;
                                value = value * 16 + d;
                            }
                            None => break,
                        }
                    }
                    Escape::Char(char::from_u32(value).ok_or(Invalid)?)
                }
            }
            'c' => {
                let c = self.next()?;
                if !c.is_ascii() || c.is_ascii_control() {
                    return Err(Invalid);
                }
                let value = u32::from(c.to_ascii_uppercase()) ^ 0x40;
                Escape::Char(char::from_u32(value).ok_or(Invalid)?)
            }
            // `\v`, `\N`, `\C`, `\u`, `\l`, `\L`, `\U` and every other letter or digit: PCRE2
            // errors or gives a meaning no probe has shown Excel accepts.
            c if c.is_ascii_alphanumeric() => return Err(Invalid),
            c => Escape::Char(c),
        })
    }

    /// The code point of `{digits}` in `radix`, after its `{`.
    fn braced_code(&mut self, radix: u32) -> Result<char, Invalid> {
        let mut value: u32 = 0;
        let mut digits = 0;
        loop {
            let c = self.next()?;
            if c == '}' {
                break;
            }
            let d = c.to_digit(radix).ok_or(Invalid)?;
            value = value
                .checked_mul(radix)
                .and_then(|v| v.checked_add(d))
                .ok_or(Invalid)?;
            digits += 1;
        }
        if digits == 0 {
            return Err(Invalid);
        }
        char::from_u32(value).ok_or(Invalid)
    }

    /// A backreference to `reference`, ignoring case where the flags do.
    fn backref(&self, reference: Reference) -> Ast {
        Ast::Backref {
            reference,
            fold: self.flags.caseless,
        }
    }

    /// `\g` followed by `n`, `{n}`, `{-n}`, `-n`, `+n` or `{name}`, after the `g`. Relative
    /// references are resolved against the groups opened so far.
    fn g_reference(&mut self) -> Result<Reference, Invalid> {
        let braced = self.eat('{');
        let reference = if self.peek().is_some_and(|c| c.is_ascii_digit()) {
            Reference::Number(self.number()?.ok_or(Invalid)?)
        } else if self.eat('-') {
            let back = self.number()?.ok_or(Invalid)?;
            let number = (self.groups + 1).checked_sub(back).ok_or(Invalid)?;
            Reference::Number(number)
        } else if braced {
            self.pos -= 1;
            self.expect('{')?;
            return Ok(Reference::Name(self.name('}')?));
        } else {
            // `\g+n` and `\g<..>` are subroutine calls: unprobed.
            return Err(Invalid);
        };
        if braced {
            self.expect('}')?;
        }
        Ok(reference)
    }

    /// `\p{..}`, `\pL` or their negations, as regex-syntax's Unicode tables give them.
    fn property(&mut self, negated: bool) -> Result<ClassUnicode, Invalid> {
        let mut name = String::new();
        if self.eat('{') {
            loop {
                let c = self.next()?;
                if c == '}' {
                    break;
                }
                if !(c.is_ascii_alphanumeric() || matches!(c, '_' | '^' | ' ' | '&' | '-')) {
                    return Err(Invalid);
                }
                name.push(c);
            }
        } else {
            name.push(self.next()?);
        }
        let (negated, name) = match name.strip_prefix('^') {
            Some(rest) => (!negated, rest.to_string()),
            None => (negated, name),
        };
        if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(Invalid);
        }
        let mut class = self.known_class(&format!("\\p{{{name}}}"))?;
        if negated {
            class.negate();
        }
        Ok(class)
    }

    /// The class regex-syntax builds for `text`, one of our fixed class spellings (`\d`, `\w`,
    /// `\p{..}`, a POSIX class), folded when the flags ignore case. The fold is charged by the
    /// code points it walks.
    fn known_class(&mut self, text: &str) -> Result<ClassUnicode, Invalid> {
        let mut class = class_of(text)?;
        if self.flags.caseless && self.charge_fold(&class) {
            class.case_fold_simple();
        }
        Ok(class)
    }

    /// Charges folding `class`, [`FOLD_UNITS`] per code point it covers; whether it may be
    /// folded.
    fn charge_fold(&mut self, class: &ClassUnicode) -> bool {
        let span = class.iter().fold(0usize, |sum, range| {
            let span = u32::from(range.end()) - u32::from(range.start()) + 1;
            sum.saturating_add(usize::try_from(span).unwrap_or(usize::MAX))
        });
        self.charge(span.saturating_mul(FOLD_UNITS))
    }

    /// Adds `units` to the fold cost; false, and the cost `usize::MAX`, once past the budget.
    fn charge(&mut self, units: usize) -> bool {
        match self.fold_cost.checked_add(units) {
            Some(cost) if cost <= self.fold_budget => {
                self.fold_cost = cost;
                true
            }
            _ => {
                self.fold_cost = usize::MAX;
                false
            }
        }
    }

    /// A bracket after its `[`, `depth` levels deep. Brackets nest and `&&` intersects (as the
    /// regex crate read them: probes/regex-posix-classes-b, -c, T-1903, T-1989). Under `(?i)`
    /// every item folds except a POSIX class (`(?i)[[:upper:]]` is FALSE on `a`; a nested
    /// bracket folds by this same rule inside itself).
    fn bracket(&mut self, depth: u32) -> Result<ClassUnicode, Invalid> {
        if depth > NEST_LIMIT {
            return Err(Invalid);
        }
        let negated = self.eat('^');
        // Items that fold under `(?i)` and those that never do.
        let mut plain = ClassUnicode::empty();
        let mut fixed = ClassUnicode::empty();
        let mut intersection: Option<ClassUnicode> = None;
        let mut first = true;
        let mut quoting = false;
        loop {
            let c = self.next()?;
            if quoting {
                if c == '\\' && self.eat('E') {
                    quoting = false;
                } else {
                    self.class_char(c, &mut plain)?;
                }
                continue;
            }
            match c {
                ']' if !first => break,
                '[' => {
                    if let Some(class) = self.posix_class()? {
                        fixed.union(&class);
                    } else {
                        let class = self.bracket(depth + 1)?;
                        fixed.union(&class);
                    }
                    self.no_range_after_class()?;
                }
                // `--`, `~~` and `||` are set operators in the regex crate's (UTS #18) brackets
                // and literals in stock PCRE2; Excel's brackets nest like the former, so the
                // meaning is unprobed: invalid until T-1972 answers.
                '-' | '~' | '|' if self.peek() == Some(c) => return Err(Invalid),
                '&' if self.eat('&') => {
                    let mut set = self.close_set(plain, fixed);
                    if let Some(before) = intersection {
                        set.intersect(&before);
                    }
                    intersection = Some(set);
                    plain = ClassUnicode::empty();
                    fixed = ClassUnicode::empty();
                }
                '\\' => match self.escape(true)? {
                    Escape::Char(c) => self.class_char(c, &mut plain)?,
                    Escape::Class(class) => {
                        // Already folded by `known_class` when it should be.
                        fixed.union(&class);
                        self.no_range_after_class()?;
                    }
                    Escape::Quote => quoting = true,
                    Escape::Nothing => {}
                    Escape::Node(_) => return Err(Invalid),
                },
                c => self.class_char(c, &mut plain)?,
            }
            first = false;
        }
        let mut set = self.close_set(plain, fixed);
        if let Some(before) = intersection {
            set.intersect(&before);
        }
        if negated {
            set.negate();
        }
        Ok(set)
    }

    /// The union of a bracket's items, `plain` folded under `(?i)`.
    fn close_set(&mut self, mut plain: ClassUnicode, fixed: ClassUnicode) -> ClassUnicode {
        if self.flags.caseless && self.charge_fold(&plain) {
            plain.case_fold_simple();
        }
        plain.union(&fixed);
        plain
    }

    /// Adds the character `lo` to a bracket, or the range it starts when a `-` and an end
    /// follow (`-` before `]` is a literal).
    fn class_char(&mut self, lo: char, plain: &mut ClassUnicode) -> Result<(), Invalid> {
        let mut hi = lo;
        if self.peek() == Some('-') && self.peek_at(1).is_some_and(|c| c != ']') {
            self.pos += 1;
            hi = match self.next()? {
                '\\' => match self.escape(true)? {
                    Escape::Char(c) => c,
                    _ => return Err(Invalid),
                },
                // A range to a class (`[a-[:alpha:]]`) is a PCRE2 error (T-1985, T-2010).
                '[' => return Err(Invalid),
                c => c,
            };
            if hi < lo {
                return Err(Invalid);
            }
        }
        plain.push(ClassUnicodeRange::new(lo, hi));
        Ok(())
    }

    /// A range from a class (`[[:alpha:]-z]`, `[\d-z]`) is a PCRE2 error, while a `-` before the
    /// closing `]` stays a literal (`[[:alpha:]-]`; T-1985, T-2010).
    fn no_range_after_class(&self) -> Result<(), Invalid> {
        if self.peek() == Some('-') && self.peek_at(1).is_some_and(|c| c != ']') {
            return Err(Invalid);
        }
        Ok(())
    }

    /// After a `[` inside a bracket: a POSIX class `[:name:]` or `[:^name:]` (consumed), or
    /// `None` when the text is no POSIX-shaped item (a nested bracket follows). A collating
    /// element `[.a.]`, an equivalence class `[=a=]` and an unknown or upper-case name are
    /// PCRE2 errors (probes/regex-posix-classes Z1-Z3, Z8-Z10, T-1985).
    fn posix_class(&mut self) -> Result<Option<ClassUnicode>, Invalid> {
        let Some(mark) = self.peek().filter(|c| matches!(c, ':' | '.' | '=')) else {
            return Ok(None);
        };
        let rest = &self.chars[self.pos + 1..];
        // The item ends at the first `]`, which must follow the mark: one bounded scan.
        let Some(end) = rest
            .iter()
            .position(|&c| c == ']')
            .and_then(|close| close.checked_sub(1))
            .filter(|&end| rest[end] == mark)
        else {
            return Ok(None);
        };
        let inner = &rest[..end];
        if inner.iter().any(|c| matches!(c, '[' | ']' | '\\')) {
            return Ok(None);
        }
        if mark != ':' {
            return Err(Invalid);
        }
        let (negated, name) = match inner.split_first() {
            Some(('^', name)) => (true, name),
            _ => (false, inner),
        };
        let name: String = name.iter().collect();
        let text = posix_class_text(&name).ok_or(Invalid)?;
        self.pos += end + 3;
        let mut class = class_of(text)?;
        if negated {
            class.negate();
        }
        Ok(Some(class))
    }
}

/// The items of one alternative as a node, and its depth.
fn concat(mut items: Vec<(Ast, u32)>) -> (Ast, u32) {
    let depth = items.iter().map(|(_, d)| *d).max().unwrap_or(0);
    let ast = match items.len() {
        0 => Ast::Empty,
        1 => items.pop().map_or(Ast::Empty, |(ast, _)| ast),
        _ => Ast::Concat(items.into_iter().map(|(ast, _)| ast).collect()),
    };
    (ast, depth)
}

/// The alternatives of a group, the last one's `items` included, as a node and its depth.
fn alternation(mut alternatives: Vec<(Ast, u32)>, items: Vec<(Ast, u32)>) -> (Ast, u32) {
    let last = concat(items);
    if alternatives.is_empty() {
        return last;
    }
    alternatives.push(last);
    let depth = alternatives.iter().map(|(_, d)| *d).max().unwrap_or(0);
    let ast = Ast::Alternation(alternatives.into_iter().map(|(ast, _)| ast).collect());
    (ast, depth)
}

/// The node a closed group becomes, one level deeper than its contents.
fn close(frame: Frame) -> Result<(Ast, u32), Invalid> {
    let alternatives = frame.alternatives.len() + 1;
    let (ast, depth) = alternation(frame.alternatives, frame.items);
    let depth = depth + 1;
    if depth > NEST_LIMIT {
        return Err(Invalid);
    }
    let ast = Box::new(ast);
    let ast = match frame.kind {
        GroupKind::Root => return Err(Invalid),
        GroupKind::Capture { index, name } => Ast::Capture { index, name, ast },
        GroupKind::NonCapture => *ast,
        GroupKind::LookAround(kind) => Ast::LookAround { kind, ast },
        GroupKind::Atomic => Ast::Atomic(ast),
        GroupKind::Conditional(condition) => {
            if alternatives > 2 {
                return Err(Invalid);
            }
            let (yes, no) = match *ast {
                Ast::Alternation(mut both) if alternatives == 2 => {
                    let no = both.pop().ok_or(Invalid)?;
                    let yes = both.pop().ok_or(Invalid)?;
                    (yes, no)
                }
                yes => (yes, Ast::Empty),
            };
            Ast::Conditional {
                condition,
                yes: Box::new(yes),
                no: Box::new(no),
            }
        }
    };
    Ok((ast, depth))
}
