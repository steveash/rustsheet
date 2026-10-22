//! The syntax tree of a PCRE2 pattern (SPEC-0013 section 1).
//!
//! The parser has already resolved flags, `\Q..\E`, comments and extended-mode white space:
//! every node carries the meaning its flags gave it (a literal knows whether it ignores case, a
//! class is already folded where it should be). The tree is at most [`super::parse::NEST_LIMIT`]
//! deep, groups and stacked quantifiers counted alike, so the recursive passes over it (its
//! `Drop` included) are bounded.

use regex_syntax::hir::ClassUnicode;

/// One node of a pattern.
#[derive(Debug, Clone)]
pub(super) enum Ast {
    /// Matches the empty string.
    Empty,
    /// One character; `fold` when it ignores case (simple case folding).
    Literal { c: char, fold: bool },
    /// A character class, already folded and negated as its flags say.
    Class(ClassUnicode),
    /// `.`: any character, `\n` too when `dot_all`.
    Dot { dot_all: bool },
    /// A zero-width assertion.
    Anchor(Anchor),
    /// A capture group: its 1-based index and name.
    Capture {
        index: u32,
        name: Option<Box<str>>,
        ast: Box<Ast>,
    },
    /// Items in sequence.
    Concat(Vec<Ast>),
    /// Alternatives in priority order.
    Alternation(Vec<Ast>),
    /// A quantified item.
    Repeat {
        min: u32,
        max: Option<u32>,
        kind: RepeatKind,
        ast: Box<Ast>,
    },
    /// A backreference to a group, by number (resolved once the pattern is parsed) or name;
    /// `fold` when it ignores case (simple case folding, as its scope's flags say).
    Backref { reference: Reference, fold: bool },
    /// A lookahead or lookbehind.
    LookAround { kind: LookKind, ast: Box<Ast> },
    /// `(?>..)`.
    Atomic(Box<Ast>),
    /// `(?(cond)yes|no)`; `no` is [`Ast::Empty`] when absent.
    Conditional {
        condition: Condition,
        yes: Box<Ast>,
        no: Box<Ast>,
    },
    /// `\K`: resets the start of the reported match.
    KeepOut,
    /// `\R` (any line break, atomic in PCRE2) and `\X` (an extended grapheme cluster): Excel
    /// accepts both (probes/regex-ascii-classes A27, A42); T-2287 implements them.
    Unimplemented,
}

/// A zero-width assertion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Anchor {
    /// `^` without `(?m)`, `\A`.
    Start,
    /// `^` under `(?m)`.
    StartLine,
    /// `$` without `(?m)`, `\Z`: at the end or before a final `\n`.
    EndOrFinalLf,
    /// `$` under `(?m)`.
    EndLine,
    /// `\z`.
    End,
    /// `\b`.
    WordBoundary,
    /// `\B`.
    NotWordBoundary,
    /// `\G`: where the search started.
    SearchStart,
}

/// How a quantifier repeats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RepeatKind {
    Greedy,
    Lazy,
    Possessive,
}

/// The group a backreference or a condition names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Reference {
    Number(u32),
    Name(Box<str>),
}

/// What a conditional group tests: whether the group has matched. Assertion conditions
/// (`(?(?=x)..)`) are left to T-2354 and are invalid until then.
#[derive(Debug, Clone)]
pub(super) enum Condition {
    Group(Reference),
}

/// Which lookaround.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LookKind {
    Ahead,
    NotAhead,
    Behind,
    NotBehind,
}

impl Ast {
    /// Whether the tree runs on the automata: it holds none of the constructs only a backtracker
    /// can run (SPEC-0013 section 2). The list is closed: a construct added later is non-regular
    /// unless it is added here with a test.
    pub(super) fn is_regular(&self) -> bool {
        let mut stack = vec![self];
        while let Some(ast) = stack.pop() {
            match ast {
                Ast::Empty
                | Ast::Literal { .. }
                | Ast::Class(_)
                | Ast::Dot { .. }
                | Ast::Anchor(
                    Anchor::Start
                    | Anchor::StartLine
                    | Anchor::EndOrFinalLf
                    | Anchor::EndLine
                    | Anchor::End
                    | Anchor::WordBoundary
                    | Anchor::NotWordBoundary,
                ) => {}
                Ast::Capture { ast, .. } => stack.push(ast),
                Ast::Repeat { kind, ast, .. } => {
                    if *kind == RepeatKind::Possessive {
                        return false;
                    }
                    stack.push(ast);
                }
                Ast::Concat(asts) | Ast::Alternation(asts) => stack.extend(asts),
                Ast::Anchor(Anchor::SearchStart)
                | Ast::Backref { .. }
                | Ast::LookAround { .. }
                | Ast::Atomic(_)
                | Ast::Conditional { .. }
                | Ast::KeepOut
                | Ast::Unimplemented => return false,
            }
        }
        true
    }
}

/// The characters `ast` always matches, `None` when that varies. PCRE2 requires it of each
/// top-level alternative of a lookbehind (`(?<=a+)` is `#VALUE!`; text.md "REGEX* patterns and
/// match iteration are PCRE2 (T-1688)"). The recursion is bounded by the parser's nest limit.
pub(super) fn fixed_length(ast: &Ast) -> Option<u32> {
    match ast {
        Ast::Empty | Ast::Anchor(_) | Ast::KeepOut | Ast::LookAround { .. } => Some(0),
        Ast::Literal { .. } | Ast::Class(_) | Ast::Dot { .. } => Some(1),
        Ast::Capture { ast, .. } | Ast::Atomic(ast) => fixed_length(ast),
        Ast::Concat(asts) => asts
            .iter()
            .try_fold(0u32, |sum, ast| sum.checked_add(fixed_length(ast)?)),
        Ast::Alternation(asts) => {
            // Each alternative is measured once: measuring the first twice doubles the work
            // at every nested level (`(?<=(?:(?:a|b)|c)|d)` 250 deep).
            let mut lengths = asts.iter().map(fixed_length);
            let first = lengths.next()??;
            lengths.all(|length| length == Some(first)).then_some(first)
        }
        Ast::Repeat { min, max, ast, .. } if Some(*min) == *max => {
            min.checked_mul(fixed_length(ast)?)
        }
        Ast::Repeat { .. } | Ast::Backref { .. } | Ast::Conditional { .. } | Ast::Unimplemented => {
            None
        }
    }
}
