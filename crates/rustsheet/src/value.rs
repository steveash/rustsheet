use std::sync::Arc;

use rustsheet_engine::expressions::token::Error as EngineError;

/// The kind of an Excel error value (`#DIV/0!`, `#N/A`, ...).
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[non_exhaustive]
pub enum ErrorKind {
    /// `#NULL!`
    Null,
    /// `#DIV/0!`
    Div0,
    /// `#VALUE!`
    Value,
    /// `#REF!`
    Ref,
    /// `#NAME?`
    Name,
    /// `#NUM!`
    Num,
    /// `#N/A`
    NA,
    /// `#SPILL!`
    Spill,
    /// `#CALC!`
    Calc,
    /// `#CIRC!` (engine-specific: Excel reports a circular reference as 0 or a warning)
    Circ,
    /// `#N/IMPL!` (engine-specific: feature not implemented)
    NotImplemented,
    /// `#ERROR!` (engine-specific: the formula could not be processed). Not Excel's `#UNKNOWN!`.
    Unknown,
}

impl ErrorKind {
    /// The kind for an engine error. Exhaustive on purpose: a new engine error must be mapped.
    pub(crate) fn from_engine(error: &EngineError) -> Self {
        match error {
            EngineError::NULL => Self::Null,
            EngineError::DIV => Self::Div0,
            EngineError::VALUE => Self::Value,
            EngineError::REF => Self::Ref,
            EngineError::NAME => Self::Name,
            EngineError::NUM => Self::Num,
            EngineError::NA => Self::NA,
            EngineError::SPILL => Self::Spill,
            EngineError::CALC => Self::Calc,
            EngineError::CIRC => Self::Circ,
            EngineError::NIMPL => Self::NotImplemented,
            EngineError::ERROR => Self::Unknown,
        }
    }

    /// The engine error for this kind (inverse of [`ErrorKind::from_engine`]).
    pub(crate) fn to_engine(self) -> EngineError {
        match self {
            Self::Null => EngineError::NULL,
            Self::Div0 => EngineError::DIV,
            Self::Value => EngineError::VALUE,
            Self::Ref => EngineError::REF,
            Self::Name => EngineError::NAME,
            Self::Num => EngineError::NUM,
            Self::NA => EngineError::NA,
            Self::Spill => EngineError::SPILL,
            Self::Calc => EngineError::CALC,
            Self::Circ => EngineError::CIRC,
            Self::NotImplemented => EngineError::NIMPL,
            Self::Unknown => EngineError::ERROR,
        }
    }

    /// The en-US literal as stored in xlsx files and accepted in formulas.
    ///
    /// A static mirror of the engine's literal table; `tests::literals_match_engine` keeps the
    /// two in sync.
    #[must_use]
    pub fn literal(self) -> &'static str {
        match self {
            Self::Null => "#NULL!",
            Self::Div0 => "#DIV/0!",
            Self::Value => "#VALUE!",
            Self::Ref => "#REF!",
            Self::Name => "#NAME?",
            Self::Num => "#NUM!",
            Self::NA => "#N/A",
            Self::Spill => "#SPILL!",
            Self::Calc => "#CALC!",
            Self::Circ => "#CIRC!",
            Self::NotImplemented => "#N/IMPL!",
            Self::Unknown => "#ERROR!",
        }
    }

    /// Parses an en-US error literal (case-insensitive, as the engine does).
    #[must_use]
    pub fn from_literal(literal: &str) -> Option<Self> {
        let language = rustsheet_engine::language::get_default_language();
        rustsheet_engine::expressions::token::get_error_by_name(&literal.to_uppercase(), language)
            .as_ref()
            .map(Self::from_engine)
    }

    /// The number `ERROR.TYPE` returns for this error.
    ///
    /// Always `Some` today; `Option` leaves room for kinds without a code.
    #[must_use]
    pub fn code(self) -> Option<u8> {
        Some(self.to_engine().error_type_code())
    }
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.literal())
    }
}

/// An Excel error *value* (a cell result), never a Rust `Err`.
#[derive(Clone, Debug, PartialEq)]
pub struct XlError {
    /// Which error.
    pub kind: ErrorKind,
    /// The engine's explanation (`FormulaValue::Error.message`), when it gave one.
    pub detail: Option<Arc<str>>,
}

impl XlError {
    /// An error with no detail message.
    #[must_use]
    pub fn new(kind: ErrorKind) -> Self {
        Self { kind, detail: None }
    }

    /// An error with the engine's detail message.
    #[must_use]
    pub fn with_detail(kind: ErrorKind, detail: impl Into<Arc<str>>) -> Self {
        Self {
            kind,
            detail: Some(detail.into()),
        }
    }
}

impl From<ErrorKind> for XlError {
    fn from(kind: ErrorKind) -> Self {
        Self::new(kind)
    }
}

impl std::fmt::Display for XlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.kind.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [ErrorKind; 12] = [
        ErrorKind::Null,
        ErrorKind::Div0,
        ErrorKind::Value,
        ErrorKind::Ref,
        ErrorKind::Name,
        ErrorKind::Num,
        ErrorKind::NA,
        ErrorKind::Spill,
        ErrorKind::Calc,
        ErrorKind::Circ,
        ErrorKind::NotImplemented,
        ErrorKind::Unknown,
    ];

    #[test]
    fn every_kind_round_trips_through_engine_and_literal() {
        for kind in ALL {
            assert_eq!(ErrorKind::from_engine(&kind.to_engine()), kind);
            assert_eq!(ErrorKind::from_literal(kind.literal()), Some(kind));
            assert_eq!(
                ErrorKind::from_literal(&kind.literal().to_lowercase()),
                Some(kind)
            );
        }
        assert_eq!(ErrorKind::from_literal("#BOGUS!"), None);
    }

    #[test]
    fn literals_match_engine() {
        for kind in ALL {
            let engine = kind.to_engine().to_string();
            // The engine prints `#N/IMPL` without the `!` its parser requires.
            assert_eq!(
                kind.literal().trim_end_matches('!'),
                engine.trim_end_matches('!'),
                "{kind:?}"
            );
        }
    }

    #[test]
    fn codes_follow_error_type() {
        // Excel's ERROR.TYPE numbers; 101/102/104 are engine-specific.
        let expected = [1, 2, 3, 4, 5, 6, 7, 9, 14, 104, 102, 101];
        for (kind, code) in ALL.into_iter().zip(expected) {
            assert_eq!(kind.code(), Some(code), "{kind:?}");
        }
    }

    #[test]
    fn xl_error_displays_literal() {
        let e = XlError::with_detail(ErrorKind::Div0, "Divide by zero");
        assert_eq!(e.to_string(), "#DIV/0!");
        assert_eq!(XlError::from(ErrorKind::NA), XlError::new(ErrorKind::NA));
    }
}
