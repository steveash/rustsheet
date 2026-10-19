//! POSIX bracket classes (`[[:alpha:]]`) as Excel's PCRE2 (UCP mode) reads them.
//!
//! The regex crate's POSIX classes are ASCII-only; Excel's are Unicode (probes/regex-posix-classes,
//! docs/excel-semantics/text.md "REGEX* POSIX classes"). [`unicode_posix_classes`] rewrites every
//! POSIX class item of a parsed pattern into the Unicode class Excel matches and prints the
//! pattern back. Escapes, nesting and set operations are the parser's, so nothing is rewritten
//! inside an escaped `\[` or a literal. A pattern that does not parse is returned unchanged: it
//! fails to compile the same way.
//!
//! `[:upper:]` and `[:lower:]` ignore case-insensitive matching (`(?i)[[:upper:]]` does not match
//! `a`), so a non-negated top-level bracket that holds them becomes an alternation of the rest of
//! the bracket and `(?-i:\p{Lu})` / `(?-i:\p{Ll})`.

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
        // White_Space: NEL, NBSP, U+2028 and U+3000 are spaces, ZWSP is not.
        ClassAsciiKind::Space => r"[\p{White_Space}]",
        ClassAsciiKind::Upper => r"[\p{Lu}]",
        // Letters, all numbers (`²`, `½`, `Ⅳ`), non-spacing marks and connectors.
        ClassAsciiKind::Word => r"[\p{L}\p{N}\p{Mn}\p{Pc}]",
        ClassAsciiKind::Xdigit => r"[0-9A-Fa-f\x{FF10}-\x{FF19}\x{FF21}-\x{FF26}\x{FF41}-\x{FF46}]",
        // ASCII is ASCII in both.
        ClassAsciiKind::Ascii => return None,
    })
}

/// `pattern` with its POSIX classes rewritten to Excel's Unicode ones; borrowed when it has none
/// or does not parse.
pub(super) fn unicode_posix_classes(pattern: &str) -> Cow<'_, str> {
    if !pattern.contains("[:") {
        return Cow::Borrowed(pattern);
    }
    let Ok(mut ast) = Parser::new().parse(pattern) else {
        return Cow::Borrowed(pattern);
    };
    if !rewrite(&mut ast) {
        return Cow::Borrowed(pattern);
    }
    let mut out = String::with_capacity(pattern.len() * 2);
    // Printing to a String cannot fail.
    if Printer::new().print(&ast, &mut out).is_err() {
        return Cow::Borrowed(pattern);
    }
    Cow::Owned(out)
}

/// Rewrites the POSIX classes under `ast`; whether there were any. The recursion is bounded by
/// the parser's nesting limit.
fn rewrite(ast: &mut Ast) -> bool {
    match ast {
        Ast::ClassBracketed(class) => {
            if let Some(group) = case_sensitive_split(class) {
                *ast = group;
                return true;
            }
            rewrite_set(&mut class.kind)
        }
        Ast::Repetition(rep) => rewrite(&mut rep.ast),
        Ast::Group(group) => rewrite(&mut group.ast),
        Ast::Alternation(alt) => alt.asts.iter_mut().fold(false, |any, a| rewrite(a) | any),
        Ast::Concat(cat) => cat.asts.iter_mut().fold(false, |any, a| rewrite(a) | any),
        Ast::Empty(_)
        | Ast::Flags(_)
        | Ast::Literal(_)
        | Ast::Dot(_)
        | Ast::Assertion(_)
        | Ast::ClassUnicode(_)
        | Ast::ClassPerl(_) => false,
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

/// A non-negated bracket of plain items holding `[:upper:]` or `[:lower:]`, as
/// `(?:[rest]|(?-i:\p{Lu}))`, so case-insensitive matching leaves those two alone (Excel:
/// `REGEXTEST("a","(?i)[[:upper:]]")` is FALSE). `None` for any other bracket.
fn case_sensitive_split(class: &ast::ClassBracketed) -> Option<Ast> {
    if class.negated {
        return None;
    }
    let items: Vec<&ClassSetItem> = match &class.kind {
        ClassSet::Item(ClassSetItem::Union(union)) => union.items.iter().collect(),
        ClassSet::Item(item) => vec![item],
        ClassSet::BinaryOp(_) => return None,
    };
    let cased = |item: &ClassSetItem| {
        matches!(item, ClassSetItem::Ascii(a)
            if !a.negated && matches!(a.kind, ClassAsciiKind::Upper | ClassAsciiKind::Lower))
    };
    if !items.iter().any(|i| cased(i)) {
        return None;
    }
    let mut alternatives = Vec::new();
    let mut rest = String::new();
    for item in items {
        match item {
            ClassSetItem::Ascii(a) if cased(item) => alternatives.push(match a.kind {
                ClassAsciiKind::Upper => r"(?-i:\p{Lu})",
                _ => r"(?-i:\p{Ll})",
            }),
            _ => {
                let mut one = ClassSetItem::Union(ast::ClassSetUnion {
                    span: class.span,
                    items: vec![escaped(item)],
                });
                rewrite_item(&mut one);
                let bracket = Ast::class_bracketed(ast::ClassBracketed {
                    span: class.span,
                    negated: false,
                    kind: ClassSet::Item(one),
                });
                let mut printed = String::new();
                Printer::new().print(&bracket, &mut printed).ok()?;
                // `[x]` → `x`, so the rest prints as one bracket.
                rest.push_str(printed.get(1..printed.len().checked_sub(1)?)?);
            }
        }
    }
    let mut text = String::from("(?:");
    if !rest.is_empty() {
        text.push('[');
        text.push_str(&rest);
        text.push_str("]|");
    }
    text.push_str(&alternatives.join("|"));
    text.push(')');
    Parser::new().parse(&text).ok()
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
    use super::unicode_posix_classes;

    #[test]
    fn rewrites_only_bracketed_posix_classes() {
        assert_eq!(unicode_posix_classes("a[b]"), "a[b]");
        assert_eq!(unicode_posix_classes("[[:alpha:]]"), r"[[\p{L}]]");
        assert_eq!(unicode_posix_classes("[[:^digit:]x]"), r"[[^\p{Nd}]x]");
        assert_eq!(unicode_posix_classes(r"[\[:alpha:]]"), r"[\[:alpha:]]");
        assert_eq!(unicode_posix_classes("[[:ascii:]]"), "[[:ascii:]]");
        assert_eq!(unicode_posix_classes("[[:foo:]]"), "[[:foo:]]");
    }

    #[test]
    fn upper_and_lower_stay_case_sensitive() {
        assert_eq!(
            unicode_posix_classes("[x[:upper:]]"),
            r"(?:[x]|(?-i:\p{Lu}))"
        );
        assert_eq!(unicode_posix_classes("[[:lower:]]+"), r"(?:(?-i:\p{Ll}))+");
        assert_eq!(unicode_posix_classes("[^[:upper:]]"), r"[^[\p{Lu}]]");
        assert_eq!(
            unicode_posix_classes("[[:upper:]^a]"),
            r"(?:[\^a]|(?-i:\p{Lu}))"
        );
        assert_eq!(
            unicode_posix_classes(r"[&[:lower:]\&]"),
            r"(?:[\&\&]|(?-i:\p{Ll}))"
        );
    }
}
