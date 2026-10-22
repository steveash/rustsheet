//! The PCRE2 dialect of the REGEX* functions (SPEC-0013): one parser for every pattern, regular
//! patterns translated to a `Hir` for the automata of `regex_engine`. The backtracker for the
//! other constructs (backreferences, lookaround, atomic and possessive groups, `\K`,
//! conditionals, `\G`) is T-1900; until then they parse and are `#VALUE!`.

mod ast;
mod class;
mod hir;
mod parse;
#[cfg(test)]
mod tests;

pub(in crate::functions::text) use hir::FinalLf;
use regex_syntax::hir::Hir;

/// A pattern this dialect cannot run: invalid in PCRE2, unknown to us, or needing the
/// backtracker (T-1900).
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Unsupported;

/// A pattern translated for the automata.
pub(super) struct Translated {
    pub(super) hir: Hir,
    /// Code points its case folding walked: charged to the compile budget (T-2138).
    pub(super) fold_cost: usize,
}

/// `pattern` as a `Hir`, ignoring case when `caseless` (`(?i)` and `(?-i)` override it), with
/// `$` and `\Z` translated per `final_lf`. Its case folding stops past `fold_budget` code points,
/// with a `fold_cost` of `usize::MAX`.
pub(super) fn translate(
    pattern: &str,
    caseless: bool,
    final_lf: FinalLf,
    fold_budget: usize,
) -> Result<Translated, Unsupported> {
    let parsed = parse::parse(pattern, caseless, fold_budget).map_err(|_| Unsupported)?;
    if !parsed.ast.is_regular() {
        return Err(Unsupported);
    }
    let hir = hir::to_hir(&parsed.ast, final_lf).map_err(|_| Unsupported)?;
    Ok(Translated {
        hir,
        fold_cost: parsed.fold_cost,
    })
}

/// Which translation of `$` searching `text` needs (SPEC-0013 section 2): `EndLf` unless `text`
/// holds a `\n` before its last character, where neither is exact and `End` keeps today's
/// meaning until T-1900's memoized mode.
pub(super) fn final_lf_for(text: &str) -> FinalLf {
    let body = text.strip_suffix('\n').unwrap_or(text);
    if body.contains('\n') {
        FinalLf::End
    } else {
        FinalLf::EndLf
    }
}
