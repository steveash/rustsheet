//! RustSheet public API v0 (SPEC-0001): a typed facade over the calculation engine.
//!
//! Engine types never appear in public signatures; engine `String` errors are classified into
//! the typed errors defined here.
#![deny(missing_docs)]
#![forbid(unsafe_code)]

mod error;
mod value;

pub use error::{EditError, LoadError, RefError, SaveError};
pub use value::{ErrorKind, XlError};
