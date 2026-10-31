---
id: T-3346
title: "Record the owner's Phase 1 two-gate exit rulings"
milestone: M0
lane: quality
status: done
priority: P0
size: M
tier: opus
depends_on: []
---

## Goal
The repository records the owner's 2026-10-31 rulings (decision 0188): Phase 1 exits on an accuracy gate (Gate A) and
user-representative fifteen-digit parity (Gate B); the probe and ironcalc-calc rows count only semantic and
Gate-A-failing cells; Excel is followed except where it is more than 1e-4 off the math (`MATH-OVER-EXCEL`); numeric
fits are guarded against overfitting; last-digit sweep work is Phase 1b; and the tasks that implement all of it are
filed.

## Context
Owner's 2026-10-31 assessment of a fresh eval at `main` 921ed6ad (decision 0188 has the numbers): 97% of the
visibly-off numbers are fit-lab sweeps of six distributions, 99.5% of the differing numeric cells agree to at least 12
digits, while real bugs that users hit (IRR and RATE `#NUM!`, T.INV.2T's cancellation, CHISQ.INV at a huge df, BETA.INV
near 1, BINOM at a large n, XIRR, F.DIST at a tiny x) are not what the exit measures, and the probe-tuned inverses
generalize badly to fresh points.

## Acceptance criteria
- [x] Decision 0188 records rulings R1-R7 and the evidence; it amends 0052, 0114, 0118, 0143 and 0146 (each carries a
      one-line pointer).
- [x] CLAUDE.md directive 1 points at decision 0188 R3; docs/milestones.md's Phase 1 section states Gate A, Gate B,
      R1(c), R4, R5 and R7; docs/excel-semantics/README.md has the `MATH-OVER-EXCEL` status, and statistics.md and
      financial.md have the three initial entries.
- [x] R6: the three COMPLEXs.xlsx cells are lines of eval/milestones/exit-exceptions.tsv and phase1-exceptions.tsv.
- [x] The tasks of the rulings are filed with `make new-task`; existing tasks are retagged, deferred or noted.
- [x] `python3 scripts/tasks.py validate`, `scripts/check_decisions.py`, `scripts/check_governance.py origin/main`,
      `make typos` and `make hygiene` pass.

## Out of scope
- Any engine, harness or eval change: the filed tasks do that.
- Unblocking blocked tasks or changing in-progress ones (owner instruction): they get notes only.

## Outcome
Branch `agent/quality/T-3346-phase1-gates` (owner-directed; the owner reviews and merges).

New tasks:

| Id | Title (short) | Lane | Priority | Tier | Campaign | Depends on |
|---|---|---|---|---|---|---|
| T-3347 | IRR `#NUM!` near 0 or -10% | functions-financial | P0 | opus | accuracy | – |
| T-3348 | RATE `#NUM!` with fv ≠ 0 | functions-financial | P0 | opus | accuracy | – |
| T-3349 | T.INV.2T / TINV cancellation at p near 1 | functions-stat | P0 | opus | accuracy | – |
| T-3350 | CHISQ.INV / CHISQ.INV.RT `#NUM!` at df ≥ 2e7 | functions-stat | P0 | opus | accuracy | – |
| T-3351 | BETA.INV `#NUM!` at p ≈ 1-1e-8, small b | functions-stat | P0 | opus | accuracy | – |
| T-3352 | BINOM.DIST / NEGBINOM.DIST at a large n | functions-stat | P0 | opus | accuracy | – |
| T-3353 | XIRR two-flow long spans | functions-financial | P0 | opus | accuracy | – |
| T-3354 | F.DIST at a tiny x, huge d2 | functions-stat | P0 | opus | accuracy | – |
| T-3355 | SPEC: Gate A | harness | P0 | opus | – | – |
| T-3356 | Gate A slice 1: format, reader, make check gate | harness | P0 | opus | – | T-3355 |
| T-3357 | Gate A slice 2: mpmath generator, sampler, fill | harness | P0 | opus | – | T-3355 |
| T-3358 | Gate A slice 3: first tables, R3 points | harness | P0 | opus | – | T-3356, T-3357 |
| T-3359 | Gate B, R1(c) rows, Phase 1b evidence, R7 | harness | P0 | opus | – | T-3355 |
| T-3360 | Fit-lab hold-out split (R4.1) | harness | P1 | opus | – | – |
| T-3361 | `FITTED(...)` tag and ratchet (R4.3) | quality | P1 | opus | – | – |
| T-3362 | Reviewer and skill text (R4.2, R4.4, R4.5) | quality | P1 | opus | – | – |
| T-3363 | Excel's t, F and chi-squared inverse | functions-stat | P1 | opus | visible | T-3360 |
| T-3364 | Extrapolated fits fall back to math | functions-stat | P1 | opus | – | T-3355, T-3361 |
| T-3365 | Re-validate tuned form selectors | quality | P2 | opus | – | T-3360, T-3361 |
| T-3366 | POISSON.DIST and BINOM.DIST generalization | functions-stat | P1 | opus | – | – |
| T-3367 | NPER at 0 < rate < 1e-14 follows the math (R3) | functions-financial | P2 | opus | – | T-3355 |

Existing tasks:
- `campaign: semantics`, priority raised (one Context line each): T-2816 (P2→P1), T-2824 (P0), T-3111, T-3112,
  T-2763, T-2899, T-2911, T-1875, T-1925 (P3→P1), T-3211, T-3037 (P2→P1). T-3218, T-3210 and T-1736 are done and were
  left alone.
- T-0967: T-0969 removed from `depends_on` (time-boxed under decision 0056, so it would hold T-0967 forever; T-3061's
  route needs no T-0969 CDF). T-2511 kept: it is ready now and is the same inverse code (T-0098's sequencing).
- R5 `defer: phase-1b`: T-2510, T-2790, T-2933, T-3035, T-3158, T-3173, T-3174, T-3199, T-3209, T-3220, T-3244, T-3250,
  T-3323, T-3331. T-2510 left the `depends_on` of T-2511, T-2146 and T-2633 so they are not parked behind it.
- Notes only: T-2586 (in progress), T-0969 (blocked; also names its H530 cell instead of the whole
  stat-argument-modes case), T-2774 and T-2536 (blocked; R6 settles the blocker).
- T-2425 owns probes/stat-argument-modes H266-H281 (the FORECAST.ETS cells), now owned by this M14 task alone.
- T-3006 done: R6 answers it.
- Not filed: the M7 owner for the t3181-collation-prefix-arrays B cells. At 921ed6ad B3, B4, B5, B7, B8, B11 and B12
  failed; on `main` 0c917f9f all seven pass (T-3210 and T-3317 merged), so no cell is left to own.
- Review round 1 (2026-10-31): R3 regions bounded per function (the tiny-p quantiles cover only the cited point until
  T-3358's points fix them; NPER 0 < rate < 1e-14 minus the `#DIV/0!` band; GAMMA.INV's closed-form branch with the
  chi-squared functions through it), R3 keeps directive 5 and needs an owner ruling per new case, bug-task targets are
  Excel's results with the Gate A envelope as a floor, and the Related lines name the prior solver and inverse tasks.
