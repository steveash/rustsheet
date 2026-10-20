//! Properties of function kinds beyond the per-function tables of `mod.rs`, among them the
//! internal kinds of SPEC-0012: a legacy formula's N and T (`NLegacy`, `TLegacy`).

use super::Function;

/// How `to_rc_format` stores the internal kinds. No other text shows them: the display, English,
/// FORMULATEXT and xlsx texts print N and T. Each spelling is an A1 cell, so typed or imported (A1)
/// text never holds it as a call name: there `NLG1(1)` is a call of that cell, `#REF!`. Nor can a
/// workbook name of that spelling be called: Excel opens a workbook defining a LAMBDA name `NLG1`
/// or `TLG1`, but `NLG1(1)` is `#REF!` and `ISREF(NLG1)` TRUE (probes/legacy-marker-spelling-
/// name-nlg1, -tlg1, T-1936), so the marker never shadows a name (SPEC-0012 "Storage").
const LEGACY_SPELLINGS: [(&str, Function); 2] =
    [("NLG1", Function::NLegacy), ("TLG1", Function::TLegacy)];

impl Function {
    /// The internal kinds. They are not in the lookup table, so no name parses to them and
    /// `Function::into_iter` does not list them; `function_iterator` checks that the enum is the
    /// lookup table plus this list.
    pub const INTERNAL: [Function; 2] = [Function::NLegacy, Function::TLegacy];

    /// Whether the function's result is a reference it builds from its arguments (INDIRECT, OFFSET).
    pub(crate) fn returns_reference(&self) -> bool {
        matches!(self, Function::Indirect | Function::Offset)
    }

    /// N or T, legacy or not: the calls that read a reference argument as its top-left cell.
    pub(crate) fn is_n_or_t(&self) -> bool {
        matches!(
            self,
            Function::N | Function::T | Function::NLegacy | Function::TLegacy
        )
    }

    /// The internal kind of a legacy formula's N or T, which reads an array argument as
    /// `#VALUE!` (SPEC-0012 "Placement"); every other kind, the internal ones included, is
    /// itself.
    pub(crate) fn legacy_reading(&self) -> Function {
        match self {
            Function::N => Function::NLegacy,
            Function::T => Function::TLegacy,
            other => other.clone(),
        }
    }

    /// The internal kind a call spelled `name` in stored (R1C1) text is, if any. The one matcher,
    /// exact and case-sensitive, so a variant (`nlg1`, `_xlfn.NLG1`) is no marker. A1 text has no
    /// marker. T-1851's spelling `_RS_LEGACY_N(` is no longer read: stored text cannot tell it from
    /// a call of a name of that spelling, which Excel calls (T-1936).
    pub(crate) fn from_legacy_spelling(name: &str) -> Option<Function> {
        LEGACY_SPELLINGS
            .iter()
            .find(|(spelling, _)| *spelling == name)
            .map(|(_, kind)| kind.clone())
    }

    /// The stored spelling of an internal kind, `None` for every other kind.
    pub(crate) fn legacy_spelling(&self) -> Option<&'static str> {
        LEGACY_SPELLINGS
            .iter()
            .find(|(_, kind)| kind == self)
            .map(|(spelling, _)| *spelling)
    }
}
