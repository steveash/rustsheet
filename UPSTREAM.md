# Upstream provenance

RustSheet started as a fork of [IronCalc](https://github.com/ironcalc/IronCalc)
(dual-licensed MIT OR Apache-2.0; we use it under Apache-2.0).

| Field | Value |
|---|---|
| Upstream repo | https://github.com/ironcalc/IronCalc |
| Fork point | `e29e7b5c5a36b40c8d3b3c49b8731ac426f43606` (2026-09-30, "update: improve testing for TIME, HOUR, MINUTE, SECOND (#1456)") |
| Import style | Snapshot copy (no upstream git history) |

## Path mapping at fork time

| Upstream path | RustSheet path | Notes |
|---|---|---|
| `base/` (`ironcalc_base`) | `crates/rustsheet-engine/` (`rustsheet_engine`) | calc engine core |
| `xlsx/` (`ironcalc`) | `crates/rustsheet-xlsx/` (`rustsheet_xlsx`) | xlsx import/export + upstream calc tests |
| `bindings/python/` (`ironcalc`) | `bindings/python/` (`rustsheet`) | PyO3 bindings |
| `generate_language/`, `generate_locale/` | `tools/upstream/` | generators for `language.bin` / `locales.bin` |
| `webapp/`, `bindings/wasm/`, `bindings/nodejs/`, `docs/` | dropped | out of scope for a headless engine |

Modifications at import: crate/package renames, workspace-level lint policy,
lint-debt allow-lists in each forked crate's `Cargo.toml`, `cargo fmt`.

## RustSheet-only engine additions

Not in upstream IronCalc; keep when porting nearby code: `Model::prepare_cell_for_user_input`
(typed setters run the spill/array/merge checks), `Model::take_sheet` / `Model::restore_sheet`
(`sheet_undo.rs`, undoable sheet delete), `Model::evaluation_count` and `Model::define_name`
(any parseable name body). The defined-name methods of upstream `model.rs` live in
`defined_names.rs` (moved in T-0043 to shrink `model.rs`).
`Node::SheetSpanKind` (3-D references `First:Last!A1`, T-0122): lexer `consume_sheet_span`, parser
`parser/sheet_span.rs` (where `get_sheet_index_by_name` now lives), and arms in stringify,
move_formula, static_analysis and units; a port touching `Node` matches must keep that arm.
`expressions::token::Error` has five variants upstream lacks, appended after `NULL` (`GETTINGDATA`,
`CONNECT`, `BLOCKED`, `FIELD`; SPEC-0003, T-1010; and the stored-only `BUSY`, ERROR.TYPE 0,
T-1222), plus `Error::stored_literal`. The lexer's
`consume_error` and `get_error_by_name` also accept `#GETTING_DATA` (any case) as an en-US constant
outside `Language`; `get_error_by_english_name` knows all seventeen names. A port touching an `Error`
match must keep those arms.
`Model::add_table` / `Model::remove_table` (`tables.rs`, tables added through the API, T-1036) and
`Parser::set_tables` (the parser's table list is refreshed when a table is added).

## Tracking upstream

We do **not** merge upstream. Upstream fixes are *ported* deliberately:
`scripts/upstream-log.sh` lists upstream commits since the fork point (or since
the last commit recorded below) so the "upstream watch" task can triage them.
Every port records the upstream SHA in its commit message (`Ported-From: ironcalc@<sha>`).

| Last triaged upstream commit | Date |
|---|---|
| `f428b2ebbc57fcf292f3e67027b24e1c866cb7c4` | 2026-10-13 |
