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

pub mod baseline;
pub mod calibrate;
pub mod case;
pub mod case_log;
pub mod catalog;
pub mod cellref;
pub mod deviations;
pub mod engine;
pub mod evidence_floor;
pub mod fetch;
pub mod function_scope;
pub mod milestone_lists;
pub mod overrides;
pub mod probe_milestones;
pub mod report;
pub mod runner;
pub mod suite;
pub mod value;
pub mod volatile;
pub mod xlsx_snapshot;
