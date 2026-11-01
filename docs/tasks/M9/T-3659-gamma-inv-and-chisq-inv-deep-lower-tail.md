---
id: T-3659
title: "GAMMA.INV and CHISQ.INV deep lower tail outside T-3311's cut: which Newton iterate or search point Excel returns on t3311-fresh-sweep-2 (73 cells, inside Gate A)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
The deep lower-tail GAMMA.INV and CHISQ.INV cells of probes-numeric/t3311-fresh-sweep-2 that T-3311's shipped rule
leaves visibly off (73 cells, all outside its `NEWTON_START_MAX` cut, so still on T-3131's crossing search) reach
Excel's value within half a unit of the 15th significant digit, by a rule for where Excel takes Newton steps from the
leading-factor root and which iterate it returns, or the Outcome lists the rules tried with their counts
(decision 0056).

## Context
- Gap triage 2026-11-01f (`make eval` on a branch near origin/main fa5b844d6, excel-graph;
  `scripts/exit_owners.py --all-off` over the task files of origin/main and of every open PR and task branch): these
  cells have no open owner. T-3311 (done) filed the case as its R4.2 fresh sweep and reported "75 visibly off of 360"
  without naming an owner. The current report has 74 visibly-off cells there: A199 (upper tail) goes to T-3146, and
  this task owns the other 73. `rustsheet-eval show probes-numeric/t3311-fresh-sweep-2.case.json` lists them.
- Every cell is inside Gate A's envelope. Against the 50-digit math (docs/research/t3311/reference_fresh-sweep-2.json,
  unit-scale values times the scale): Excel is 0 to 802 ulp off (median 16), ours 0 to 271 (median 23), and ours is
  at most 9.3E-14 relative from Excel. Excel is not close to correctly rounded, so this is Excel's algorithm, not a
  more accurate kernel. Decision 0188 R5 parks such dense-sweep cells as Phase 1b, hence `defer: phase-1b` (same
  reading as T-3479 and T-3640). Remove the key only when the steward records the Phase 1 exit, or when an eval shows
  one of these cells outside Gate A (then give it `campaign: visible` and a Visible-parity stage).
- The region: shapes 1.155 to 18.82 (CHISQ.INV df 4 to 35), p from 7.07E-300 to 9.23E-13, scales 1 and a few others
  (A212 35.63, A227 0.007227, A229 14.04, A231 0.001108, A238 0.01548). No cell is at shape 20 or more (T-3294's).
  Examples, Excel vs ours (Excel's and our offset from the math):
  - A95 `GAMMA.INV(2.01E-298,1.695,1)`: 3.0064784089922926E-176 vs 3.0064784089920117E-176 (+802 / +1 ulp)
  - A13 `GAMMA.INV(1.66E-297,3.216,1)`: 9.935139791493008E-93 vs 9.935139791493233E-93 (-131 / +1)
  - A43 `GAMMA.INV(7.37E-197,1.155,1)`: 1.6434149258054737E-170 vs 1.643414925805563E-170 (-214 / +271)
  - A281 `GAMMA.INV(9.23E-13,3.497,1)`: 0.0007291560111103981 vs 0.0007291560111103987 (-5 / +1)
  - A125 `CHISQ.INV(1.65E-269,29)`: 3.623439329241321E-18 vs 3.623439329241315E-18
- Suspected root cause (docs/excel-semantics/statistics.md, "Deep lower tail at a shape above 1", T-3311
  paragraphs): Excel's deep-tail quantile is Newton steps from the leading-factor root
  `EXP((LN(p)+GAMMALN(a)+LN(a))/a)` on the leading-factor series form, the root being iterate 0. The engine applies
  that (`incomplete_gamma::closed_form::excel_newton_quantile`) only where the form at the root is within
  `NEWTON_START_MAX` = 1E-6 of p, shape 1.5 to 7.5 and p 1E-200 to 1.0000000827E-10 (FITTED(T-3311, 23 cells)),
  after T-2561's `excel_tiny_quantile` (`TINY_QUANTILE_CHECK`). On T-3311's first sweep (probes-numeric/t3311-fresh-sweep)
  the unrestricted rule was bit-exact at 50 of the 64 cells the engine leaves off at shapes 1 to 20 (T-3294's
  Context), so the likely fix is a wider gate plus the root test T-3294 and T-3403 discuss.
- Related open tasks, not duplicated here: T-3294 (shape 20 and above, and the root test), T-3403 (which iterate when
  the steps cycle; A355, A169, A107 of gen-m9 inside the cut), T-3265 (shape below 1), T-3146 (upper-tail flat band).
  A rule found here may close T-3294's root-test question; say so in both Outcomes.
- Evidence already cached: t3311-fresh-sweep (240 cells), t3311-fresh-sweep-2 (360), T-3311's split points
  (docs/research/t3311/split_points.json, 153 points) and probes-numeric/t3311-gamma-inv-lower-tail-cdf (Excel's
  GAMMA.DIST at the iterates). These are the training data now, so the R4.2 check needs a new sweep filled after the
  freeze (T-3311 did the same in-task). The case has no math sidecar yet (T-3588 keeps sidecars current).
- Code: crates/rustsheet-engine/src/functions/statistical/incomplete_gamma/ (closed_form.rs, the search in
  `gamma_quantile`). Scorer to reuse: docs/research/t3311/score.py and reference.py.
- Cells:
  - probes-numeric/t3311-fresh-sweep-2.case.json (73): S!A2, A4, A6, A7, A9, A13, A16, A25, A27, A33, A34, A38, A42, A43, A55, A57, A58, A71, A78, A82, A91, A93, A95, A96, A98, A102, A106, A108, A115, A120, A125, A127, A130, A135, A145, A146, A147, A156, A158, A162, A168, A212, A227, A229, A231, A238, A241, A247, A248, A250, A251, A255, A260, A270, A274, A279, A280, A281, A282, A286, A293, A295, A297, A300, A303, A304, A305, A314, A323, A324, A329, A338, A345

## Acceptance criteria
- [ ] Each of the 73 cells is within half a unit of Excel's 15th significant digit, or the Outcome lists it with
      Excel, new and 50-digit values and the reason; the count reaching 15 digits is stated against main's.
- [ ] Decision 0188 R4 (reviewer-excel-fidelity check 7), each guard shown in the Outcome:
      - hold-out counts: a deterministic ~30% split by cell hash over the cached sweeps, train and test side by side
        (bit-exact / within 4 ulp / within 15 digits), the chosen candidate against the correctly rounded one and
        main, the margin stated before scoring;
      - a fresh log-uniform sweep over shapes 1 to 20 and p 1E-300 to 1E-10, oracle-filled after the model is frozen
        (commit and fill times recorded), with its counts;
      - the Gate A envelope result on that sweep, every point outside it listed;
      - every fitted constant, cut or range tagged `FITTED(T-3659, n_cells, range)` and applied only inside its range,
        with accurate math (today's search) outside it; quality/fitted.json updated through `check_fitted.py --update`;
      - no step fitted to differences below the reproducibility floor (4 ulp web against desktop,
        docs/research/oracle-calibration.md);
      - no keep-matched branch (no special case that only preserves cells already matching).
- [ ] No gated cell regresses: these probes-numeric cells are ungated, so the gated gen-m9 cells T-2561, T-3013 and
      T-3311 matched stay bit-exact (A338, A212, A301 and the others in T-3311's Outcome), and no known-failure line
      is added to any eval/baseline file; any line that disappears is recorded with `make eval-baseline`.
- [ ] docs/excel-semantics/statistics.md's "Deep lower tail at a shape above 1" entry records the rule found (or the
      rules ruled out), citing cells of this case.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Shapes of 20 and more (T-3294), shapes below 1 (T-3265), the cycling iterate inside T-3311's cut (T-3403).
- t3311-fresh-sweep-2 A199 `CHISQ.INV.RT(1-2.09E-4,36)` (upper tail, T-3146).
- Cells within 15 digits that the chosen rule does not reach.

## Deferred (decision 0114)
`defer: phase-1b`: every cell this task owns is inside Gate A (decision 0188 R5), so it waits until Phase 1 exits.
