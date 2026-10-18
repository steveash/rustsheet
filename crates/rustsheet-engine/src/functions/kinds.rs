//! Properties of function kinds beyond the per-function tables of `mod.rs`, among them the
//! internal kinds of SPEC-0012: a legacy formula's N and T (`NLegacy`, `TLegacy`).

use super::Function;

/// How `to_rc_format` stores the internal kinds. No other text shows them: the display, English,
/// FORMULATEXT and xlsx texts print N and T. A typed or imported (A1) call of the spelling is
/// plain N or T, so no A1 formula is stored with it (SPEC-0012 "Storage").
const LEGACY_SPELLINGS: [(&str, Function); 2] = [
    ("_RS_LEGACY_N", Function::NLegacy),
    ("_RS_LEGACY_T", Function::TLegacy),
];

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

    /// The kind a call spelled `name` is, if it is the stored spelling of an internal kind: that
    /// kind when `stored` (R1C1 text), else plain N or T. The one matcher of both modes, exact
    /// and case-sensitive, so a variant (`_rs_legacy_n`, `_xlfn._RS_LEGACY_N`) is no marker in
    /// either.
    pub(crate) fn from_legacy_spelling(name: &str, stored: bool) -> Option<Function> {
        let (_, kind) = LEGACY_SPELLINGS
            .iter()
            .find(|(spelling, _)| *spelling == name)?;
        Some(match (kind, stored) {
            (kind, true) => kind.clone(),
            (Function::NLegacy, false) => Function::N,
            _ => Function::T,
        })
    }

    /// The stored spelling of an internal kind, `None` for every other kind.
    pub(crate) fn legacy_spelling(&self) -> Option<&'static str> {
        LEGACY_SPELLINGS
            .iter()
            .find(|(_, kind)| kind == self)
            .map(|(spelling, _)| *spelling)
    }
}
