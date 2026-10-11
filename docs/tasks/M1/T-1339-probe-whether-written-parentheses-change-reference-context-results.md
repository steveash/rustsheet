---
id: T-1339
title: "Probe whether written parentheses change reference-context results (SPEC-0007 step 0)"
milestone: M1
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
spec: docs/specs/SPEC-0007-written-parentheses.md
---

## Goal
Oracle evidence for whether written parentheses change any reference-context result, so SPEC-0007's "groups are transparent to evaluation" rule is backed before T-1340 lands.

## Context
SPEC-0007 "Excel behavior to match" lists the unknowns. All 18 values of probes/at-operand-nested-groups match with groups dropped, but none of them is a reference-context case.

## Acceptance criteria
- [x] A probes case (`(INDEX(D1:D3,1))#` is unevidenced: excel-graph rejects the workbook, see Outcome) (excel-graph) covers `=ISREF((A1))`, `=ROWS(((A1:A3)))`, `=(INDEX(D1:D3,1))#`, `=ANCHORARRAY(@(IF(TRUE,D1)))` with D1 = SEQUENCE(3), `=SUM(((A1,B1)))`, `=LET(x,(A1:A3),ROWS(x))`, `=(0.1+0.2-0.3)` and `=((0.1+0.2-0.3))` (root snap), each with FORMULATEXT.
- [x] Findings recorded in docs/excel-semantics/references.md (or syntax.md). Any cell where a group is not transparent is flagged in SPEC-0007 as an amendment needed before T-1340.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
probes/written-parens-reference-context (35 cells, excel-graph): groups are transparent in reference context; the root snap is the only value a group changes. `(INDEX(D1:D3,1))#` cannot be built through excel-graph (unsupported workbook), recorded as unevidenced. SPEC-0007 needs no amendment; findings in references.md. Six new known failures in the baseline (FORMULATEXT drops groups: H10, H11, H14, which T-1340 fixes; plain `ANCHORARRAY(IF(TRUE,D1))` gives TypeMismatch: E20:E22). Follow-up: T-1363 for the ANCHORARRAY gap. Follow-up: T-1364 (reference-only checks and the `)#` shape).
