---
id: T-1372
title: "Financial functions lift scalar arguments over arrays, and the core ones over ranges (probes/fin-argument-modes)"
milestone: M9
lane: functions-financial
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-1770]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
Every scalar argument of the M9 financial functions lifts elementwise over an array literal. The core worksheet
functions also lift over a multi-cell range, while the Analysis ToolPak ones are `#VALUE!` there, as in Excel.

## Context
SPEC-0011 rollout step 2 (docs/specs/SPEC-0011-scalar-argument-lifting.md). The "lift literals only" variant this task asked for is the table's `LiftArrays` kind. The core worksheet functions are `Lift`, and the Analysis ToolPak ones are `LiftArrays` in every scalar argument. EFFECT's later argument is refused too (probes/scalar-lift-shapes H11, H12; -b H, I). NPV's values, IRR's and MIRR's values, XNPV's and XIRR's values and dates and FVSCHEDULE's schedule are `WholeByCaller` (their rules are T-1373's). The general rules (shape, `#N/A` padding, errors per element, legacy `@`) are probes/scalar-lift-shapes H13, H14, H17-H19, H23-H33, H41-H48, J1, J3-J5, K1, K3-K5 and -c G, R. Ulp misses there are T-0901's and T-1374's. Before entering a later scalar argument as `LiftArrays`, probe it over a multi-cell range on the range's own rows (a probe case in this task, excel-probe skill), unless SPEC-0011 or T-1777 already cites a cell for it. Only the first scalar argument of each function and the later ones listed in docs/excel-semantics/arrays.md "Scalar arguments lift ..." are evidenced. The table describes a function whole, so a function with an unprobed argument is entered only once that argument is probed. Until then it keeps today's code.

probes/fin-argument-modes (excel-graph, 2026-10-11, T-1084; legend in docs/excel-semantics/financial.md, "Financial
function argument modes"). All 256 `ROWS(F(..{v;v}..))` cells are 2 in Excel. Of the 55 `ROWS(F(G1:G2,..))` cells, 18 are 2
(DB, DDB, FV, IPMT, IRR guess, ISPMT, MIRR, NPER, NPV rate, PDURATION, PMT, PPMT, PV, RATE, RRI, SLN, SYD, VDB) and 37
are `#VALUE!` (the ATP functions); the engine already matches those 37. The engine fails 274 lift cells: `#N/IMPL!` for
an array (256) and `#VALUE!` for a range (18). Reuse the lifting the math, date and lookup functions already use (search
before adding a helper; T-1290 does the same for the statistical functions). The ATP functions lift arrays but not
ranges, so the helper needs a "lift literals only" variant, or an existing way to say that. Split by family if larger than M.

More cells (T-1289's gen-m9 suite, excel-graph, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/675763979c1defb6 Calc!A13 `=PRICEDISC(42,(-7/Data!E2),(FILTER(Data!B1:E12,..)+RADIANS(42)),(Data!B$1^7)%)`: Excel `#SPILL!`, engine `#N/IMPL!`.
- gen-m9/b58b6d7cd39d195f Calc!A21 `=PPMT(MUNIT(3),1,359,117633,154)`: Excel `#SPILL!`, engine `#N/IMPL!`.

## Acceptance criteria
- [ ] The lift cells of probes/fin-argument-modes pass (their baseline lines removed), and the 37 range `#VALUE!` cells
  still pass. The PMT and EFFECT shape and error cells of probes/scalar-lift-shapes and -b listed under Context pass. Value cells that miss by ulps stay T-0901's and T-1374's.
- [ ] gen-m9/675763979c1defb6 A13 and b58b6d7cd39d195f A21 pass, with no new failing gen-m9 cell and no new known failures in eval/baseline/.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Base-value ulp misses (T-1374, T-0901, T-0916).
