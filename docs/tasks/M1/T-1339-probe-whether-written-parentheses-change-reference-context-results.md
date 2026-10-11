---
id: T-1339
title: "Probe whether written parentheses change reference-context results (SPEC-0007 step 0)"
milestone: M1
lane: research
status: in-progress
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
- [ ] A probes case (excel-graph) covers `=ISREF((A1))`, `=ROWS(((A1:A3)))`, `=(INDEX(D1:D3,1))#`, `=ANCHORARRAY(@(IF(TRUE,D1)))` with D1 = SEQUENCE(3), `=SUM(((A1,B1)))`, `=LET(x,(A1:A3),ROWS(x))`, `=(0.1+0.2-0.3)` and `=((0.1+0.2-0.3))` (root snap), each with FORMULATEXT.
- [ ] Findings recorded in docs/excel-semantics/references.md (or syntax.md). Any cell where a group is not transparent is flagged in SPEC-0007 as an amendment needed before T-1340.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
