---
id: T-2693
title: "Probe (x)=0 companions for the 89 cells where Excel for the web shows 0 and the engine keeps a subnormal (EXP, BINOM.DIST, NEGBINOM.DIST, BETA.DIST, GAMMA.DIST)"
milestone: M5
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
For every cell where the excel-graph value is exactly 0 and the engine returns a nonzero subnormal, Excel's own evidence
says whether Excel kept a subnormal or returned a true 0: each such formula gets a `=(<formula>)=0` companion cell in
its case, filled from the oracle. Cells whose companion is FALSE then score as passes under decision 0042; cells whose
companion is TRUE (Excel flushes) become fix tasks.

## Context
Gap triage 2026-10-25 (`make eval`). Microsoft Graph reports a subnormal as 0.0, so an excel-graph 0 cannot tell a kept
subnormal from a true 0 (decisions/0042-an-excel-graph-0-scores-a-kept-subnormal.md). The harness passes such a cell only
when the same case has a `=(<formula>)=0` cell whose excel-graph value is FALSE (`graph_hides_subnormal` in
crates/rustsheet-eval/src/runner.rs). docs/excel-semantics/numbers.md "Subnormal results become 0" says Excel keeps a
subnormal from `+`, `-`, SUM, EXP, MIN, ABS and SIN; the distribution functions below are unprobed.
These 89 cells have no companion, so they count as visibly off (0 against nonzero) and no open task owns them:
- probes-numeric/t2513-r3-binom-pmf-n.case.json (64): Calc!B1729:B1792 (`BINOM.DIST(4000000,10000000,p,FALSE)`, ours e.g. 1.2065E-320)
- probes-numeric/t2513-verify-r3-binom-pmf-h.case.json (4): Calc!A229, A238 (BINOM.DIST), B229, B238 (NEGBINOM.DIST)
- probes-numeric/t2513-r3-m1-residual-hold.case.json (3): Calc!D863, D875, D957 (`BETA.DIST(A,B,C,FALSE)`)
- probes-numeric/t2586-r5-f-edges-a.case.json (6): Calc!A927:A932 (`EXP(-720)`, `EXP(-740)`; ours 2.0322308024E-313, 4.2E-322)
- probes-numeric/t2586-r5-f-edges-hold.case.json (3): Calc!A293:A295 (`EXP(-730)`)
- probes-numeric/t2586-verify-r5-f-edges-hold.case.json (5): Calc!A55, A59, A63, A67, A71 (`EXP(-725)`)
- probes-numeric/m9-gamma-dist-a500.case.json (1): Calc!A22 (`GAMMA.DIST(50000000,500,1000000,FALSE)`)
- gen-m9/165db58d1540f014.case.json (1): Calc!A65 (EXPON.DIST), gen-m9/899f9555e67da2d6.case.json (1): Calc!A386 (LOGINV),
  gen-m9/eccd03df42e40006.case.json (1): Calc!A96 (WEIBULL)
List them again with a filter over `eval/runs/latest/report.json`: expected `{"t":"n","v":0}`, actual a number with
`0 < |v| < 2.2250738585072014E-308`.

How: follow the `excel-probe` skill. Add one companion cell per probes-numeric formula above to its own case (the rule
needs the companion in the same case), then refill those cases with the excel-graph oracle (`cd tools && uv run
rustsheet-oracle fill ...`); every other cell's value must come back unchanged (check the refilled sidecar against the old
one). The three gen-m9 cells are generated, so do not edit their cases: copy their formulas and Data cells into one new
probes case with companions and record the answer.

## Acceptance criteria
- [ ] Every probes-numeric cell listed above has a filled `=(<formula>)=0` companion in its case, and the refill changed
      no other cell's expected value.
- [ ] Visible parity: each listed probes-numeric cell either passes under decision 0042 (companion FALSE) or, where the
      companion is TRUE, is owned by a new fix task (engine flushes that function's result) filed with the cells listed.
- [ ] A new probes case answers the three gen-m9 formulas (companion values recorded); numbers.md "Subnormal results
      become 0" lists which of EXP, BINOM.DIST, NEGBINOM.DIST, BETA.DIST, GAMMA.DIST, EXPON.DIST, LOGINV and WEIBULL keep a
      subnormal, with the cells cited.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing decision 0042 or the harness rule (a companion in another case does not count).
- The engine fixes for any function Excel turns out to flush (follow-up tasks).

## Visible-parity stage
Decision 0083: these 89 cells read as 0 against a nonzero engine value, so they are visibly off until the evidence says
which side is right. The second acceptance criterion is the visible-parity target (decision 0114, `semantics`: the
question is which value kind Excel returns).
