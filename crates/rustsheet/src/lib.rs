//! RustSheet public API v0 (SPEC-0001): a typed facade over the calculation engine.
//!
//! Engine types never appear in public signatures; engine `String` errors are classified into
//! the typed errors defined here.
#![deny(missing_docs)]
#![forbid(unsafe_code)]

mod error;
mod refs;
mod value;

pub use error::{EditError, LoadError, RefError, SaveError};
pub use refs::{
    parse_cell, parse_range, resolve_sheet_by_name, CellRef, RangeRef, SheetId, Target,
};
pub use value::{ErrorKind, XlError};
