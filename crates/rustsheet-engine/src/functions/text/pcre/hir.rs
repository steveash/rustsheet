//! A regular [`Ast`] as a `regex_syntax` [`Hir`], for the automata of `regex_engine`
//! (SPEC-0013 section 2).

use regex_syntax::hir::{Capture, Class, Dot, Hir, Look, Repetition};

use super::{
    ast::{Anchor, Ast, RepeatKind},
    class::literal_class,
    parse::Invalid,
};

/// Which texts a pattern is compiled for, as `$` and `\Z` (at the end or before a final LF, CR
/// or CRLF) and `(?m)$` read on them (SPEC-0013 section 2).
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
    /// Texts holding a CR, where `$`, `\Z` and `(?m)$` match before it (and `(?m)$` between a CR
    /// and its LF), which no `Look` gives: a regular pattern holding one of them runs on the
    /// memoized mode (T-2364).
    Cr,
}

impl FinalLf {
    /// The anchors no `Look` reads exactly on texts of this kind: `$` and `\Z` on an
    /// [`FinalLf::InnerLf`] text (T-2355), and `(?m)$` too on a [`FinalLf::Cr`] one (T-2364).
    pub(super) fn inexact_anchors(self) -> &'static [Anchor] {
        match self {
            FinalLf::End | FinalLf::EndLf => &[],
            FinalLf::InnerLf => &[Anchor::EndOrFinalLf],
            FinalLf::Cr => &[Anchor::EndOrFinalLf, Anchor::EndLine],
        }
    }
}

/// `ast` as a `Hir`, `$` and `\Z` read for `final_lf` texts ([`FinalLf::InnerLf`] and
/// [`FinalLf::Cr`] read as `EndLf`: `translate` sends a pattern holding them to the backtracker
/// there); `Invalid` for a construct only the backtracker runs. `.` and the `(?m)` anchors take
/// the CRLF forms, exact on every text but for `(?m)$` before the LF of a CRLF, a [`FinalLf::Cr`]
/// text (Excel's ANYCRLF convention, T-2364). The recursion is bounded by the parser's nest
/// limit.
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
            Dot::AnyCharExceptCRLF
        }),
        Ast::Anchor(anchor) => Hir::look(match anchor {
            Anchor::Start => Look::Start,
            Anchor::StartLine => Look::StartCRLF,
            Anchor::EndLine => Look::EndCRLF,
            Anchor::End => Look::End,
            Anchor::EndOrFinalLf => match final_lf {
                FinalLf::End => Look::End,
                FinalLf::EndLf | FinalLf::InnerLf | FinalLf::Cr => Look::EndLF,
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
