---
id: T-2688
title: "XIRR with an empty guess argument (XIRR(v,d,)) uses the default 0.1, not guess 0 (calipers pv23_056 Summary!F23)"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
XIRR reads an empty guess argument the way Excel does. On the desktop evidence in the repo that is the
default 0.1, so `XIRR(values,dates,)` gives Excel's value, not the guess-0 run. IRR's empty guess is
probed and matched at the same time.

## Context
Gap triage 2026-10-25 (`make eval` on agent/engine/T-2632-typed-arg-counts at 4dbb3c62). This is the
only visibly-off cell of calipers pv23_056. The other 124 failing cells there are ulp-level.
- `calipers/pv_goldens/pv23_056/golden.xlsx` Summary!F23
  `=XIRR('Monthly Cash Flow Statement'!E38:BY38,'Monthly Cash Flow Statement'!E4:BY4,)`.
  Excel (cached by Microsoft Macintosh Excel, AppVersion 16.0300): 0.11376532912254333. Ours: 0.11376532714843753.
  Known-failure line `F calipers/pv_goldens/pv23_056/golden.xlsx Summary!F23` in eval/baseline/calipers.txt.
  The flows are 73 cells: -22,450,000 first, then monthly inflows, and 36 trailing zeros. Their dates are
  45778 to 47999. The engine's row-38 inputs differ from Excel's by ulps only (numeric_ulp).
- The triager ran the fit lab's XIRR model (`tools/src/rustsheet_tools/fitlab/kernels/xirr.py` `xirr_model`
  with `fitted_stop()`) on Excel's cached flows and dates:
  - guess 0.1 gives 0.11376532912254333, Excel's value, as bisection midpoint 23;
  - guess 0 (s = 1e-5) gives 0.11376532714843753, ours exactly;
  - guess 0.01 gives 0.11376532077789303.
  So the engine reads the empty third argument as 0 (`cast_number_or(arguments.get(2), 0.1, cell)` in `fn_xirr`,
  functions/financial/mod.rs: the empty-argument node casts to 0). Excel used the default 0.1.
- This is the opposite of RATE. financial.md "RATE: an empty guess is guess 0, not the default 0.1"
  (probes/rate-empty-arguments, excel-graph) says `RATE(48,-200,8000,0,1,)` and a reference to an empty cell
  both run from guess 0. So the rule is per function, and neither side may be generalised from the other.
- Evidence status. The only cell is desktop (Mac 16.x). Decision 0030 makes desktop the reference.
  Decision 0039 makes excel-graph the oracle of record for Phases 1-3, so first probe it on excel-graph
  (excel-probe skill):
  - `XIRR(v,d,)`, with v and d as ranges and as array constants;
  - `XIRR(v,d,E1)` with E1 empty, and `XIRR(v,d,"")`;
  - `XIRR(v,d)` and `XIRR(v,d,0)` as controls, using flows whose guess-0 and guess-0.1 bisections end on
    different midpoints (pv23_056's flows do);
  - the same four forms for IRR (`IRR(v,)`, `IRR(v,E1)`; IRR is a secant in v = 1/(1+r) from v0, financial.md),
    with flows whose guess-0 and guess-0.1 results differ.
  If the web agrees with the Mac cell, implement it. If it disagrees, decision 0030 applies: desktop
  evidence already in the repo stays the target. Record the conflict in the semantics entry, and add the
  desktop re-check to the M14 desktop task T-1075.
- No docs/excel-semantics entry covers XIRR's empty guess yet; this task adds it.

## Acceptance criteria
- [x] A probe case (excel-graph, oracle-filled, at least 12 cells) covers the XIRR and IRR forms above.
      It is gated or baselined.
- [x] financial.md has an entry "XIRR (and IRR): an empty guess argument is ..." that cites
      pv23_056 Summary!F23 and the probe cells, and cross-references the RATE entry.
- [ ] (Partly: F23 passes and the probe's XIRR cells pass; the line removal waits on T-0463, the six IRR cells on T-2131, see Outcome.) The engine matches. `F calipers/pv_goldens/pv23_056/golden.xlsx Summary!F23` disappears from
      eval/baseline/calipers.txt (`rustsheet-eval update-baseline --suite calipers`), the new probe cells pass,
      and no baseline gains a line.
- [x] RATE's empty-guess cells (probes/rate-empty-arguments) still pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- XIRR's bisection model, stop rule and caps (financial.md "XIRR is a bracketing bisection"), and XNPV's power
  (T-2566).
- RATE's empty-argument rule (verified; keep it).

## Visible-parity stage
Decision 0083: this task is done when every cell it owns is within half a unit of the 15th significant digit of
Excel's value (an expected 0 must be exactly 0), or has the right type, error and text. Criteria above that ask
for bit-exact or ulp-level matches and are still unmet then move to a follow-up task (no campaign key), named in
the Outcome. Regressions that stay within 4 ulp follow decision 0083's allowance.

Owned visibly-off cells (2026-10-25 eval run): calipers pv23_056 Summary!F23 (XIRR, off in the 9th digit).
Stage `semantics`: a semantic rule (how an empty argument reads) gives the wrong number.

## Outcome
- Probe `probes/xirr-irr-empty-guess` (22 cells, excel-graph) agrees with the Mac cell: `XIRR(v,d,)` is the
  default 0.1 (same bits as no guess and `,0.1`), while an empty-cell guess and `0.1-0.1` are guess 0. IRR reads
  `IRR(v,)` and `IRR(v,E1)` as guess 0, like RATE. A `""` guess is `#VALUE!` for both.
- `fn_xirr` drops an `EmptyArg` guess so the default applies; IRR and RATE are unchanged
  (probes/rate-empty-arguments still passes). New financial.md entry "XIRR (and IRR): an empty guess argument
  is the default 0.1 for XIRR, guess 0 for IRR", cross-referencing RATE's.
- Eval delta: probes 2599 -> 2605 known failures, all new coverage: the six IRR guess-0 cells (A11, A13, A15,
  A17, A19, A21), where our IRR from guess 0 is a few ulps off Excel (…3171 for …3149), an existing IRR kernel
  gap, now named in T-2131, not an empty-guess rule. The 16 XIRR cells pass. calipers pv23_056 Summary!F23 now
  passes (0.11376532912254333).
- Not met here: removing the `F ... pv23_056 ... Summary!F23` line. `update-baseline --suite calipers` refuses on
  main's 36 pre-existing ulp regressions (pv23_007/008/009/056 PPMT, NPV, SUM; none XIRR), exactly the cells
  T-0463 owns, and recording them is that task's call. The line is stale until T-0463 ratchets the suite.
