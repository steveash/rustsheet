---
id: T-0087
title: "Gap-triager sweep: file an owner for every visibly-off Excel cell no open task owns (Phase 1 exit rule)"
milestone: M0
lane: harness
status: todo
priority: P1
size: M
tier: sonnet
depends_on: [T-0086]
---

## Goal
Every visibly-off Excel cell of a full eval run has an open, non-deferred owning task, so the Phase 1 exit rule "each
remaining failure, attributed to a function or not, has a filed task or a DEVIATION entry" (docs/milestones.md)
holds and no function below the Phase 1 rate has cells nobody will pick.

## Context
- T-0090 triage (eval of 2026-10-27 at `main` 7f6b373a; calipers, poi and corpus-bench not fetched; ownership after
  the triage's own edits): 478 attributed visibly-off cells of functions below the Phase 1 rate have no open,
  non-deferred owner. By function: TINV 52, ODDFYIELD 51, FREQUENCY 36, T.INV.2T 30, F.INV.RT 29, F.INV 28, FINV 23,
  GAMMA.DIST 20, IRR 17, NEGBINOMDIST 16, CONFIDENCE.T 16, FORMULATEXT 15, TEXT 14, CHISQ.INV 13, CHISQ.INV.RT 10,
  T.INV 8, POISSON 8, GAUSS 7, SUM 6, IMSQRT 6, FDIST 6, TTEST 5, ROWS 5, POISSON.DIST 4, PERCENTILE.INC 4,
  CHISQ.DIST.RT 4, AMORDEGRC 4, COUNTA 3, COLUMNS 3, COLUMN 3, CHISQ.DIST 3, and 1-2 cells each of XIRR, PERCENTILE,
  KURT, GAMMAINV, F.DIST.RT, F.DIST, CUMIPMT, CHIINV, BINOM.DIST.RANGE, BETAINV, YIELD, WEIBULL, T.DIST.2T, RRI,
  GAMMA.INV, FISHER, DEVSQ, BINOMDIST and BINOM.DIST. Also 365 unattributed visibly-off cells (`make unowned-off`)
  and unowned visibly-off probe cells of M0-M2 (1), M7 (13), M8 (35), M9 (109) and M10 (13).
  Handed to this sweep because their intended owner T-2681 finished before the triage landed (re-check them first;
  T-2681 may have fixed them): F.TEST gen-m9/b6593d7dd13f2dac Calc!A306, gen-m9/8f2ac25945af87da Calc!A119,
  gen-m9/fec39efaba9284c3 Calc!A36; FTEST gen-m9/58e3253bbd3a7a82 Calc!A141, gen-m9/a7bf0d8d81d33ba6 Calc!A81,
  gen-m9/f34f66fb7f81e673 Calc!A4. Likewise the cells the triage meant to stage on T-2779 (TEXT), T-2841 (IMREAL,
  IMSEC, IMCSCH, IMAGINARY), T-2339 and T-2834 (ROW, ROWS, COLUMNS, FORMULATEXT), T-2572 (IMLN) and T-2548
  (BESSELJ): those tasks were finished or blocked on `main` in the meantime, so check on a fresh eval which of
  their cells are still visibly off and give each a live owner.
  Several of these functions are one or two owned cells away from the rate once their owners land (for example
  GAUSS, PERCENTILE.INC, KURT, FISHER, RRI, DEVSQ, WEIBULL), so this sweep lifts functions directly.
- Use T-0086's `--all-off` listing once it lands (this task depends on it); the counts above are from the triage's
  one-off script and will have moved.
- Follow the gap-triager agent (.claude/agents/gap-triager.md): group cells by root cause, prefer extending an
  existing open task (add the case stem and cells to its text) over a new one, tag every new or extended task with
  its `campaign:` stage (semantics, accuracy or visible, decision 0114) and a `## Visible-parity stage` section,
  and never file a visibly-off cell under a `defer: phase-1b` task. A cell that only desktop Excel can settle goes
  to an M14 task (it then stops counting, T-0085).

## Acceptance criteria
- [ ] After the sweep, `python3 scripts/exit_owners.py --all-off --strict` (T-0086) exits 0 on a full `make eval`
      with every suite fetched, or the task's Outcome lists each remaining cell and why it has no owner.
- [ ] Every task created or extended carries a `campaign:` stage and names its cells or cases in the form the owner
      rule reads (case stem plus A1 names, or a stem alone on a line for a whole case).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing the cells (the owning tasks).
