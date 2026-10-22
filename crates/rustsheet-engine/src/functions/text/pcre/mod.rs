//! The PCRE2 dialect of the REGEX* functions (SPEC-0013): one parser for every pattern, regular
//! patterns translated to a `Hir` for the automata of `regex_engine`, the others (backreferences,
//! atomic and possessive groups, `\G`) compiled to a [`Program`] for the backtracker
//! ([`Backtracker`], T-1900). Lookaround, conditionals and `\K` parse and are `#VALUE!` until
//! T-2354.

mod ast;
mod backtrack;
mod class;
mod hir;
mod parse;
mod program;
#[cfg(test)]
mod tests;

pub(in crate::functions::text) use backtrack::{BacktrackCache, Backtracker};
pub(in crate::functions::text) use hir::FinalLf;
pub(in crate::functions::text) use program::Program;
use regex_syntax::hir::Hir;

/// A pattern this dialect cannot run: invalid in PCRE2 or unknown to us.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Unsupported;

/// A pattern translated for one of the two engines.
pub(super) struct Translated {
    pub(super) form: Form,
    /// Code points its case folding walked: charged to the compile budget (T-2138).
    pub(super) fold_cost: usize,
}

/// Which engine a pattern runs on.
pub(super) enum Form {
    /// A regular pattern, for the automata.
    Automata(Hir),
    /// Any other, compiled for the backtracker by [`Backtrack::compile`].
    Backtrack(Backtrack),
}

/// A parsed pattern only the backtracker runs.
pub(super) struct Backtrack(ast::Ast);

/// Why [`Backtrack::compile`] failed.
pub(super) enum ProgramFailure {
    /// The program passes the size limit.
    TooBig,
    /// It holds a construct the backtracker does not run yet.
    Unsupported,
}

impl Backtrack {
    /// The program, within `size_limit` bytes.
    pub(super) fn compile(&self, size_limit: usize) -> Result<Program, ProgramFailure> {
        program::compile(&self.0, size_limit).map_err(|error| match error {
            program::CompileError::TooBig => ProgramFailure::TooBig,
            program::CompileError::Unsupported => ProgramFailure::Unsupported,
        })
    }
}

/// `pattern` for its engine, ignoring case when `caseless` (`(?i)` and `(?-i)` override it), with
/// `$` and `\Z` translated for the automata per `final_lf` (the backtracker reads them exactly on
/// every text). Its case folding stops past `fold_budget` code points, with a `fold_cost` of
/// `usize::MAX`.
pub(super) fn translate(
    pattern: &str,
    caseless: bool,
    final_lf: FinalLf,
    fold_budget: usize,
) -> Result<Translated, Unsupported> {
    let parsed = parse::parse(pattern, caseless, fold_budget).map_err(|_| Unsupported)?;
    let form = if parsed.ast.is_regular() {
        Form::Automata(hir::to_hir(&parsed.ast, final_lf).map_err(|_| Unsupported)?)
    } else {
        Form::Backtrack(Backtrack(parsed.ast))
    };
    Ok(Translated {
        form,
        fold_cost: parsed.fold_cost,
    })
}

/// Which translation of `$` searching `text` needs (SPEC-0013 section 2): `EndLf` unless `text`
/// holds a `\n` before its last character, where neither is exact and `End` keeps today's
/// meaning until T-2355's memoized mode.
pub(super) fn final_lf_for(text: &str) -> FinalLf {
    let body = text.strip_suffix('\n').unwrap_or(text);
    if body.contains('\n') {
        FinalLf::End
    } else {
        FinalLf::EndLf
    }
}
