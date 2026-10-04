//! RustSheet public API v0 (SPEC-0001): a typed facade over the calculation engine.
//!
//! Engine types never appear in public signatures; engine `String` errors are classified into
//! the typed errors defined here.
#![deny(missing_docs)]
#![forbid(unsafe_code)]

mod edit;
mod error;
mod refs;
mod value;
mod workbook;

pub use edit::{ChangeSet, Input, InputChange, SheetChange, Txn, ValueChange};
pub use error::{EditError, LoadError, RefError, SaveError};
pub use refs::{
    parse_cell, parse_range, resolve_sheet_by_name, CellRef, RangeRef, SheetId, Target,
};
pub use value::{Array, ArrayShapeError, ErrorKind, Value, XlError};
pub use workbook::{CellInfo, LoadOptions, SaveOptions, SheetMeta, Workbook, WorkbookOptions};
