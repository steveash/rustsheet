---
id: T-3408
title: "IRR's digits on conventional flows where the secant fails: match Excel's stop (21 cells of the T-3347 cases)"
milestone: M9
lane: functions-financial
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-3347]
campaign: accuracy
---

## Goal
IRR returns Excel's value to 15 significant digits on the conventional cash-flow series where the secant model fails
(near r = 0 and near -10%): today T-3347's last stage returns the exact root (within 1.5e-11), which is 1e-13 to
2.2e-11 relative from Excel's double on 21 of the 162 cells of the T-3347 cases.

## Context
- T-3347 removed the `#NUM!` (docs/excel-semantics/financial.md "IRR on conventional flows near 0 and -10%"): a
  Newton stage on P(v) from v = 1, after the secant and the v = 0 fallback, fitted on which flows return. Its digits
  are a converged, polished Newton; Excel's are not (its stop is looser), and neither a longer secant nor any secant
  stop variant gives Excel's bits (the T-3347 entry lists the variants tried).
- Cells: probes/t3347-irr-conventional-num A10-A13, A16, A17, A19, A37-A40; probes-numeric/t3347-irr-conventional-sweep
  (the numeric_close cells after T-3347; `make eval-show CASE=...`). Exact roots in each case's `.roots.tsv`.
- Code: `irr_conventional`, `irr_newton` in crates/rustsheet-engine/src/functions/financial/financial_util.rs.
- Related: T-2928 (blocked: the v = 0 fallback's digits, the same question for non-conventional flows).
- Any fit follows decision 0188 R4 (hold-out split, fresh sweep, `FITTED` tag via T-3361).

## Acceptance criteria
- [ ] Every cell of the two T-3347 cases is within 15 significant digits of Excel's value, with the right type.
- [ ] A fresh hold-out sweep (decision 0188 R4.1) shows the same, and the Gate A floor still holds.
- [ ] Oracle cells for the shapes the stage takes but T-3347 did not probe: the mirrored sign (a positive first flow,
      then negative ones) and zero flows after the outlay; the stage is narrowed if Excel differs there.
- [ ] No gated cell regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Bit-level parity (Phase 1b, decision 0188 R5).
