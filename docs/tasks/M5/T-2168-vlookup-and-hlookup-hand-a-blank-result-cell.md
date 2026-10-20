---
id: T-2168
title: "VLOOKUP and HLOOKUP hand a blank result cell to their caller as 0"
milestone: M5
lane: functions-lookup
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
VLOOKUP and HLOOKUP hand a blank result cell to the enclosing function as the number 0, as Excel does.

## Context
Gap triage of gen-m5 (T-2114, excel-graph oracle, 2026-10-20). Ours keeps the blank (`ISBLANK(VLOOKUP(..))` is TRUE), so `ASC` of it is "" and AVERAGE skips it. Excel's ASC gives "0" and
AVERAGE counts a 0. Rule: docs/excel-semantics/lookup.md "VLOOKUP and HLOOKUP hand a blank result cell to their caller as 0"
(added by this triage). Probe first (`excel-probe`): `ISBLANK`, `&""`, `ISNUMBER` and `ARABIC` of a VLOOKUP/HLOOKUP landing on a
blank, the same for LOOKUP and INDEX/MATCH, so the change is scoped right (INDEX returns a reference and probably keeps the blank).
ARABIC over such a result (6a511bea A106, b0d597ec A365) is in T-2153.

Failing cells (`make eval-show CASE=gen-m5/<case>.case.json`), 2 in eval/baseline/gen-m5.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/49bfbf67a1409563 Calc!A103 | `=ASC(IFS(IF(Data!$B5,Data!B5),VLOOKUP(Data!E12,Data!D3:E7,2),ISNA(Data!E10),(1e-15/Data!A9),CELL("...` | "0" | "" |
| gen-m5/aac6230c2fb40e1e Calc!A334 | `=SEC(AVERAGE(HLOOKUP(2.5,Data!B8:D11,4),(Data!$A$12+Data!C1),Data!B7:B9))` | 1.2724471433871758 | 1.8508157176809255 |

The probe must separate VLOOKUP/HLOOKUP from IFS: the ASC cell (49bfbf67 A103) passes the blank through IFS, which may be what makes it 0; the HLOOKUP→AVERAGE cell (aac6230c A334) has no IFS.

## Acceptance criteria
- [x] These 2 known-failure lines disappear from eval/baseline/gen-m5.txt (`make eval-baseline`), with no new known failure in any suite.
- [x] lookup.md entry updated with the probe results.
- [x] `make check` green; reviewers approve.

## Out of scope
- XLOOKUP (already returns 0).

## Outcome
Probed first (probes/lookup-blank-result-to-caller, lookup-vector-blank-result-to-caller; excel-graph): a
VLOOKUP, HLOOKUP or LOOKUP that lands on a blank cell gives the same empty value IFERROR gives for one (ISBLANK
TRUE, `&""` "", but COUNT, COUNTA and AVERAGE count it as 0), and IFS and SWITCH turn that empty value into 0,
while IF, CHOOSE and LET keep it and INDEX/MATCH stays a reference. Implemented as
`CalcResult::blank_as_empty_value` on the scalar result of the three lookups and `empty_value_as_zero` on what IFS
and SWITCH pick. Eval: gen-m5 known failures 212 → 210 (both target cells), 2 new probe cases at 173/173, no
regressions. lookup.md entry rewritten with the probe results.
