---
id: T-0021
title: "Fuzz the formula parser and xlsx importer (cargo-fuzz, nightly)"
milestone: M0
lane: quality
status: done
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
Untrusted formulas and workbooks are fuzzed continuously, so panics, stack overflows and
unbounded allocation are found by machines, not by luck (CLAUDE.md directive 5).

## Context
T-0931 (stack overflow on deeply nested parentheses) was found by hand. Fuzzing was planned for
M12; the 2026-10-04 review moved it forward.

## Acceptance criteria
- [x] `fuzz/` with cargo-fuzz targets: formula parse, parse+evaluate on a small model, xlsx import (with zip-bomb limits on).
- [x] Seed corpora from eval/suites and crates/rustsheet-xlsx/tests; dictionary of formula tokens.
- [x] Nightly workflow runs each target for a fixed time budget; a crash uploads the input and fails the job; the steward files a task per distinct crash.
- [x] The `fuzz/` crate depends only on public engine/xlsx APIs; nothing in the engine depends on it.
- [x] Any crash found while building this is fixed or filed.
- [x] The workflow uses least-privilege `permissions:`, never `pull_request_target`, and uploads only the crashing input (no secrets or environment).
- [x] Decision record for the protected-path change (CI); `make check` green; reviewers approve.

## Outcome
`fuzz/` (own workspace and nightly `rust-toolchain.toml`, public APIs only) with cargo-fuzz
targets `formula_parse` (parse + every stringifier + re-parse), `formula_eval` (facade
workbook with numbers, text, bool, error, spill, defined name, second sheet) and `xlsx_import`
(facade `from_xlsx_bytes` with 16 MiB / 500-entry limits, recalc, read, re-export).
`fuzz/seed_corpus.py` seeds from every formula and defined name in eval/suites (~17.4k) and the
253 workbooks under crates/rustsheet-xlsx/tests and eval/suites, and builds `dict/formula.dict`
(committed syntax tokens + function names seen). `.github/workflows/fuzz.yml` runs each target
20 min nightly (read-only permissions, crash inputs only uploaded, corpus kept in the cache);
steward duty in the steward skill and agent-workflow.md (decision 0035).
Crashes found while building (5 min per target locally): fixed `column_to_number` i32
overflow (`Data!ISERROR(…)`; in release `NKJDCLBT1` lexed as a reference) and an out-of-bounds
signature index for too many arguments to a fixed-signature function (`DMIN(A1:C4,C1,,C1,A6)`);
filed T-0047 for the `DATE(11:1899,12,1)` out-of-memory. Eval delta: none (no eval cells change).
