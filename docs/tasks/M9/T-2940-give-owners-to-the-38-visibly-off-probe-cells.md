---
id: T-2940
title: "Give owners to the 38 visibly-off probes/t2436, ftest-decomposition and t2829-ftest-a306-shapes cells that no open task names (22 of them unattributed)"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Each of the 38 visibly-off cells below is named, by its probe case path and A1 cell, in the open task that fixes its root
cause. Where no open task has that root cause, a new task names it. After this, none of the 38 relies on a task that
names only a gen-m9 hash with the same stem.

## Context
- Planner run 2026-10-27 on the `make eval` report of that day (commit ece922e6; calipers, corpus-bench and poi not
  fetched). `scripts/exit_owners.py --unattributed` reports 651 unowned cells. If I simulate T-2731's `<stem>.case.json`
  rule (patched copy of `names`), 22 are left. These are the 22 unattributed cells below (decision 0132). The other 16
  are attributed visibly-off cells of the same cases. A task "owns" those 16 only by coincidence, because it names the
  gen-m9 hash or the A1 cell for another case (T-2880's shared-stem leak). No open task names any of the 38 with its
  `probes/...` path (checked with `rg`, open statuses only).
- T-2731's Out of scope names this work: "Companion cells that no task names. The probes/t2436 ROWS/COLUMNS cells in
  columns T and U ... need a bookkeeping task, not a tool rule." When T-2880 lands, it lists the shared-stem cells as
  unowned. Doing this task first keeps that list short.
- Proposed owners. Check each one against the cell's formula and the owner's rule before you write the line. Read the
  cell with `make eval-show CASE=probes/<case>`. Each row lists the cells, then the proposed owner:
  - probes/t2436/4250f880ab640c49 Calc!A121, T121, U121 (`BETAINV(..,MAKEARRAY(4,DATE(1952,..),..),..)`, Excel `#SPILL!`,
    4 and 51638, ours `#ERROR!`): T-2607 (engine, same formula as gen-m9 A237).
  - probes/t2436/7674c89c824aca43 Calc!A31, T16, U16, T31, U31; probes/t2436/7c20351562a2d71d Calc!A196, A241, T196,
    U196, T241, U241; probes/t2436/9cca17438a7e6f8a Calc!T31, U31 (a one-cell TAKE under an operator or an element-wise
    call is a scalar, so `ROWS(<error>)` is the error, not 1): T-2519.
  - probes/t2436/abc2939ebd1e2365 Calc!A1, A16, T1, U1, T16, U16 (`F.INV.RT(1E-200,1,1.999)`, Excel 3.696066238512854E+150,
    ours `#NUM!`): T-2808.
  - probes/t2436/e85daedce5c2675e Calc!A31, A46 (`COUNT(PIVOTBY(..,LAMBDA(y,y)),..)`, Excel 142, ours 167):
    T-2460, which already describes gen-m9 e85daedce5c2675e A35 as "the same COUNT plus 2".
  - probes/t2436/8d3588d8df0812df Calc!A61 (`AVERAGEA(GROUPBY(..,_xleta.PRODUCT,0,2),..)`): T-2460 or the GROUPBY task
    that owns gen-m9 8d3588d8df0812df's GROUPBY cells. Decide which one.
  - probes/t2436/4ab2ba98d26c8a5e Calc!A256 (`FISHER(Data!E4/Data!$D2)`, Excel 2.2204460492503126E-16, ours 3.33E-16):
    T-2437, which names the same formula at gen-m9 A395.
  - probes/ftest-decomposition Calc!A8, C8, E8, F8 (`2*MIN(F.DIST(..),F.DIST.RT(..))`): T-2219 (F.DIST to 15 digits) or
    T-2765 (F.TEST's two-tailed F). Trace which component moves the result past half a unit first.
  - probes/t2829-ftest-a306-shapes S!A1, A2, A7, A8, A9, A10, A11, A12 (F.TEST and `2*F.DIST.RT(..,23,12)` near
    1.24E-69): T-2219, T-2924 (betaln fit, probes/t2829-ftest-ln-beta-grid) or T-2765. T-2829 (done) probed the case.
    Its Outcome says which kernel piece is open.
- A cell whose root cause no open task covers gets a new task, filed with `make new-task`, that names it.
- Ownership rule: decision 0096 and scripts/exit_owners.py `owned`. Write each case as `probes/t2436/<hash>` with its
  cells on the same line. A bare hash is shared with gen-m9 (T-2880).

## Acceptance criteria
- [ ] Every one of the 38 cells is listed, as `probes/<case> Sheet!A1` on a line with its case path, in exactly one open
      task whose rule explains it. The PR description gives the cell-to-task table. New tasks are listed there too.
- [ ] `python3 scripts/exit_owners.py --unattributed` no longer lists any of the 22 unattributed cells above. Give the
      count before and after in the Outcome (the count before is 651).
- [ ] Only docs/tasks files change; `python3 scripts/tasks.py validate` and `make check` green; reviewers approve.

## Out of scope
- Fixing any of the cells.
- The exit-owner tool rules (T-2731, T-2880) and the gen-m9 FORECAST.ETS cells T-2425 lists (T-2731 makes them owned).
