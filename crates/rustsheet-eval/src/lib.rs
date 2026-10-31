//! RustSheet Excel-compatibility evaluation harness.
//!
//! The harness loads test cases from pluggable *suites* (`eval/suites/*/suite.toml`),
//! runs them through an [`engine::Engine`], compares every checked cell against the
//! Excel oracle value with strict rules ([`value::compare`]), and produces:
//!
//! * a scoreboard and per-function gap report ([`report::RunReport`]),
//! * a no-regression ratchet against committed baselines ([`baseline`]).
//!
//! See `docs/architecture/eval-harness.md`.

pub mod accuracy;
pub mod argument_coverage;
pub mod attribution;
pub mod baseline;
pub mod calibrate;
pub mod case;
pub mod case_log;
pub mod catalog;
pub mod catalog_coverage;
pub mod cellref;
pub mod closeness;
pub mod deviations;
pub mod engine;
pub mod evidence_floor;
pub mod exit_exceptions;
pub mod fetch;
pub mod formula_calls;
pub mod function_scope;
pub mod gate_b;
pub mod lift_kinds;
pub mod lock_file;
pub mod milestone_exit;
pub mod milestone_lists;
pub mod out_of_scope;
pub mod overrides;
pub mod phase1_exceptions;
pub mod poi_m8_subset;
pub mod probe_milestones;
pub mod report;
pub mod run_meta;
pub mod runner;
mod spec_pivots;
mod spill_extent;
pub mod suite;
pub mod text_expect;
pub mod v15_checkpoint;
pub mod value;
pub mod visible_parity;
pub mod volatile;
pub mod xlsx_snapshot;
