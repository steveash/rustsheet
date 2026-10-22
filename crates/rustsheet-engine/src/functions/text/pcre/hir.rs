//! A regular [`Ast`] as a `regex_syntax` [`Hir`], for the automata of `regex_engine`
//! (SPEC-0013 section 2).

use regex_syntax::hir::{
    Capture, Class, ClassUnicode, ClassUnicodeRange, Dot, Hir, Look, Repetition,
};

use super::{
    ast::{Anchor, Ast, RepeatKind},
    parse::Invalid,
};

/// How `$` and `\Z` translate: at the end, or also before a final `\n`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::functions::text) enum FinalLf {
    /// `Look::End`: exact on a text with no `\n`, and today's gap on one holding a `\n` before
    /// its last character (the backtracker's memoized mode closes it, T-1900).
    End,
    /// `Look::EndLF`: exact on a text whose only `\n` is its last character.
    EndLf,
}

/// `ast` as a `Hir`; `Invalid` for a construct only the backtracker runs. The recursion is
/// bounded by the parser's nest limit.
pub(super) fn to_hir(ast: &Ast, final_lf: FinalLf) -> Result<Hir, Invalid> {
    Ok(match ast {
        Ast::Empty => Hir::empty(),
        Ast::Literal { c, fold } => {
            let mut class = ClassUnicode::new([ClassUnicodeRange::new(*c, *c)]);
            if *fold {
                class.case_fold_simple();
            }
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
                FinalLf::EndLf => Look::EndLF,
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
        | Ast::Unimplemented => return Err(Invalid),
    })
}
