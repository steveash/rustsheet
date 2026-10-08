---
id: T-1006
title: "Oracle sweep for T-0925: NORM.S.INV at NORM.S.DIST(k), tails to 1e-300"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
T-0925 can fit Excel's NORM.S.INV formulation against an oracle-filled sweep instead of the few values cached in Excel-authored workbooks.

## Context
Split from T-0942 (one sweep per M9 fitting task). Build a spec case `eval/suites/probes-numeric/cases/m9-norm-s-inv.case.json` (`rustsheet-case/v1`, sheet `Calc`, one formula per cell, a `hypotheses` label per cell naming the inputs, as in `m9-fin-power.case.json` from T-0942) and fill it with `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric` (excel-probe skill).

Grid: p = NORM.S.DIST(k,TRUE) for k in -38..8 step 0.25, plus literal tails p = 1e-300, 1e-200, 1e-100, 1e-50, 1e-20, 1e-10, 0.02425 ± 1e-6 (AS241/Acklam region edges), 0.5 ± 1e-9, and 1 - 1e-k; include NORMSINV and NORM.INV(p,0,1) on the same p.

## Acceptance criteria
- [ ] `m9-norm-s-inv.case.json` and its `.oracle.json` sidecar are committed; no pending cells.
- [ ] Quirks visible in the sweep (for example where the engine already differs, `rustsheet-eval show probes-numeric/m9-norm-s-inv.case.json`) are recorded in docs/excel-semantics/numbers.md with the case as evidence.
- [ ] T-0925 links the case and its `depends_on` no longer lists T-0942.
- [ ] Fixtures hold only formulas and Excel values; no RUSTSHEET_ORACLE_* values in logs or commits.
- [ ] If the oracle is unavailable, the task is `status: blocked` with the reason.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fitting or implementing the formulation (T-0925 does that).

## Outcome
Added `probes-numeric/m9-norm-s-inv` (648 cells) with its excel-graph oracle sidecar (no pending cells). 447 of 639 cells differ from the engine (438 ulp, 3 close, 6 flushed-tail `#NUM!`); quirks in numbers.md. T-0925 already depends on T-1006 (not T-0942). Eval gates unchanged (probes-numeric is ungated).

## Handoff (resumed)
Tried: built `probes-numeric/m9-norm-s-inv` (648 cells incl. A640-A648 flush probes), filled from excel-graph, `make check` green; security, architecture, quality approved. The fidelity reviewer requested changes twice on the numbers.md section "oracle sweep (T-1006)" (data is sound).
Open items from round 2 (all doc-only except where noted):
- A648 `NORM.S.INV(2.3E-308-1E-309)` is not a subnormal p: the literal 1E-309 reads as 0 (see numbers.md T-0189/T-1007), so it equals NORM.S.INV(2.3E-308). Relabel it, or replace it with a subtraction-built subnormal such as `2.3E-308-2E-308` (remove the old sidecar for this case only, then `rustsheet-oracle fill`). Only A647 shows NORM.S.INV accepting a subnormal p.
- "0-2 ulp for k <= 2" omits k = +-1 (4 doubles apart, 2 ulp of 1.0): list them or say how ulp is measured.
- A640-A645 engine failures are a NORM.S.DIST non-flush issue and A647 is the `-` flush (T-1027), not T-0925: point them at T-1027 and a NORM.S.DIST task (T-0968 / T-1040 or a new follow-up); true Phi(-37.6) ~ 1.07E-309 is subnormal and Phi(-37.5) ~ 4.6E-308 is normal, cutoff near k = -37.52, keep it marked unprobed.
- A644/A645 (numeric 0.0) are not flush evidence (Graph reports 0.0 for a kept subnormal); the claim rests on A640-A642.
Then rerun `reviewer-excel-fidelity`, set `status: done` (keep the Outcome below, update its counts to 648 cells), merge on green CI.
