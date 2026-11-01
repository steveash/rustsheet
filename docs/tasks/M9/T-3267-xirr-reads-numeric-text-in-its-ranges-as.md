---
id: T-3267
title: "XIRR reads numeric text in its ranges as numbers (xirr-back-dated-stop A31)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`XIRR(G1:G13,H1:H13)` on probes/xirr-back-dated-stop, whose values and dates are numeric text in cells, returns
Excel's 0.03189964592456816 (the bits of the same flows as numbers, A2) instead of our `#VALUE!`.

## Context
- Known failure: probes/xirr-back-dated-stop Sheet1!A31 (excel-graph, 2026-10-30, T-2045). Found by the
  T-2045 excel-fidelity review: the probe first wrote its range data as JSON strings.
- Semantics: financial.md "XIRR reads empty cells as 0 and accepts earlier dates; XNPV is #NUM! on empty cells"
  (T-2045 note: numeric text in the ranges reads as numbers; non-numeric text is `#VALUE!`, xirr-mismatched-empties
  X6, X7). Excel's XNPV does not refuse numeric text in one values cell either (financial.md "XIRR and XNPV under
  the gen-m9 statistical cells", 81bcdc0bec0c8e6b A37).
- Code: XIRR's argument reading in crates/rustsheet-engine/src/functions/financial/mod.rs (`xirr_size_error` and
  the values/dates readers) and `compute_xirr` in financial_util.rs. Reuse the engine's existing numeric-text
  coercion; do not write a new parser.

## Acceptance criteria
- [ ] xirr-back-dated-stop A31 leaves eval/baseline/probes.txt, with no new known-failure line (xirr-mismatched-empties
      X6 and X7 stay `#VALUE!`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Numeric text in literal arrays or typed arguments (not probed).
- XNPV's own reading of ranges (probe it first).
