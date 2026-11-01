---
id: T-3079
title: "Route the 588 visibly-off T-0969 sweep cells (t0969 t-tail-grid, t-pdf-runs, beta-density-b, beta-switch; m9 t-dist-*) from time-boxed T-0969 to the kernel tasks that compute them"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
Every visibly-off cell that today has blocked, time-boxed T-0969 as its only open, undeferred owner is named by the
live kernel task whose branch computes it (T-2509 for the Student-t tails, T-2585 for the Student-t density, T-2507 for
the beta density with both shapes above 1), or by a new campaign task when none of them does. T-0969 keeps only the
cells nothing else explains. Then those kernel tasks measure these cells when they land, and the campaign queue
reaches them.

## Context
- Gap triage 2026-10-28 (`make eval` on `main` bd7679c6b; calipers, poi and corpus-bench not fetched, none of these
  cells are in them). `python3 scripts/exit_owners.py --all-off` reports a single unowned visibly-off cell. But when the
  owners are split by status, 588 visibly-off Excel cells are owned *only* by T-0969 (`status: blocked`, "time-boxed
  (decision 0056)"). T-0969's block says a dependency landing does not reopen it. Its 2026-10-24 update hands the
  density to T-2585 and the tails to T-2509 ("This task stays blocked until T-2585 and T-2509 land"). But neither task
  names these cases or cells. T-2981, which routed the incomplete-beta sweeps, skipped them because T-0969 already
  "owned" them.
- The cells, by case and branch. Shapes come from the case JSON; the routing is a first guess, so verify each one.
  t0969 `x` stands for the case probes-numeric/t0969-x, m9 `x` for probes-numeric/m9-x, and `gen-m9/17c110a5*` for
  the gen-m9 case whose stem starts with those hex digits. This task spells no case in full on purpose: under the
  ownership rule below, it would then own these cells itself and trivially meet its own third criterion.
  - Student-t right tail and two-tailed CDF, T-2509 ("Student t tails through the ported kernel"):
    t0969 `t-tail-grid` 227 T.DIST.RT cells (a df 2 to 1000 grid and three runs of consecutive doubles;
    e.g. Calc!B1001 `=T.DIST.RT(A1001,50)` Excel 7.690416172835412e-7, ours 7.690416172835406e-7);
    m9 `t-dist-rt` 4 cells (`T.DIST.RT(100,1)` Excel 0.003182992764908255, ours 0.0031829927649082573;
    `T.DIST.RT(5,100)` Excel 1.2250867067519005e-6, ours 1.2250867067519037e-6); m9 `t-dist-2t` 6 cells
    (`T.DIST.2T(100,1)` Excel 0.00636598552981651, ours 0.006365985529816515); t0969 `t-df-ratio` 2 T.DIST.RT
    cells (`T.DIST.RT(0.5,A13)` Excel 0.3127156705406078, ours 0.31271567054060817); gen-m9/17c110a5* Calc!A159
    `T.DIST.2T(ODD(7),1E+10)` and gen-m9/91e167cd* Calc!A292 `T.DIST.2T(GEOMEAN(..),1)` (unattributed).
  - Student-t density, T-2585 ("Loader's dt for df >= 3 and the df 1 and df 2 closed forms"): t0969 `t-pdf-runs`
    146 cells (df 3: 29, 4: 9, 5: 14, 7: 48, 10: 46; e.g. `T.DIST(A11337,3,FALSE)` Excel 0.20892483134574413, ours
    0.208924831345744); m9 `t-dist-pdf` 25 cells (df 2 to 1E6; `T.DIST(100,30,FALSE)` Excel
    2.9682918578588325e-40, ours 2.968291857858848e-40); t0969 `t-df-ratio` 1 density cell (df 3).
  - Beta density with min(a, b) > 1, T-2507 (Loader's saddle point, its "Cases owned (T-2981)" list):
    t0969 `beta-density-b` 129 cells, all column D, `BETA.DIST(x,1.5,b,FALSE)` with b > 1 (e.g. Pdf!D10
    Excel 0.19972198869469973, ours 0.1997219886946995); t0969 `beta-switch` 46 cells (a and b 6.5 to 9.5,
    x 0.01 and 0.03; e.g. `BETA.DIST(0.01,A296,B296,FALSE)` Excel 1.9190855269410299, ours 1.9190855269410287). These
    probes were built to isolate Excel's 1/B(a, b) prefactor near b = 8. If T-2507's Loader form does not move them,
    the owner is the ln B fit (T-2924, blocked; T-3015 probes the dense grid), and the cells need a live owner
    with a campaign key.
- T-2509 is itself `blocked` on its branch (agent/functions-stat/T-2509-t-tails-bratio): the baseline update needs
  decision 0083's ulp-regression allowance. Routing the tail cells there is still right, since that branch computes them. Record
  on its branch's eval which of the 239 tail cells it fixes (the branch's Outcome gives gen-m9 -70, ironcalc-calc -8,
  probes -4, and no probes-numeric count).
- Ownership rule: decision 0096 and `scripts/exit_owners.py` `owned`; name each case by its `probes-numeric/<stem>` path.
  A line that names a stem and no cell owns the whole case (decision 0136). Use that only where every visibly-off cell
  of the case goes to the same owner.

## Acceptance criteria
- [ ] Each case above is named, with its visibly-off count, in a "Cases owned (T-3079)" list in the task that computes
      it (T-2509, T-2585 or T-2507). The PR description has a case-to-task table and, for each case, the
      `make eval-show CASE=probes-numeric/<stem>` evidence or kernel trace that decided it.
- [ ] Cells that none of the three explains (for example, the beta-switch cells if they are the ln B prefactor) get a
      new task with a `campaign:` key and a `## Visible-parity stage` section, filed with `make new-task`.
- [ ] T-0969's text no longer is the only open, undeferred owner of any of the 588 cells. To check, rerun the
      status-split count from this task's Context (open owners of each `--all-off` cell, minus blocked and deferred
      tasks). Give the before and after counts in the Outcome.
- [ ] Only docs/tasks files change; `python3 scripts/tasks.py validate` and `make check` green; reviewers approve.

## Out of scope
- Fixing any kernel (T-2509, T-2585, T-2507 and their chain).
- Bit-level residue of these sweeps (Phase 1b tasks).
- Reopening T-0969 (its block lists what reopens it).

## Visible-parity stage
Target (decision 0114): every one of the 588 visibly-off cells has an owner the campaign queue can pick, not only a
time-boxed blocked task. This task changes no value.
