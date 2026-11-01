---
id: T-3797
title: "Gate A: classify the 51 M9 financial functions with no table or exemption, and file their table tasks"
milestone: M0
lane: harness
status: todo
priority: P2
size: M
tier: opus
depends_on: []
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
Every in-scope numeric M9 financial function that Gate A counts as missing ends up in one of two states:
- it has an `eval/accuracy/exempt.tsv` line backed by a decision record;
- or an open task names it in its title or Goal and will build its table.

## Context
- Gate A holds only with no missing table (SPEC-0029 "Gate A holds", decision 0188 R1(a)). Decision 0228 classified
  M5, M6 and M7. Its Consequences, and T-3551's Out of scope, leave M9's functions outside decision 0188's 18 for a
  separate classification task, filed once the solver pattern settled. It has: T-3535 (NPER, RATE) and T-3586 (IRR,
  XIRR, with array-literal arguments) are done, and their definitions are in
  `tools/src/rustsheet_tools/accuracy/defs/annuity.py` and `cashflow.py`.
- `rustsheet-eval accuracy` on main (56917e82c) prints `missing tables 271`. 51 of them are M9 financial functions
  (catalog `category: financial`) that no open task names:
  ACCRINT, ACCRINTM, AMORDEGRC, AMORLINC, COUPDAYBS, COUPDAYS, COUPDAYSNC, COUPNCD, COUPNUM, COUPPCD, CUMIPMT,
  CUMPRINC, DB, DDB, DISC, DOLLARDE, DOLLARFR, DURATION, EFFECT, FV, FVSCHEDULE, INTRATE, IPMT, ISPMT, MDURATION,
  MIRR, NOMINAL, NPV, ODDFPRICE, ODDFYIELD, ODDLPRICE, ODDLYIELD, PDURATION, PMT, PPMT, PRICE, PRICEDISC, PRICEMAT,
  PV, RECEIVED, RRI, SLN, SYD, TBILLEQ, TBILLPRICE, TBILLYIELD, VDB, XNPV, YIELD, YIELDDISC, YIELDMAT.
  Several open tasks fix these functions' engine cells (T-3731 CUMPRINC, T-3722 and T-3723 DURATION and PRICE, T-3620
  YIELD), but none builds a table. Recount before filing, because tasks merged in the meantime may change the list
  (`python3 -c` over `eval/runs/accuracy.json`'s `check.missing_tables`, joined with `eval/catalog/functions.yaml`).
- Questions that need judgment, each answered in the decision record:
  - **Integer day and coupon counts.** COUPNUM is a count. COUPDAYBS, COUPDAYS and COUPDAYSNC are day counts, but
    some bases give fractions (basis 3 is 365/2 days a period), and COUPDAYS basis 1 has its own rule (financial.md
    "COUPDAYS basis 1 counts from the chained next coupon"). COUPNCD and COUPPCD return serials. A function is `exact`
    only when no basis gives a floating-point result, as decision 0228 required for HOUR, MINUTE and SECOND.
  - **Date parameters and order constraints.** Settlement before maturity, issue before first coupon (financial.md
    "ODDFPRICE and ODDFYIELD need the first coupon on the maturity's schedule"). The generator can draw them as
    integer serials through a Definition's `domain` rule or a joint `draw` (`accuracy/model.py`). If a constraint
    cannot be expressed that way, file a generator task first, as T-3686 does for 2-D literals.
  - **Solvers** (YIELD, ODDFYIELD, ODDLYIELD, and the bond prices they invert). Follow the IRR and RATE pattern:
    `solver=True`, 300 points, and Excel's root choice cited from a semantics entry. Points with several roots are
    not drawn without one.
  - **Array arguments** (NPV, XNPV, MIRR, FVSCHEDULE). Each needs a one-row array literal (T-3586's support), or is
    `composite` with the eval suites named if the literal cannot drive it (financial.md "XNPV, XIRR and FVSCHEDULE
    beside their whole data").
  - **Excel's own arithmetic** (AMORDEGRC's and AMORLINC's rounding, DB's 3-digit rate, VDB's charge order:
    financial.md "AMORLINC and AMORDEGRC over many periods", "DB and VDB period cap", "VDB charge order"). These need
    a table. Where Excel is far from the math, note it for the table task, which then needs R3 marks (decision 0188
    R3: no agent adds an R3 case without an owner ruling).
  - **DOLLARDE and DOLLARFR** truncate the fraction argument and do decimal arithmetic. Decide between a table and
    `exact` from the semantics entries; with no entry, choose a table.
- `exempt.tsv` is protected (decision 0199, T-3379): every line added needs the decision record. A line naming a
  function that has a table fails the check, so check the open table tasks first. The three sibling classification
  tasks (T-3798, T-3799, T-3800) also add lines, so rebase before merging and keep the file sorted.
- Group the functions that need a table into table tasks of at most about seven functions each (M tasks, like
  T-3668 to T-3678), by family: annuity (FV, PV, PMT, IPMT, PPMT, ISPMT, CUMIPMT, CUMPRINC, PDURATION, RRI), rates
  (EFFECT, NOMINAL), depreciation, coupon dates, discount securities and T-bills, bonds, odd-period bonds, cash-flow
  arrays. Create each with `make new-task MILESTONE=M0 LANE=harness`, `tier: opus`, `spec:` SPEC-0029.

## Acceptance criteria
- [ ] A decision record (`make new-decision`) classifies each of the 51 functions (or the recounted list) as
      `exact`, `composite` or "needs a table (T-XXXX)", with a cited reason (a semantics entry or probe) for each
      exemption.
- [ ] `eval/accuracy/exempt.tsv` gains the exempt lines, sorted. `rustsheet-eval accuracy` reports `missing tables`
      lower by their number, with no stale or bad exemption.
- [ ] Every function classified "needs a table" is named in the title or Goal of an open task filed by this PR, with
      the judgment notes above that apply to it. A generator gap found here (an order constraint, a date draw) is
      its own task, and the table tasks that need it depend on it.
- [ ] `python3 scripts/tasks.py validate` passes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Building any table, definition or generator change (the filed tasks do that).
- The other M9 families: distributions (T-3798), sample statistics (T-3799), engineering and database (T-3800).
- Decision 0188's 18 functions (NPER, RATE, IRR and XIRR have tables already).
- Engine fixes for the functions' failing eval cells (their own tasks own them).
