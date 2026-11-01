---
id: T-3548
title: "IRR past the secant fit set: 50 T-3478 sweep cells off Excel (33 roots the engine misses, 17 Excel errors)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
IRR gives Excel's type (number, `#NUM!`, `#DIV/0!`) and, for numbers, Excel's value within 15 digits on the 50 cells
of probes-numeric/t3478-irr-r4-wide outside Gate A's envelope, or each remaining cell is explained in financial.md.

## Context
- T-3478 drew 300 conventional series (one sign change, 50-digit roots in the case's `.math.tsv`) past the IRR secant
  fit set (2..41 flows, |c| 1E-6..1E+9, guess -0.99..100, roots -0.99..99) and kept the fitted offset and residual:
  in aggregate they still beat every alternative there (financial.md "RATE and IRR secant constants past their fit sets"). The
  misses are elsewhere in the solver:
  - **Excel returns a root, the engine is `#NUM!` (33):** Calc!A1, A2, A9, A12, A13, A15, A18, A19, A21, A22, A27,
    A32, A33, A34, A35, A36, A40, A41, A43, A46, A48 (21 of the 50 series with 42..150 flows; roots 1E-4 to 1.4E-2),
    A53, A83 (tiny or huge flows), A206, A210, A216, A229, A234, A242 (guesses near -1 or above 100), A251, A269,
    A294, A296. The secant from the guess oscillates and hits the 50-step cap (A9 is still 1E-2 off at step 50), then
    the fallback fails. A 100-step cap recovers 12 cells of the sweep (244 against 232 in the fit lab replica), a 1000-step one 15,
    so the cap is part of it but not all: 19 of the 21 long series stay `#NUM!` at 1000 steps. This is the family of
    "IRR on conventional flows where the secant fails" (T-3406, T-3411, T-3347).
  - **Excel errs, the engine returns the root (13):** `#NUM!` at Calc!A60, A68, A69, A70, A90 (flows of 1E-12 to 1E-6;
    why Excel errs there is unknown), A102, A131, A135 (roots near -0.995 with an outlay 1E+6 to
    1E+7 times the inflows), A258, A262; `#DIV/0!` at A261, A275, A281 (guesses near -1 or above 100).
  - **Both err, with another kind (3):** Excel `#DIV/0!`, the engine `#NUM!` at Calc!A277, A290, A298 (far guesses).
  - **A289** `IRR({-3.02E-09, ...},96471664.1364151)`: Excel `#DIV/0!`; the engine returns 0.10121131912486914, which
    is not the root (0.000174819755577...): with flows near 1E-11 every |NPV| is under the absolute 1E-7 stop, so the
    retry from 0.1 stops at its first small step. This is the residual stop's absolute scale, a wrong value and not
    only a wrong type.
- financial.md "IRR is #NUM! or #DIV/0! on some conventional series it could solve" lists the 17 Excel-error cells.
- Code: `irr_secant`, `compute_irr`, `irr_fallback` (crates/rustsheet-engine/src/functions/financial/financial_util.rs).

## Acceptance criteria
- [ ] Each of the 50 cells matches Excel's type and, for a number, its value within 15 digits, or is listed in
      financial.md with what is known; the rule found is evidenced by a probe (excel-graph).
- [ ] No gated cell regresses; IRR's 263-cell fit set stays bit-exact.
- [ ] Gate A envelope on a fresh sweep of its own after the change (decision 0188 R4.2).
- [ ] `make check` green; reviewers approve.

## Out of scope
- T-3347's near-zero roots and T-3406/T-3411's annuity boundary, except where one rule covers both (then say so in
  both tasks).
