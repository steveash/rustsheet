//! POSIX bracket classes (`[[:alpha:]]`) as Excel's PCRE2 (UCP mode) reads them.
//!
//! The regex crate's POSIX classes are ASCII-only; Excel's are Unicode (probes/regex-posix-classes,
//! docs/excel-semantics/text.md "REGEX* POSIX classes"). [`unicode_posix_classes`] rewrites every
//! POSIX class item of a parsed pattern into the Unicode class Excel matches and prints the
//! pattern back. Escapes, nesting and set operations are the parser's, so nothing is rewritten
//! inside an escaped `\[` or a literal. A pattern that does not parse is returned unchanged: it
//! fails to compile the same way.
//!
//! PCRE2 rejects POSIX syntax the parser keeps as plain brackets or literals: `[:alpha:]` outside a
//! bracket, an unknown or upper-case class name, collating elements (`[.a.]`) and equivalence
//! classes (`[=a=]`), and a range that starts at a class (`[[:alpha:]-z]`); Excel gives
//! `#VALUE!` for each (probes/regex-posix-classes Z1-Z3, Z8-Z10; T-1985). [`unicode_posix_classes`]
//! reports them as an [`InvalidPosixSyntax`].
//!
//! No POSIX class folds under case-insensitive matching (`(?i)[[:upper:]]` does not match `a`,
//! `(?i)[[:ascii:]]` not the Kelvin sign), while the other items of its bracket do, so where
//! `(?i)` is on such a bracket becomes `(?-i:[...])` with those items folded explicitly.

use std::borrow::Cow;

use regex_syntax::ast::{
    self, parse::Parser, print::Printer, Ast, ClassAsciiKind, ClassSet, ClassSetItem,
};

/// The Unicode class each POSIX class matches, as a bracket (probes/regex-posix-classes; the
/// characters it does not probe follow PCRE2's UCP definitions).
fn unicode_class(kind: &ClassAsciiKind) -> Option<&'static str> {
    Some(match kind {
        ClassAsciiKind::Alnum => r"[\p{L}\p{N}]",
        ClassAsciiKind::Alpha => r"[\p{L}]",
        // PCRE2's `\h`: Zs, tab and U+180E; LF, VT, NEL and U+2028 are not blank.
        ClassAsciiKind::Blank => r"[\t\p{Zs}\x{180E}]",
        ClassAsciiKind::Cntrl => r"[\p{Cc}]",
        ClassAsciiKind::Digit => r"[\p{Nd}]",
        // L, M, N, P, S and Cf (soft hyphen, ZWSP and BOM are graphic), but no separator.
        ClassAsciiKind::Graph => {
            r"[[\p{L}\p{M}\p{N}\p{P}\p{S}\p{Cf}]--[\x{61C}\x{180E}\x{2066}-\x{2069}]]"
        }
        ClassAsciiKind::Lower => r"[\p{Ll}]",
        // Graph plus the Zs spaces (not tab, LF or U+2028).
        ClassAsciiKind::Print => {
            r"[[\p{L}\p{M}\p{N}\p{P}\p{S}\p{Cf}\p{Zs}]--[\x{61C}\x{180E}\x{2066}-\x{2069}]]"
        }
        // Every P, plus the ASCII symbols (`$+<=>^`|~`); `×` and `€` are not punctuation.
        ClassAsciiKind::Punct => r"[\p{P}[\x00-\x7F&&\p{S}]]",
        // White_Space plus U+180E (probes/regex-posix-classes-c A29): NEL, NBSP, U+2028 and
        // U+3000 are spaces, ZWSP is not.
        ClassAsciiKind::Space => r"[\p{White_Space}\x{180E}]",
        ClassAsciiKind::Upper => r"[\p{Lu}]",
        // Letters, all numbers (`²`, `½`, `Ⅳ`), non-spacing marks and connectors.
        ClassAsciiKind::Word => r"[\p{L}\p{N}\p{Mn}\p{Pc}]",
        ClassAsciiKind::Xdigit => r"[0-9A-Fa-f\x{FF10}-\x{FF19}\x{FF21}-\x{FF26}\x{FF41}-\x{FF46}]",
        // ASCII is ASCII in both.
        ClassAsciiKind::Ascii => return None,
    })
}

/// A pattern whose POSIX class syntax PCRE2 rejects.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct InvalidPosixSyntax;

/// `pattern` with its POSIX classes rewritten to Excel's Unicode ones; borrowed when it has none
/// or does not parse (it then fails to compile the same way). `case_insensitive` is the
/// `case_sensitivity` argument's choice, which `(?i)` and `(?-i)` in the pattern override. `Err`
/// when PCRE2 rejects its POSIX syntax, or when its brackets under `(?i)` would grow past
/// [`MAX_UNFOLDED_LEN`].
pub(super) fn unicode_posix_classes(
    pattern: &str,
    case_insensitive: bool,
) -> Result<Cow<'_, str>, InvalidPosixSyntax> {
    if !["[:", "[.", "[="].iter().any(|open| pattern.contains(open)) {
        return Ok(Cow::Borrowed(pattern));
    }
    let Ok(mut ast) = Parser::new().parse(pattern) else {
        return Ok(Cow::Borrowed(pattern));
    };
    if invalid_ast(&ast, pattern) {
        return Err(InvalidPosixSyntax);
    }
    if !rewrite(&mut ast, &mut { case_insensitive }, &mut {
        MAX_UNFOLDED_LEN
    })? {
        return Ok(Cow::Borrowed(pattern));
    }
    let mut out = String::with_capacity(pattern.len() * 2);
    // Printing to a String cannot fail.
    if Printer::new().print(&ast, &mut out).is_err() {
        return Ok(Cow::Borrowed(pattern));
    }
    Ok(Cow::Owned(out))
}

/// Whether a bracket under `ast` holds POSIX syntax PCRE2 rejects. The recursion is bounded by
/// the parser's nesting limit.
fn invalid_ast(ast: &Ast, pattern: &str) -> bool {
    match ast {
        Ast::ClassBracketed(class) => {
            posix_shaped(&class.span, pattern) || invalid_set(&class.kind, pattern)
        }
        Ast::Repetition(rep) => invalid_ast(&rep.ast, pattern),
        Ast::Group(group) => invalid_ast(&group.ast, pattern),
        Ast::Alternation(alt) => alt.asts.iter().any(|a| invalid_ast(a, pattern)),
        Ast::Concat(cat) => cat.asts.iter().any(|a| invalid_ast(a, pattern)),
        Ast::Empty(_)
        | Ast::Flags(_)
        | Ast::Literal(_)
        | Ast::Dot(_)
        | Ast::Assertion(_)
        | Ast::ClassUnicode(_)
        | Ast::ClassPerl(_) => false,
    }
}

/// Walks the `lhs` chain of set operations in a loop: `[a&&b&&c...]` nests as deep as the pattern
/// is long, which the parser's nesting limit does not bound.
fn invalid_set(mut set: &ClassSet, pattern: &str) -> bool {
    while let ClassSet::BinaryOp(op) = set {
        if invalid_set(&op.rhs, pattern) {
            return true;
        }
        set = &op.lhs;
    }
    matches!(set, ClassSet::Item(item) if invalid_item(item, pattern))
}

fn invalid_item(item: &ClassSetItem, pattern: &str) -> bool {
    match item {
        // The parser reads a class followed by `-` and an item as three items, not a range.
        ClassSetItem::Union(union) => {
            union.items.iter().any(|i| invalid_item(i, pattern))
                || union.items.windows(3).any(|w| {
                    matches!(w[0], ClassSetItem::Ascii(_))
                        && matches!(&w[1], ClassSetItem::Literal(lit)
                            if lit.c == '-' && lit.kind == ast::LiteralKind::Verbatim)
                })
        }
        ClassSetItem::Bracketed(class) => {
            posix_shaped(&class.span, pattern) || invalid_set(&class.kind, pattern)
        }
        ClassSetItem::Empty(_)
        | ClassSetItem::Literal(_)
        | ClassSetItem::Range(_)
        | ClassSetItem::Ascii(_)
        | ClassSetItem::Unicode(_)
        | ClassSetItem::Perl(_) => false,
    }
}

/// Whether the bracket at `span` reads `[:name:]`, `[.name.]` or `[=name=]`: a POSIX class, which
/// PCRE2 allows only as a valid name directly inside a bracket (the parser has turned a valid
/// one into [`ClassSetItem::Ascii`] already), or a collating element or equivalence class, which
/// it never does.
fn posix_shaped(span: &ast::Span, pattern: &str) -> bool {
    let Some(text) = pattern.get(span.start.offset..span.end.offset) else {
        return false;
    };
    let Some(inner) = text.strip_prefix('[').and_then(|t| t.strip_suffix(']')) else {
        return false;
    };
    [':', '.', '='].iter().any(|&mark| {
        inner.len() >= 2
            && inner.starts_with(mark)
            && inner.ends_with(mark)
            && !inner[1..inner.len() - 1].contains(['[', ']', '\\'])
    })
}

/// Rewrites the POSIX classes under `ast`; whether there were any. `case_insensitive` is whether
/// `(?i)` is on where `ast` starts; it is updated as the regex-syntax translator does: a flag
/// group (`(?i)`) sets it for the rest of its enclosing group, across `|`, and a group restores
/// it on exit. The recursion is bounded by the parser's nesting limit.
fn rewrite(
    ast: &mut Ast,
    case_insensitive: &mut bool,
    budget: &mut usize,
) -> Result<bool, InvalidPosixSyntax> {
    Ok(match ast {
        Ast::ClassBracketed(class) => {
            if *case_insensitive && has_posix_class(&class.kind) {
                *ast = unfolded_posix_bracket(class, budget)?;
                return Ok(true);
            }
            rewrite_set(&mut class.kind)
        }
        Ast::Flags(flags) => {
            set_case_insensitive(&flags.flags, case_insensitive);
            false
        }
        Ast::Repetition(rep) => rewrite(&mut rep.ast, case_insensitive, budget)?,
        Ast::Group(group) => {
            let mut inner = *case_insensitive;
            if let ast::GroupKind::NonCapturing(flags) = &group.kind {
                set_case_insensitive(flags, &mut inner);
            }
            rewrite(&mut group.ast, &mut inner, budget)?
        }
        Ast::Alternation(alt) => rewrite_all(&mut alt.asts, case_insensitive, budget)?,
        Ast::Concat(cat) => rewrite_all(&mut cat.asts, case_insensitive, budget)?,
        Ast::Empty(_)
        | Ast::Literal(_)
        | Ast::Dot(_)
        | Ast::Assertion(_)
        | Ast::ClassUnicode(_)
        | Ast::ClassPerl(_) => false,
    })
}

/// [`rewrite`] over `asts` in order, the flags of one carrying to the next; whether any rewrote.
fn rewrite_all(
    asts: &mut [Ast],
    case_insensitive: &mut bool,
    budget: &mut usize,
) -> Result<bool, InvalidPosixSyntax> {
    let mut any = false;
    for ast in asts {
        any |= rewrite(ast, case_insensitive, budget)?;
    }
    Ok(any)
}

/// Applies the `i` items of `flags` (`(?i)`, `(?-i)`, `(?x-i)`) to `case_insensitive`.
fn set_case_insensitive(flags: &ast::Flags, case_insensitive: &mut bool) {
    let mut negated = false;
    for item in &flags.items {
        match item.kind {
            ast::FlagsItemKind::Negation => negated = true,
            ast::FlagsItemKind::Flag(ast::Flag::CaseInsensitive) => *case_insensitive = !negated,
            ast::FlagsItemKind::Flag(_) => {}
        }
    }
}

fn rewrite_set(set: &mut ClassSet) -> bool {
    match set {
        ClassSet::Item(item) => rewrite_item(item),
        ClassSet::BinaryOp(op) => rewrite_set(&mut op.lhs) | rewrite_set(&mut op.rhs),
    }
}

fn rewrite_item(item: &mut ClassSetItem) -> bool {
    match item {
        ClassSetItem::Ascii(ascii) => {
            let Some(mut class) = unicode_class(&ascii.kind).and_then(parse_bracket) else {
                return false;
            };
            class.negated = ascii.negated;
            *item = ClassSetItem::Bracketed(Box::new(class));
            true
        }
        ClassSetItem::Bracketed(class) => rewrite_set(&mut class.kind),
        ClassSetItem::Union(union) => union
            .items
            .iter_mut()
            .fold(false, |any, i| rewrite_item(i) | any),
        ClassSetItem::Empty(_)
        | ClassSetItem::Literal(_)
        | ClassSetItem::Range(_)
        | ClassSetItem::Unicode(_)
        | ClassSetItem::Perl(_) => false,
    }
}

/// Whether `set` holds a POSIX class at any depth.
fn has_posix_class(set: &ClassSet) -> bool {
    match set {
        ClassSet::Item(item) => item_has_posix_class(item),
        ClassSet::BinaryOp(op) => has_posix_class(&op.lhs) || has_posix_class(&op.rhs),
    }
}

fn item_has_posix_class(item: &ClassSetItem) -> bool {
    match item {
        ClassSetItem::Ascii(_) => true,
        ClassSetItem::Bracketed(class) => has_posix_class(&class.kind),
        ClassSetItem::Union(union) => union.items.iter().any(item_has_posix_class),
        ClassSetItem::Empty(_)
        | ClassSetItem::Literal(_)
        | ClassSetItem::Range(_)
        | ClassSetItem::Unicode(_)
        | ClassSetItem::Perl(_) => false,
    }
}

/// The most text the brackets under `(?i)` of one pattern may be rewritten to, all together. A
/// folded `\p{L}` prints as some thousand ranges, so a pattern of many such items is refused as
/// soon as their printed folds pass this, before the rest is built (CLAUDE.md directive 5); the
/// NFA size limit would refuse it next anyway.
const MAX_UNFOLDED_LEN: usize = 1 << 16;

/// Takes `len` bytes from `budget`, or refuses the pattern when it has not that many left.
fn spend(budget: &mut usize, len: usize) -> Result<(), InvalidPosixSyntax> {
    *budget = budget.checked_sub(len).ok_or(InvalidPosixSyntax)?;
    Ok(())
}

/// A bracket holding a POSIX class where `(?i)` is on, as `(?-i:[...])` with its POSIX classes
/// rewritten and every other item replaced by its case closure: no POSIX class folds, the other
/// items still do, whatever the negation, nesting or set operation (`(?i)[^[:upper:]x]` matches
/// `b` but neither `B` nor `X`; probes/regex-posix-classes-b A8-A11, -c A1-A26, T-1989).
fn unfolded_posix_bracket(
    class: &ast::ClassBracketed,
    budget: &mut usize,
) -> Result<Ast, InvalidPosixSyntax> {
    let mut class = class.clone();
    fold_set(&mut class.kind, budget)?;
    rewrite_set(&mut class.kind);
    let mut text = String::from("(?-i:");
    Printer::new()
        .print(&Ast::class_bracketed(class), &mut text)
        .map_err(|_| InvalidPosixSyntax)?;
    text.push(')');
    spend(budget, text.len())?;
    Parser::new().parse(&text).map_err(|_| InvalidPosixSyntax)
}

/// Replaces the items of `set` that are no POSIX class by their case closure. The `lhs` chain of
/// set operations is walked in a loop (see [`invalid_set`]).
fn fold_set(mut set: &mut ClassSet, budget: &mut usize) -> Result<(), InvalidPosixSyntax> {
    loop {
        match set {
            ClassSet::BinaryOp(op) => {
                fold_set(&mut op.rhs, budget)?;
                set = &mut op.lhs;
            }
            ClassSet::Item(item) => return fold_item(item, budget),
        }
    }
}

/// [`fold_set`] for one item: a union's plain items fold together, into one bracket.
fn fold_item(item: &mut ClassSetItem, budget: &mut usize) -> Result<(), InvalidPosixSyntax> {
    match item {
        ClassSetItem::Bracketed(class) => fold_set(&mut class.kind, budget),
        ClassSetItem::Union(union) => {
            let mut plain = Vec::new();
            let mut kept = Vec::new();
            for mut one in union.items.drain(..) {
                match one {
                    ClassSetItem::Literal(_)
                    | ClassSetItem::Range(_)
                    | ClassSetItem::Unicode(_)
                    | ClassSetItem::Perl(_) => plain.push(escaped(&one)),
                    ClassSetItem::Empty(_) => {}
                    _ => {
                        fold_item(&mut one, budget)?;
                        kept.push(one);
                    }
                }
            }
            if !plain.is_empty() {
                let span = union.span;
                let union = ClassSetItem::Union(ast::ClassSetUnion { span, items: plain });
                kept.push(folded(union, budget)?);
            }
            union.items = kept;
            Ok(())
        }
        ClassSetItem::Literal(_)
        | ClassSetItem::Range(_)
        | ClassSetItem::Unicode(_)
        | ClassSetItem::Perl(_) => {
            *item = folded(escaped(item), budget)?;
            Ok(())
        }
        ClassSetItem::Empty(_) | ClassSetItem::Ascii(_) => Ok(()),
    }
}

/// The case closure of `item` (regex-syntax's simple case folding, the one `(?i)` applies), as a
/// bracket.
fn folded(item: ClassSetItem, budget: &mut usize) -> Result<ClassSetItem, InvalidPosixSyntax> {
    let span = *item.span();
    let bracket = Ast::class_bracketed(ast::ClassBracketed {
        span,
        negated: false,
        kind: ClassSet::Item(item),
    });
    let mut text = String::from("(?i)");
    Printer::new()
        .print(&bracket, &mut text)
        .map_err(|_| InvalidPosixSyntax)?;
    let hir = regex_syntax::Parser::new()
        .parse(&text)
        .map_err(|_| InvalidPosixSyntax)?;
    let mut printed = String::new();
    regex_syntax::hir::print::Printer::new()
        .print(&hir, &mut printed)
        .map_err(|_| InvalidPosixSyntax)?;
    spend(budget, printed.len())?;
    // A class prints as one bracket; anything else (a single character prints bare) is wrapped.
    let class = if printed.starts_with('[') {
        parse_bracket(&printed)
    } else {
        parse_bracket(&format!("[{printed}]"))
    };
    class
        .map(|class| ClassSetItem::Bracketed(Box::new(class)))
        .ok_or(InvalidPosixSyntax)
}

/// `item` with a meta-character literal escaped: once the cased classes are taken out, a
/// verbatim `^` may lead the rest (`[[:upper:]^a]` must not become `[^a]`) and two `&`, `-` or
/// `~` may meet and read as a set operator.
fn escaped(item: &ClassSetItem) -> ClassSetItem {
    match item {
        ClassSetItem::Literal(lit) if regex_syntax::is_meta_character(lit.c) => {
            ClassSetItem::Literal(ast::Literal {
                kind: ast::LiteralKind::Meta,
                ..lit.clone()
            })
        }
        _ => item.clone(),
    }
}

/// The bracket class `text` (one of [`unicode_class`]'s constants).
fn parse_bracket(text: &str) -> Option<ast::ClassBracketed> {
    match &Parser::new().parse(text).ok()? {
        Ast::ClassBracketed(class) => Some(class.as_ref().clone()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{unicode_posix_classes, InvalidPosixSyntax};

    fn rewritten(pattern: &str) -> String {
        unicode_posix_classes(pattern, false).unwrap().into_owned()
    }

    #[test]
    fn rewrites_only_bracketed_posix_classes() {
        assert_eq!(rewritten("a[b]"), "a[b]");
        assert_eq!(rewritten("[[:alpha:]]"), r"[[\p{L}]]");
        assert_eq!(rewritten("[[:^digit:]x]"), r"[[^\p{Nd}]x]");
        assert_eq!(rewritten(r"[\[:alpha:]]"), r"[\[:alpha:]]");
        assert_eq!(rewritten("[[:ascii:]]"), "[[:ascii:]]");
    }

    #[test]
    fn long_set_operation_chain_does_not_overflow_the_stack() {
        let chain = format!("[a{}[.a.]]", "&&a".repeat(20_000));
        // Only that it returns: the parser may itself refuse a chain this long.
        let _ = unicode_posix_classes(&chain, true);
    }

    #[test]
    fn rejects_posix_syntax_pcre2_rejects() {
        for pattern in [
            "[:alpha:]",
            "[[:foo:]]",
            "[[:ALPHA:]]",
            "[[.a.]]",
            "[[=a=]]",
            "[[:alpha:]-z]",
            "[a[:upper:]-z]",
            "x(?:[[.a.]])+",
        ] {
            assert_eq!(
                unicode_posix_classes(pattern, false),
                Err(InvalidPosixSyntax),
                "{pattern}"
            );
        }
        for pattern in [
            r"[\[:alpha:]]",
            "[[:alpha:]-]",
            "[-[:alpha:]]",
            "[:a]",
            "[a:]",
            "[[:alpha:]]",
        ] {
            assert!(unicode_posix_classes(pattern, false).is_ok(), "{pattern}");
        }
    }

    /// Whether `pattern` matches `text`, compiled as REGEXTEST compiles it.
    fn matches(pattern: &str, text: &str, case_insensitive: bool) -> bool {
        let rewritten = unicode_posix_classes(pattern, case_insensitive).unwrap();
        regex_automata::meta::Regex::builder()
            .syntax(regex_automata::util::syntax::Config::new().case_insensitive(case_insensitive))
            .build(&rewritten)
            .unwrap()
            .is_match(text)
    }

    #[test]
    fn no_posix_class_folds_while_the_other_items_do() {
        // probes/regex-posix-classes-b A8-A11, -c A1-A26 (excel-graph, T-1989).
        for (pattern, text, expected) in [
            ("(?i)[[:upper:]]", "a", false),
            ("(?i)[[[:upper:]]]", "a", false),
            ("(?i)[[[:upper:]]]", "A", true),
            ("(?i)[^[:upper:]]", "a", true),
            ("(?i)[[:^upper:]]", "a", true),
            ("(?i)[[:^lower:]]", "A", true),
            ("(?i)[^[:lower:]]", "a", false),
            ("(?i)[^[:upper:]x]", "X", false),
            ("(?i)[^[:upper:]x]", "x", false),
            ("(?i)[^[:upper:]x]", "B", false),
            ("(?i)[^[:upper:]x]", "b", true),
            ("(?i)[[:^upper:]x]", "X", true),
            ("(?i)[[:lower:]a-c]", "B", true),
            ("(?i)[[:lower:]a-c]", "D", false),
            ("(?i)[[:upper:]&&[:alpha:]]", "a", false),
            ("(?i:[[:upper:]])", "a", false),
            ("(?i)(?-i)(?i)[^[:^upper:]]", "a", false),
            ("(?i)[[:ascii:]]", "\u{212A}", false),
            ("(?i)[[:ascii:]]", "\u{17F}", false),
            ("(?i)k", "\u{212A}", true),
            ("(?-i)[[:upper:]x]|(?i)[[:upper:]x]", "X", true),
            ("[[:upper:]x]", "X", true),
        ] {
            assert_eq!(
                matches(pattern, text, false),
                expected,
                "{pattern} on {text}"
            );
        }
        // The case_sensitivity argument 1 starts with (?i) on, which (?-i) turns off.
        assert!(!matches("[^[:upper:]x]", "X", true));
        assert!(matches("[^[:upper:]x]", "b", true));
        assert!(matches("[[:lower:]x]", "X", true));
        assert!(!matches("(?-i)[[:lower:]x]", "X", true));
    }

    #[test]
    fn brackets_under_case_insensitive_share_one_size_budget() {
        let one = "(?i)[[:upper:]\\pL\\pN\\pS\\pM]";
        assert!(unicode_posix_classes(one, false).is_ok());
        // Each folded `\pL` prints as kilobytes: many of them, in one bracket's `&&` chain or in
        // many brackets, spend the shared budget and are refused before the rest is folded.
        let chain = format!("(?i)[[:upper:]{}]", "&&\\pL".repeat(100));
        assert_eq!(
            unicode_posix_classes(&chain, false),
            Err(InvalidPosixSyntax)
        );
        let many = "[[:upper:]\\pL]".repeat(2_000);
        assert_eq!(unicode_posix_classes(&many, true), Err(InvalidPosixSyntax));
        // Without (?i) nothing folds, so nothing is spent.
        assert!(unicode_posix_classes(&many, false).is_ok());
    }
}
