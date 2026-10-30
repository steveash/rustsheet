---
id: T-3184
title: "Excel cells for the M9 financial argument kinds no suite covers (611 parameter/kind pairs in 55 functions, mostly empty text and errors)"
milestone: M9
lane: research
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Every value parameter of the 55 M9 financial functions below has an Excel-backed cell of each argument kind that
report.json `argument_coverage` expects. Phase 1's exit needs this: decision 0094 asks for coverage "across the
argument modes its catalog signature lists". Every new cell that differs from Excel gets an owner.

## Context
Gap triage 2026-10-30 (`make eval` on main c269641be; decision 0031 evidence gaps). The M2, M5, M6 and M7
argument-kind gaps have probe tasks (T-2861 to T-2878), but no task covers M9's. M9 lacks 1,392 parameter/kind
pairs. 611 are in the financial category, in 55 of its functions. By kind (the kinds of
crates/rustsheet-eval/src/argument_coverage.rs `classify_value`), counting parameters: empty_text 257, error 218,
numeric_text 78, blank 28, boolean 22, text 5, zero 2, negative 1.
Functions: ACCRINT, ACCRINTM, AMORDEGRC, AMORLINC, COUPDAYBS, COUPDAYS, COUPDAYSNC, COUPNCD, COUPNUM, COUPPCD,
CUMIPMT, CUMPRINC, DB, DDB, DISC, DOLLARDE, DOLLARFR, DURATION, EFFECT, FV, FVSCHEDULE, INTRATE, IPMT, IRR, ISPMT,
MDURATION, MIRR, NOMINAL, NPER, NPV, ODDFPRICE, ODDFYIELD, ODDLPRICE, ODDLYIELD, PDURATION, PMT, PPMT, PRICE,
PRICEDISC, PRICEMAT, PV, RATE, RECEIVED, RRI, SLN, SYD, TBILLEQ, TBILLPRICE, TBILLYIELD, VDB, XIRR, XNPV, YIELD,
YIELDDISC, YIELDMAT. Examples: ACCRINT.basis lacks blank, empty text, error and text; COUPDAYS.frequency lacks blank,
empty text, error and text; MIRR.values lacks empty text, error, numeric text and text; XIRR.guess lacks error and
numeric text. The full list is report.json `argument_coverage` filtered to catalog `category: financial`,
`milestone: M9`, `covered < expected`.

Why the generator never reaches them, which is the design question here: T-2866 added `""` and error draws
(`EMPTY_TEXT_RATE`, `ERROR_RATE`, 3% each) in `Gen.scalar_text` (tools/src/rustsheet_tools/gen/generator.py). Most
financial parameters have an edge `pool:` (POOL_RATE 0.35) and a soft `int`/`enum` domain, though. `domain_value` takes
70% of draws and the pool most of the rest, so a "" lands in well under 1% of cells. Kinds such as blank and numeric
text never land at all. Choose, and record in docs/architecture/synthetic-generator.md, one of:
(a) a coverage-directed `--focus` mode that draws the missing kinds per parameter;
(b) hand-written excel-probe cases in the T-2861 style (one literal and one cell form per kind, errors wrapped in
    `ERROR.TYPE`), probes/m2-argument-kind-gaps*;
(c) a mix.
Whatever you choose, keep the committed gen-m9 cases unchanged. New batches go in new case files and are oracle-filled
with excel-graph.

Known Excel behavior to cite where it applies: docs/excel-semantics/financial.md (argument check order of the bond and
odd-period functions, T-2482 and T-2483; date coercion). The 1904 variants are T-2843.

## Acceptance criteria
- [ ] After `make eval`, report.json `argument_coverage` shows `covered == expected` for every value parameter of the
      55 functions. Where Excel refuses a workbook holding a kind (a hard domain), the kind is exempted in the catalog
      with its reason instead.
- [ ] Every new cell where ours differs from Excel is fixed in this PR, if it is a one-line coercion fix in the
      function's own module, or listed by case and cell in a functions-financial task filed with `make new-task`
      (`campaign: semantics` and a `## Visible-parity stage` section for wrong types and errors). The new known failures
      are recorded with `make eval-baseline`, and `make unowned-off ALL=1` lists none of them.
- [ ] Each new Excel rule is in docs/excel-semantics/financial.md (or coercion.md), citing the cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The statistical (T-3185), engineering and database argument kinds.
- Fixing the numeric residue of new cells (file it as tasks).
