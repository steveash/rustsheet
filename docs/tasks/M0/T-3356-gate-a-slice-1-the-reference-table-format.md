---
id: T-3356
title: "Gate A slice 1: the reference-table format, its reader and the make check gate (from the Gate A spec)"
milestone: M0
lane: harness
status: todo
priority: P0
size: M
tier: opus
depends_on: [T-3355]
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
The Gate A reference-table format of the Gate A spec (T-3355) exists, with a reader and the check `make check` runs:
a point fails when it is neither within ε = 1e-9 relative of its math value (absolute 1e-300 near 0) nor within 15
significant digits of Excel's value, or when one side is an error and the other a number (decision 0188 R2).

## Context
- Decision 0188 R1(a) and R2; the format, location and runner are the spec's (T-3355).
- Reuse the eval engine adapter and `Visible::is_off` (crates/rustsheet-eval), never a second 15-digit rule.

- SPEC-0029 decides the design: the `eval/accuracy/` manifest resolved by `Suite::load_named` (for `cases` and
  `show`, never `make eval`), batches `tables/<FUNCTION>/<batch>.case.json` with oracle and `rustsheet-math/v1` math
  sidecars, `envelope` (the one definition T-3359 and T-3380 reuse), `known-failures.tsv` (stale lines fail),
  `math-over-excel.tsv` (anchors checked by `deviations::check_entry` given the status, Excel off by > 1e-4 on a filled
  marked point), the summary and `--json eval/runs/accuracy.json`, a 30 s budget, and the `make eval-gates` wiring
  (Makefile is protected: a decision record).
## Acceptance criteria
- [ ] The reader and the check exist as the spec says, with unit tests for a passing point, a point within ε of the
      math only, within 15 digits of Excel only, outside both, an error against a number, and an R3 point checked
      against the math only.
- [ ] `make check` runs the check and fails on a failing point (shown by a test fixture, not a real table).
- [ ] The check's runtime on the slice's fixture tables is reported in the Outcome against the spec's budget.
- [ ] docs/architecture/eval-harness.md describes the check and links SPEC-0029.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The generator and the oracle fill (T-3357); the real tables (T-3358).
