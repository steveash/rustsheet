//! A regular [`Ast`] as a `regex_syntax` [`Hir`], for the automata of `regex_engine`
//! (SPEC-0013 section 2).

use regex_syntax::hir::{Capture, Class, Dot, Hir, Look, Repetition};

use super::{
    ast::{Anchor, Ast, RepeatKind},
    class::literal_class,
    parse::Invalid,
};

/// Which texts a pattern is compiled for, as `$` and `\Z` (at the end or before a final `\n`)
/// read on them (SPEC-0013 section 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::functions::text) enum FinalLf {
    /// Texts that hold a `\n` but do not end in one: `$` is `Look::End`, exact there.
    End,
    /// Texts whose only `\n`, if any, is the last character: `$` is `Look::EndLF` (the end, or
    /// before a `\n`), exact there.
    EndLf,
    /// Texts with a `\n` before the last character, where no `Look` is exact: a regular pattern
    /// with `$` or `\Z` runs on the backtracker's memoized mode (T-2355).
    InnerLf,
}

/// `ast` as a `Hir`, `$` and `\Z` read for `final_lf` texts ([`FinalLf::InnerLf`] reads as
/// `EndLf`: `translate` sends a pattern holding them to the backtracker there); `Invalid` for a
/// construct only the backtracker runs. The recursion is bounded by the parser's nest limit.
pub(super) fn to_hir(ast: &Ast, final_lf: FinalLf) -> Result<Hir, Invalid> {
    Ok(match ast {
        Ast::Empty => Hir::empty(),
        Ast::Literal { c, fold } => {
            let class = literal_class(*c, *fold);
            match class.literal() {
                Some(bytes) => Hir::literal(bytes),
                None => Hir::class(Class::Unicode(class)),
            }
        }
        Ast::Class(class) => Hir::class(Class::Unicode(class.clone())),
        Ast::Dot { dot_all } => Hir::dot(if *dot_all {
            Dot::AnyChar
        } else {
            Dot::AnyCharExceptLF
        }),
        Ast::Anchor(anchor) => Hir::look(match anchor {
            Anchor::Start => Look::Start,
            Anchor::StartLine => Look::StartLF,
            Anchor::EndLine => Look::EndLF,
            Anchor::End => Look::End,
            Anchor::EndOrFinalLf => match final_lf {
                FinalLf::End => Look::End,
                FinalLf::EndLf | FinalLf::InnerLf => Look::EndLF,
            },
            Anchor::WordBoundary => Look::WordUnicode,
            Anchor::NotWordBoundary => Look::WordUnicodeNegate,
            Anchor::SearchStart => return Err(Invalid),
        }),
        Ast::Capture { index, name, ast } => Hir::capture(Capture {
            index: *index,
            name: name.clone(),
            sub: Box::new(to_hir(ast, final_lf)?),
        }),
        Ast::Concat(asts) => Hir::concat(
            asts.iter()
                .map(|ast| to_hir(ast, final_lf))
                .collect::<Result<_, _>>()?,
        ),
        Ast::Alternation(asts) => Hir::alternation(
            asts.iter()
                .map(|ast| to_hir(ast, final_lf))
                .collect::<Result<_, _>>()?,
        ),
        Ast::Repeat {
            min,
            max,
            kind,
            ast,
        } => {
            let greedy = match kind {
                RepeatKind::Greedy => true,
                RepeatKind::Lazy => false,
                RepeatKind::Possessive => return Err(Invalid),
            };
            Hir::repetition(Repetition {
                min: *min,
                max: *max,
                greedy,
                sub: Box::new(to_hir(ast, final_lf)?),
            })
        }
        Ast::Backref { .. }
        | Ast::LookAround { .. }
        | Ast::Atomic(_)
        | Ast::Conditional { .. }
        | Ast::KeepOut
        | Ast::Grapheme => return Err(Invalid),
    })
}
