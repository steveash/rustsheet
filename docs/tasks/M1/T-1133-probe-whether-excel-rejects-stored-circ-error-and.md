---
id: T-1133
title: "Probe whether Excel rejects stored #CIRC!, #ERROR! and #N/IMPL literals; exporter fallback if so"
milestone: M1
lane: harness
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's behavior for stored `#CIRC!`, `#ERROR!` and `#N/IMPL` error literals is known, and the exporter writes them in a form Excel accepts.

## Context
Found in the T-1011 fidelity review. T-0999 showed Excel for the web rejects stored literals it has no kind for; `exported_error_literal` in `crates/rustsheet-xlsx/src/export/worksheets.rs` still writes these three engine-only kinds as literals. Follow the `excel-probe` skill (stored-constant probes as in probes/error-stored-constants).

## Acceptance criteria
- [x] A probe case stores each of the three literals; the oracle result (accepted or rejected) is committed and recorded in `docs/excel-semantics/syntax.md`.
- [x] If rejected, `exported_error_literal` writes them as `#VALUE!` with a test; otherwise nothing changes.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Excel for the web rejects a stored `#CIRC!`, `#ERROR!` and `#N/IMPL` constant on load (501 `unsupportedWorkbook`, one-cell throw-away cases, no sidecar since nothing loads; recorded in syntax.md). `exported_error_literal` now writes the three as `#VALUE!`, with the unit test extended. No eval delta (no scored cells change).
