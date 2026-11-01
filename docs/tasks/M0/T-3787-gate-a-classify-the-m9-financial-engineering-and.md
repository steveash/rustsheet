---
id: T-3787
title: "Gate A: classify the M9 financial, engineering and database functions with no table or exemption, and file their table tasks"
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
Every in-scope numeric M9 financial, engineering and database function that Gate A counts as missing ends up in one
of two states. Either it has an `exempt.tsv` line backed by a decision record, or an open task names it and will
build its table.

## Context
- Decision 0228 (T-3551) classified M5, M6 and M7 and left M9 to its own classification task "once the M9 table
  slices settle the solver and distribution patterns". For finance those slices are done: T-3535 (NPER and RATE, the
  annuity residual in `defs/annuity.py`) and T-3586 (IRR and XIRR, array literals and dated flows in
  `defs/cashflow.py`). No task on main or on any pending branch classifies the rest. T-3786 does the statistical
  functions.
- On main (fafb6c6f) these 72 have neither a table nor an exemption:
  - financial (51): ACCRINT, ACCRINTM, AMORDEGRC, AMORLINC, COUPDAYBS, COUPDAYS, COUPDAYSNC, COUPNCD, COUPNUM,
    COUPPCD, CUMIPMT, CUMPRINC, DB, DDB, DISC, DOLLARDE, DOLLARFR, DURATION, EFFECT, FV, FVSCHEDULE, INTRATE, IPMT,
    ISPMT, MDURATION, MIRR, NOMINAL, NPV, ODDFPRICE, ODDFYIELD, ODDLPRICE, ODDLYIELD, PDURATION, PMT, PPMT, PRICE,
    PRICEDISC, PRICEMAT, PV, RECEIVED, RRI, SLN, SYD, TBILLEQ, TBILLPRICE, TBILLYIELD, VDB, XNPV, YIELD, YIELDDISC,
    YIELDMAT;
  - engineering (12): BESSELI, BESSELJ, BESSELK, BESSELY, BIN2DEC, CONVERT, HEX2DEC, IMABS, IMAGINARY, IMARGUMENT,
    IMREAL, OCT2DEC (ERF and ERFC are T-3784);
  - database (9): DAVERAGE, DMAX, DMIN, DPRODUCT, DSTDEV, DSTDEVP, DSUM, DVAR, DVARP.
  Recount before filing: merges in the meantime may change the list.
- Questions that need judgment:
  - **Date-driven bond functions** (COUP*, PRICE, YIELD, DURATION, MDURATION, ODD*, ACCRINT*, TBILL*, DISC,
    INTRATE, RECEIVED, PRICEDISC, PRICEMAT, YIELDDISC, YIELDMAT). Their arguments are serial dates plus a basis, and
    the day counts are integer arithmetic (financial.md). Decide which have a floating-point kernel a table checks
    (PRICE, YIELD's solver, DURATION) and which are `exact` (COUPNUM, and COUPDAYS where its basis gives whole days),
    citing the entry for each. YIELD and ODDFYIELD are solvers: read financial.md "IRR, RATE and XIRR stop early" for
    whether a math root exists to tabulate, as T-3535 did for RATE.
  - **Annuity family** (PV, FV, PMT, IPMT, PPMT, CUMIPMT, CUMPRINC, ISPMT, NPV, XNPV, MIRR, PDURATION, RRI, EFFECT,
    NOMINAL, FVSCHEDULE). They should reuse `defs/annuity.py` and `defs/cashflow.py`, not copy them. Where
    financial.md shows Excel past R3's 1e-4 against the math, say so, so that the table task plans those points as
    R3 candidates. CUMIPMT and CUMPRINC at a tiny rate (T-3731) are only ulps off, which is inside the envelope's
    reach and not an R3 case.
  - **Depreciation** (DB, DDB, SLN, SYD, VDB, AMORDEGRC, AMORLINC): DB rounds its rate to three decimals and
    AMORDEGRC rounds per period (financial.md). Decide whether the math is the documented algorithm at working
    precision.
  - **Text-argument engineering functions**: BIN2DEC, HEX2DEC and OCT2DEC parse text into integers (`exact`?), and
    IMREAL, IMAGINARY, IMABS and IMARGUMENT take complex-number text. A table drives numeric literals; decide whether
    a text-literal point is allowed (a SPEC-0029 note) or these are `composite`, naming the eval suites. CONVERT is a
    factor table times a number: decide whether its points need a table per unit pair or it is `composite`.
  - **D-functions** take a database range and a criteria range, so they are likely `composite`
    (references.md and the gen-m9 D-function cases), as SUBTOTAL was in decision 0228.
  - **Bessel**: real kernels with evidenced overflow and underflow cuts (engineering.md "BESSELK underflow, BESSELI
    overflow ..." T-2421, T-2550, T-2613). They need tables.
- `exempt.tsv` is protected (decision 0199). Every line added needs the decision record (`make new-decision`). T-3786
  also appends to it; whichever lands second rebases.
- Group the functions that need a table into table tasks of at most about six functions each (size M), by family
  (annuities, cash flows, depreciation, bonds by kind, Bessel). Create each with `make new-task MILESTONE=M0
  LANE=harness`.
- Why opus: each exemption is a judgment against the evidence, and a text-literal point may need a SPEC-0029 note.

## Acceptance criteria
- [ ] A decision record classifies each function of the recounted list as `exact`, `composite` or "needs a table
      (T-XXXX)", with a cited reason for each exemption.
- [ ] `eval/accuracy/exempt.tsv` gains the exempt lines. `rustsheet-eval accuracy` reports `missing tables` lower by
      their number, with no stale exemption.
- [ ] Every function classified "needs a table" is named in the title or Goal of an open task filed by this PR, and
      `python3 scripts/tasks.py validate` passes.
- [ ] If a ruling changes SPEC-0029 (text-literal points), the spec note is in this PR and any reader or generator
      change is filed as a task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Building any table: the filed tasks do that.
- The statistical functions (T-3786) and ERF/ERFC (T-3784).
