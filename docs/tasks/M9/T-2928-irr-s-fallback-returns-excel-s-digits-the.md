---
id: T-2928
title: "IRR's fallback returns Excel's digits: the polynomial Newton from v = 0 lands 15 to 35 ulp from Excel (10 visibly-off cells)"
milestone: M9
lane: functions-financial
status: blocked
priority: P2
size: M
tier: opus
campaign: visible
depends_on: [T-2927]
---

## Goal
Where both fitted secant runs fail, IRR returns Excel's value to within half a unit of the 15th significant digit.
Today the fallback finds the right root but returns it correctly rounded, while Excel returns an iterate a few E-15
away from it. Excel is the inaccurate one here, so this means reproducing Excel's iteration and stop. Converging
further would not help.

## Context
Gap triage 2026-10-27 (`make eval` on origin/main 7dba3bfe). No open task owned the probe cells. Quirk entry:
docs/excel-semantics/financial.md "IRR's fallback when the secant fails" (the exact-root offsets are recorded there). T-2727 (done) fitted
which flows return and which are `#NUM!` (financial.md "IRR's fallback when the secant fails"). T-1686 (research)
owns the solver caps and overflow probes, T-2131 the secant iterate and T-2565 the large-root departures. None of
them covers the fallback's returned digits.

**Needs an oracle sweep.** Three distinct flow sets cannot separate the candidate stop rules, so this task depends
on T-2927 (probes-numeric/t2927-irr-fallback-sweep) and is not attempted before that case has values.

Owned cells, all visibly off except P!A21. Exact roots are from mpmath at 50 digits.
| case | cell | formula | Excel | ours | exact root |
|---|---|---|---|---|---|
| probes/irr-fallback-roots | P!A12, A13, A14, A15 | `IRR({42,1,1,-1},G)`, G = 0.5, 1, 0.01, 0 | -0.7467044766537371 | -0.7467044766537408 | -0.74670447665374079 |
| probes/irr-fallback-roots | P!A22, A23, A24 | `IRR({-400,1,1,1,1,1},G)`, G = 0.5, 1, 0.01 | -0.6737366751808405 | -0.6737366751808422 | -0.67373667518084225 |
| probes/irr-fallback-roots | P!A21 (1 ulp) | `IRR({0.3,-2.5,1.005},1)` | 6.9098085436343055 | 6.909808543634309 | 6.90980854363461213 |
| ironcalc-calc/FINANCIAL/IRR.xlsx | General!L17 | `IRR(A17:F17)` (= {-400,1,1,1,1,1}) | -0.6737366751808405 | -0.6737366751808422 | as above |
| gen-m9/31474c20fc1dafd6 | Calc!A173 | `IRR(Data!C1:E7)` | 661.4076391219855 | 661.4076391462188 | |
| gen-m9/31474c20fc1dafd6 | Calc!A271 | `IRR(Data!D5:E8)` | -0.7467044766537371 | -0.7467044766537408 | as {42,1,1,-1} |
IRR.xlsx L17 and the two gen-m9 cells are also listed in T-2131. Both take the fallback path today, so this task
owns them, and T-2131's Context notes the move.

Excel is about 3.7E-15 and 1.8E-15 above the exact root at the first two flows. That is the size of a Newton or
secant step stopped one iterate early with |dv| near 1e-7.5, or of a stop on the residual |P(v)|. The engine
(`irr_fallback`, `IRR_FALLBACK_STEPS` in crates/rustsheet-engine/src/functions/financial/financial_util.rs) runs
Newton on P(v) = sum c_i v^i from v = 0, in Horner order from the highest power. It stops when
|next - v| < 1e-7·max(|next|, 1) and returns `irr_from_discount(next)`. Candidates to score in the fit lab:
- return `v` instead of `next`;
- a relative or an absolute stop, and the threshold (1e-7, 1e-8, 1e-10);
- Newton in r = 1/v - 1, or in x = 1 + r, instead of v;
- P with ascending powers instead of Horner, or with the leading zeros kept;
- an NPV in r rather than in v for the residual;
- the start (v = 0, v = 1, the last secant iterate).
Add an `irr_fallback` family to tools/src/rustsheet_tools/fitlab/kernels/solvers.py next to `irr_secant_v`, over
the T-2927 sweep plus the cells above. Excel's selection rule (which flows return, which are `#NUM!`) is verified
and must not change.

T-2927's sweep (probes-numeric/t2927-irr-fallback-sweep, 150 flows, with t2927-irr-fallback-sweep.roots.tsv; financial.md
"IRR's fallback when the secant fails", Sweep): offsets from -10524 (A138) to +9036 (A144) ulps over the whole sweep,
with `{-284,1,1,1,1,1}` (A24) at +1865 ulps while its scaled and negated forms are within 1 ulp; scaling by 1E6 and negating the flows
both move them; the guess changes the digits on 5 of 15 rows. Excel is `#NUM!` where the fallback rule returns for
`{-N,1,1,1,1,1}` with N = 496 to 2000 (rows A26-A31) and for their 1E6-scaled and negated forms (A101-A112).
The sweep's generator is tools/src/rustsheet_tools/fitlab/irr_fallback_sweep.py (extend `families()` and rerun `case`
and, after the oracle fill, `roots`). It left out the gen-m9 shapes of 31474c20fc1dafd6 A173 (Data!C1:E7) and A271
(Data!D5:E8), whose flows need a snapshot of those Data ranges; add them if the fit needs more three- and four-flow
shapes. The guess column's digits (5 of 15 rows differ) conflict with irr-fallback-roots (no guess dependence on
`{42,1,1,-1}` and `{-400,1,1,1,1,1}`); settle where the fallback starts as part of the fit.

## Acceptance criteria
- [ ] The fit lab scores each candidate above on probes-numeric/t2927-irr-fallback-sweep and the owned cells,
      reporting bit-exact, within-4-ulp and within-15-digits counts. financial.md "IRR's fallback when the secant
      fails" records the counts, rejected candidates included.
- [ ] Either a candidate brings every owned cell within half a unit of the 15th significant digit, and the engine
      implements it: the lines for probes/irr-fallback-roots P!A12:A15, A22:A24, ironcalc-calc IRR.xlsx General!L17
      and gen-m9/31474c20fc1dafd6 Calc!A173 and A271 leave eval/baseline/, and no baseline gains a line. Or the
      task is set `blocked`, "Blocked because: time-boxed (decision 0056)", with the candidates listed.
- [ ] The fallback stays bounded (`IRR_FALLBACK_STEPS`); the `#NUM!` cells of probes/irr-fallback-roots,
      irr-guess-edges and irr-guess-edges-2 still pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The secant path's digits and large-root flows (T-2131, T-2565); the `#DIV/0!` cells (T-2802).
- XIRR (T-2566, T-2611).

## Visible-parity stage
Decision 0083/0114, stage `visible`: Excel is the inaccurate one here (mpmath puts ours within 1E-16 of the exact
root and Excel 2E-15 to 4E-15 off it), so the target is Excel's own iterate, time-boxed under decision 0056. The task
is done when every owned cell is within half a unit of the 15th significant digit of Excel's value. If P!A21 or any
cell is still a few ulp off then, it moves to a follow-up task filed with `defer: phase-1b` and no campaign key, named
in the Outcome.

## Attempt 1 (decision 0056)
2026-10-28, opus run. Added the fit-lab family `irr-fallback` (kernels/irr_fallback.py `irr_fallback_candidates`,
`irr_fallback_newton`; `uv run rustsheet-fitlab irr-fallback`) over the T-2927 sweep and probes/irr-fallback-roots
(212 cells, `#NUM!` included). Every candidate the Context lists is scored, and none raises the bit-exact count: the
engine is 72 bit-exact, the best fallback variant (Newton in v from 0 with |P| < 1E-7) 69 against the same Newton's
63. The counts, the rejected scratch candidates (damped secants past v <= 0, a 1,200-start fixed-start secant grid in
v, r and x) and four findings are in financial.md "IRR's fallback when the secant fails", Fit. No engine change: the
owned cells stay in eval/baseline/.
Next attempt, from the findings: a secant (not Newton: the sign of Excel's error rules out a plain last Newton step on
the three anchor flows) with a signed first offset and a |P| < 1E-7 stop, whose start is guess-dependent on
`{0.3,-2.5,1.005}` and `{2,1,1,-1}` but not on `{42,1,1,-1}` or `{-400,1,1,1,1,1}`; and a rule for the `#NUM!` of the
large-first-flow rows A26-A31 and A101 (finding 5: not a Newton step cap and not a root bound). A second attempt that does not raise the
bit-exact count blocks the task (decision 0056).

## Attempt 2 (decision 0056)
2026-10-29, opus run, scratch harness only (the fit lab's `irr_secant_v` before each candidate; it matches 62 of the
212 cells with the engine's Newton fallback, where the engine itself matches 72, so counts compare within this run).
No candidate raises the bit-exact count beyond the |P| < 1E-7 test attempt 1 already scored. Candidates, findings and
counts are in financial.md "IRR's fallback when the secant fails", Fit, attempt 2. No engine change: the owned cells
stay in eval/baseline/.

## Blocked because: time-boxed (decision 0056)
- Two attempts. Neither raises the bit-exact count on the owned cells or the sweep. Attempt 1 scored every candidate in
  the Context (Newton in v, r and x; starts at 0, 1 and the guess; stop tests; summation order) and a 1,200-start
  secant grid. Attempt 2 scored Newton in d = 1 - v, finite-difference Newton, a secant from v = 0 in v and in d, and
  replacements for a secant iterate at v <= 0 (2·v1, 1.5·v1, 10·v1, 1.1·v1, v1 + 0.1, v1 + 2|v1 - v0|). The best
  attempt-2 variant (Newton in d with |P| < 1E-7, v held to 2^-53) gains only the `#NUM!` rows A103-A111 that
  attempt 1's residual test already gave, plus last-bit noise. `{42,1,1,-1}` stays 34 ulp off.
- What the next attempt starts from: (1) Newton in any variable is ruled out. Negating every flow leaves each Newton
  iterate bit-for-bit unchanged, but Excel's negated rows differ from their positive rows by up to 1,251 ulp
  (A74 against A12). So the step depends on the sign of P, as a secant's signed first offset does.
  (2) Where v < 1, Excel's results lie on a 2^-53 grid in v, as T-2565 found for the secant: 29 of the 32 distinct values, where chance gives 3.3.
  So the iterate is held as d = 1 - v or in a fixed-point form.
- Reopen on new evidence: T-3117 probes the same families scaled by exact powers of two and negated. That separates an
  absolute residual test from the iteration's arithmetic, which a 2^j scale does not change.
