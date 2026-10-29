---
id: T-3105
title: "Snap the near-zero root of a LAMBDA body called through a defined name, as an in-place call does"
milestone: M2
lane: engine
status: in-progress
priority: P2
size: S
tier: opus
depends_on: [T-3083]
---

## Goal
A LAMBDA bound to a workbook-scoped defined name snaps its body's near-zero `+`/`-` root when called, as Excel does
and as the engine already does for an in-place, LET-bound or nested call.

## Context
- Evidence: probes/t3094-snap-lambda-call-forms (T-3094, excel-graph 2026-10-29); docs/excel-semantics/numbers.md
  "Near-zero snapping", paragraph "Pickers, LET, LAMBDA and unary signs". Excel snaps the body at every call form
  probed; the body's own shape decides (a grouped body, an `IF` body or a LET body is not snapped).
- Failing cells (known failures in eval/baseline/probes.txt): Calc!C2, C3, C5 (`=snapf_n(v)`), D2, D3, D5
  (`=snapf_n(v)+0`), E2, E3, E5 (`=(snapf_n(v))`), F2, F3, F5 (`=SUM(snapf_n(v))`) and J2, J3, J5 (`=snaph_n(v)`,
  a named LAMBDA whose body calls the named `snapf_n`). Row 4 (k = 8) keeps its residue everywhere and passes.
- Code: T-3083 snaps the in-place call in `Model::call_lambda` (functions/logical/lambda.rs) through
  `Model::evaluate_formula_root` / `handle_outermost_sum` (arithmetic.rs); the named-LAMBDA call path does not
  reach it. Route it through the same root evaluation instead of adding a second snap.

## Acceptance criteria
- [ ] Every cell of probes/t3094-snap-lambda-call-forms passes (`make eval-baseline` drops its 15 known failures),
      no regressions.
- [ ] numbers.md's paragraph says the engine matches the named-call cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- LAMBDA helpers (MAP, BYROW, REDUCE, ...): T-3095.
- Sheet-scoped names.
