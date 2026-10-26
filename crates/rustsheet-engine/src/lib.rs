//! # `IronCalcBase` engine API
//!
//! This is the documentation for the base engine API.
//!
//! # Basic usage
//!
//! Add the dependency in Cargo.toml:
//!
//! ```toml
//! [dependencies]
//! rustsheet_engine = { git = "https://github.com/ironcalc/IronCalc" }
//! ```
//!
//! <small> until version 1.0.0 you should use the git dependencies as stated </small>
//!
//! In this example we use the excel function `CONCAT` to concatenate strings in cells `A1` and `B1`:
//!
//! ```rust
#![doc = include_str!("../examples/hello_world.rs")]
//! ```
//!
//! In this example, we demonstrate our ability to handle formulas and errors:
//!
//! ```rust
#![doc = include_str!("../examples/formulas_and_errors.rs")]
//! ```

#![warn(clippy::print_stdout)]

pub mod builtin_styles;
pub mod calc_result;
pub mod cell;
pub mod cf_types;
pub mod clock;
pub mod colors;
pub(crate) mod error_cell;
pub mod expressions;
pub mod formatter;
pub mod language;
mod limits;
pub mod locale;
pub mod new_empty;
pub mod number_format;
pub mod pivot;
pub mod sheet_data;
pub(crate) mod sheet_undo;
pub mod text;
pub mod themes;
pub mod types;
pub mod worksheet;

pub use crate::constants::COLUMN_WIDTH_FACTOR;
pub use crate::constants::DEFAULT_COLUMN_WIDTH;
pub use crate::constants::ROW_HEIGHT_FACTOR;
pub use crate::constants::{LAST_COLUMN, LAST_ROW};

mod actions;
mod area_budget;
mod arithmetic;
mod cast;
mod cf_styles;
mod circular;
mod conditional_formatting;
mod constants;
mod cut_paste;
mod data_table_types;
mod data_tables;
mod decimal_round;
mod defined_names;
mod evaluation;
mod excel_order;
mod external_links;
mod external_reference;
mod formula_cells;
mod formula_rewrite;
mod function_names;
mod functions;
mod implicit_intersection;
mod iteration;
mod legacy_formula;
pub mod links;
mod merged_cells;
mod model;
mod nested_array;
mod node_frames;
mod outside_cell;
mod positions;
mod reference_operators;
mod restart_log;
mod seen_ranges;
mod styles;
mod tables;
mod tailed_array;
mod typed_input;
mod tz;
mod units;
mod used_area;
mod user_model;
mod utils;
mod variable_stack;
mod workbook;
mod x87_exp;
mod x87_log10;
mod x87_log1p;
mod x87_register;

#[cfg(test)]
mod test;

pub use arithmetic::flush_subnormal;
pub use evaluation::{KeyBuild, KeyHasher};
pub use formula_cells::ParsedFormula;
pub use function_names::{implemented_function_names, lift_kinds_table, LiftKinds};
pub use locale::get_supported_locales;
pub use merged_cells::MergeStructure;
pub use model::FmtSettings;
pub use model::Model;
pub use sheet_undo::TakenSheet;
pub use text::Text;
pub use user_model::BorderArea;
pub use user_model::ClipboardData;
pub use user_model::UserModel;
pub use utils::get_all_timezones;
