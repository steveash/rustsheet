use std::fmt;

use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::language::Language;

use super::{lexer::LexerError, types::ParsedReference};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum OpCompare {
    LessThan,
    GreaterThan,
    Equal,
    LessOrEqualThan,
    GreaterOrEqualThan,
    NonEqual,
}

impl fmt::Display for OpCompare {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OpCompare::LessThan => write!(fmt, "<"),
            OpCompare::GreaterThan => write!(fmt, ">"),
            OpCompare::Equal => write!(fmt, "="),
            OpCompare::LessOrEqualThan => write!(fmt, "<="),
            OpCompare::GreaterOrEqualThan => write!(fmt, ">="),
            OpCompare::NonEqual => write!(fmt, "<>"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum OpUnary {
    Minus,
    Percentage,
}

impl fmt::Display for OpUnary {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OpUnary::Minus => write!(fmt, "-"),
            OpUnary::Percentage => write!(fmt, "%"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum OpSum {
    Add,
    Minus,
}

impl fmt::Display for OpSum {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OpSum::Add => write!(fmt, "+"),
            OpSum::Minus => write!(fmt, "-"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum OpProduct {
    Times,
    Divide,
}

impl fmt::Display for OpProduct {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OpProduct::Times => write!(fmt, "*"),
            OpProduct::Divide => write!(fmt, "/"),
        }
    }
}

/// List of `errors`
/// Note that "#ERROR!" and "#N/IMPL!" are not part of the xlsx standard
///  * "#ERROR!" means there was an error processing the formula (for instance "=A1+")
///  * "#N/IMPL!" means the formula or feature in Excel but has not been implemented in IronCalc
///    Note that they are serialized/deserialized by index
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub enum Error {
    REF,
    NAME,
    VALUE,
    DIV,
    NA,
    NUM,
    ERROR,
    NIMPL,
    SPILL,
    CALC,
    CIRC,
    NULL,
    // Excel's newer error kinds (SPEC-0003). Appended after `NULL` so the existing variant
    // indices, and with them every serialized model, stay valid.
    /// `#GETTING_DATA` (ERROR.TYPE 8). Excel saves it to xlsx as `#N/A` (`stored_literal`).
    GETTINGDATA,
    /// `#CONNECT!` (ERROR.TYPE 10).
    CONNECT,
    /// `#BLOCKED!` (ERROR.TYPE 11).
    BLOCKED,
    /// `#FIELD!` (ERROR.TYPE 13).
    FIELD,
    /// `#BUSY!` (ERROR.TYPE 0). Only a stored constant: Excel loads `<c t="e"><v>#BUSY!</v></c>`
    /// but rejects `=#BUSY!`, and no formula is known to produce it (SPEC-0003, "Stored `#BUSY!`").
    BUSY,
}

/// The en-US formula literal for [`Error::GETTINGDATA`]. Of Excel's newer errors only this one
/// parses in a formula (probes/error-literals-new A1, A11; T-0175). It is a constant rather than a
/// `Language` field because `language.bin` is a bitcode blob (SPEC-0003).
const GETTING_DATA_LITERAL: &str = "#GETTING_DATA";

/// Whether `text` starts with the `#GETTING_DATA` literal, ignoring ASCII case
/// (`=#getting_data` parses in Excel; probes/error-literals-new A11).
pub(crate) fn starts_with_getting_data(text: &str) -> bool {
    text.get(..GETTING_DATA_LITERAL.len())
        .is_some_and(|head| head.eq_ignore_ascii_case(GETTING_DATA_LITERAL))
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(self.english_literal())
    }
}
impl Error {
    /// Every kind, in declaration order; the first twelve are `error_index`'s order. For tests
    /// that must cover each kind.
    #[cfg(test)]
    pub(crate) const ALL: [Error; 17] = [
        Error::REF,
        Error::NAME,
        Error::VALUE,
        Error::DIV,
        Error::NA,
        Error::NUM,
        Error::ERROR,
        Error::NIMPL,
        Error::SPILL,
        Error::CALC,
        Error::CIRC,
        Error::NULL,
        Error::GETTINGDATA,
        Error::CONNECT,
        Error::BLOCKED,
        Error::FIELD,
        Error::BUSY,
    ];

    /// The en-US literal of this error, as `Display` writes it.
    fn english_literal(&self) -> &'static str {
        match self {
            Error::NULL => "#NULL!",
            Error::REF => "#REF!",
            Error::NAME => "#NAME?",
            Error::VALUE => "#VALUE!",
            Error::DIV => "#DIV/0!",
            Error::NA => "#N/A",
            Error::NUM => "#NUM!",
            Error::ERROR => "#ERROR!",
            Error::NIMPL => "#N/IMPL",
            Error::SPILL => "#SPILL!",
            Error::CALC => "#CALC!",
            Error::CIRC => "#CIRC!",
            Error::GETTINGDATA => GETTING_DATA_LITERAL,
            Error::CONNECT => "#CONNECT!",
            Error::BLOCKED => "#BLOCKED!",
            Error::FIELD => "#FIELD!",
            Error::BUSY => "#BUSY!",
        }
    }

    /// The number `ERROR.TYPE` returns for this error. The single source for that table; the
    /// `rustsheet` facade's `ErrorKind::code` is built on it.
    pub fn error_type_code(&self) -> u8 {
        match self {
            Error::NULL => 1,
            Error::DIV => 2,
            Error::VALUE => 3,
            Error::REF => 4,
            Error::NAME => 5,
            Error::NUM => 6,
            Error::NA => 7,
            Error::SPILL => 9,
            Error::GETTINGDATA => 8,
            Error::CONNECT => 10,
            Error::BLOCKED => 11,
            Error::FIELD => 13,
            Error::CALC => 14,
            // A real code, not "no code": ERROR.TYPE of a stored #BUSY! is 0, and
            // ERROR.TYPE(A1)+100 is 100 (probes/error-stored-constants S!B1, H1).
            Error::BUSY => 0,
            // Excel's #UNKNOWN! (12) and #EXTERNAL! (19) have no kind here: there is no evidence
            // of a producer. The rest are RustSheet/IronCalc specific.
            Error::ERROR => 101,
            Error::NIMPL => 102,
            Error::CIRC => 104,
        }
    }

    pub fn to_localized_error_string(&self, language: &Language) -> String {
        match self {
            Error::NULL => language.errors.null.to_string(),
            Error::REF => language.errors.r#ref.to_string(),
            Error::NAME => language.errors.name.to_string(),
            Error::VALUE => language.errors.value.to_string(),
            Error::DIV => language.errors.div.to_string(),
            Error::NA => language.errors.na.to_string(),
            Error::NUM => language.errors.num.to_string(),
            Error::ERROR => language.errors.error.to_string(),
            Error::NIMPL => language.errors.nimpl.to_string(),
            Error::SPILL => language.errors.spill.to_string(),
            Error::CALC => language.errors.calc.to_string(),
            Error::CIRC => language.errors.circ.to_string(),
            // Not in `Language` (SPEC-0003): the en-US names in every language.
            Error::GETTINGDATA | Error::CONNECT | Error::BLOCKED | Error::FIELD | Error::BUSY => {
                self.to_string()
            }
        }
    }

    /// The literal Excel saves for this error in a workbook's cell value: `#N/A` for
    /// `#GETTING_DATA` (SPEC-0003, "Stored form"), the `Display` text otherwise. A storage rule
    /// only; in memory the value keeps its kind, so ERROR.TYPE stays 8 and ISNA FALSE.
    pub fn stored_literal(&self) -> &'static str {
        match self {
            Error::GETTINGDATA => Error::NA.english_literal(),
            Error::REF
            | Error::NAME
            | Error::VALUE
            | Error::DIV
            | Error::NA
            | Error::NUM
            | Error::ERROR
            | Error::NIMPL
            | Error::SPILL
            | Error::CALC
            | Error::CIRC
            | Error::NULL
            | Error::CONNECT
            | Error::BLOCKED
            | Error::FIELD
            | Error::BUSY => self.english_literal(),
        }
    }
}

pub fn get_error_by_name(name: &str, language: &Language) -> Option<Error> {
    let errors = &language.errors;
    if name == errors.r#ref {
        return Some(Error::REF);
    } else if name == errors.name {
        return Some(Error::NAME);
    } else if name == errors.value {
        return Some(Error::VALUE);
    } else if name == errors.div {
        return Some(Error::DIV);
    } else if name == errors.na {
        return Some(Error::NA);
    } else if name == errors.num {
        return Some(Error::NUM);
    } else if name == errors.error {
        return Some(Error::ERROR);
    } else if name == errors.nimpl {
        return Some(Error::NIMPL);
    } else if name == errors.spill {
        return Some(Error::SPILL);
    } else if name == errors.calc {
        return Some(Error::CALC);
    } else if name == errors.circ {
        return Some(Error::CIRC);
    } else if name == errors.null {
        return Some(Error::NULL);
    } else if name.eq_ignore_ascii_case(GETTING_DATA_LITERAL) {
        // Only #GETTING_DATA of the newer errors is a formula/input literal (SPEC-0003).
        return Some(Error::GETTINGDATA);
    }
    None
}

pub fn get_error_by_english_name(name: &str) -> Option<Error> {
    if name == "#REF!" {
        return Some(Error::REF);
    } else if name == "#NAME?" {
        return Some(Error::NAME);
    } else if name == "#VALUE!" {
        return Some(Error::VALUE);
    } else if name == "#DIV/0!" {
        return Some(Error::DIV);
    } else if name == "#N/A" {
        return Some(Error::NA);
    } else if name == "#NUM!" {
        return Some(Error::NUM);
    } else if name == "#ERROR!" {
        return Some(Error::ERROR);
    } else if name == "#N/IMPL!" {
        return Some(Error::NIMPL);
    } else if name == "#SPILL!" {
        return Some(Error::SPILL);
    } else if name == "#CALC!" {
        return Some(Error::CALC);
    } else if name == "#CIRC!" {
        return Some(Error::CIRC);
    } else if name == "#NULL!" {
        return Some(Error::NULL);
    } else if name == GETTING_DATA_LITERAL {
        return Some(Error::GETTINGDATA);
    } else if name == "#CONNECT!" {
        return Some(Error::CONNECT);
    } else if name == "#BLOCKED!" {
        return Some(Error::BLOCKED);
    } else if name == "#FIELD!" {
        return Some(Error::FIELD);
    } else if name == "#BUSY!" {
        return Some(Error::BUSY);
    }
    None
}

/// Whether a COUNTIF-family criterion names an error. Deliberately only the twelve older names:
/// COUNTIF with `"#BLOCKED!"`, `"#CONNECT!"` or `"#FIELD!"` counts 0 even over a cell holding
/// that error (T-1012, information.md), so they stay text criteria. `"#BUSY!"` is assumed to
/// behave the same (UNVERIFIED, T-1228).
pub fn is_english_error_string(name: &str) -> bool {
    let names = [
        "#REF!", "#NAME?", "#VALUE!", "#DIV/0!", "#N/A", "#NUM!", "#ERROR!", "#N/IMPL!", "#SPILL!",
        "#CALC!", "#CIRC!", "#NULL!",
    ];
    names.contains(&name)
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum TableSpecifier {
    All,
    Data,
    Headers,
    ThisRow,
    Totals,
    /// `[[#Headers],[#Data]]`: everything above the totals row.
    HeadersData,
    /// `[[#Data],[#Totals]]`: everything below the header row.
    DataTotals,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum TableReference {
    ColumnReference(String),
    RangeReference((String, String)),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum TokenType {
    Illegal(LexerError),
    EOF,
    Ident(String),      // abc123
    String(String),     // "A season"
    Number(f64),        // 123.4
    Boolean(bool),      // TRUE | FALSE
    Error(Error),       // #VALUE!
    Compare(OpCompare), // <,>, ...
    Addition(OpSum),    // +,-
    Product(OpProduct), // *,/
    Power,              // ^
    LeftParenthesis,    // (
    RightParenthesis,   // )
    Colon,              // :
    Semicolon,          // ;
    LeftBracket,        // [
    RightBracket,       // ]
    LeftBrace,          // {
    RightBrace,         // }
    Comma,              // ,
    Bang,               // !
    Percent,            // %
    And,                // &
    At,                 // @
    Spill,              // #
    Backslash,          // \
    Reference {
        sheet: Option<String>,
        row: i32,
        column: i32,
        absolute_column: bool,
        absolute_row: bool,
    },
    Range {
        sheet: Option<String>,
        left: ParsedReference,
        right: ParsedReference,
    },
    StructuredReference {
        table_name: String,
        specifier: Option<TableSpecifier>,
        table_reference: Option<TableReference>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language::get_default_language;

    /// The five newer kinds (SPEC-0003) with their literal and ERROR.TYPE code.
    const NEWER: [(Error, &str, u8); 5] = [
        (Error::GETTINGDATA, "#GETTING_DATA", 8),
        (Error::CONNECT, "#CONNECT!", 10),
        (Error::BLOCKED, "#BLOCKED!", 11),
        (Error::FIELD, "#FIELD!", 13),
        (Error::BUSY, "#BUSY!", 0),
    ];

    #[test]
    fn newer_kinds_display_code_and_stored_name() {
        for (error, literal, code) in NEWER {
            assert_eq!(error.to_string(), literal);
            assert_eq!(error.error_type_code(), code);
            assert_eq!(get_error_by_english_name(literal), Some(error));
        }
    }

    #[test]
    fn stored_literal_saves_getting_data_as_na() {
        assert_eq!(Error::GETTINGDATA.stored_literal(), "#N/A");
        for error in [
            Error::CONNECT,
            Error::BLOCKED,
            Error::FIELD,
            Error::BUSY,
            Error::NA,
            Error::DIV,
        ] {
            assert_eq!(error.stored_literal(), error.to_string());
        }
    }

    #[test]
    fn only_getting_data_is_a_formula_name() {
        let language = get_default_language();
        for name in ["#GETTING_DATA", "#getting_data", "#Getting_Data"] {
            assert_eq!(get_error_by_name(name, language), Some(Error::GETTINGDATA));
        }
        for name in ["#CONNECT!", "#BLOCKED!", "#FIELD!", "#BUSY!", "#UNKNOWN!"] {
            assert_eq!(get_error_by_name(name, language), None, "{name}");
        }
    }

    #[test]
    fn criteria_error_names_are_unchanged() {
        // COUNTIF with "#BLOCKED!" counts 0 over a #BLOCKED! cell (T-1012, information.md).
        for (_, literal, _) in NEWER {
            assert!(!is_english_error_string(literal), "{literal}");
        }
        assert!(is_english_error_string("#N/A"));
    }
}
