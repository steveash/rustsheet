---
id: T-1073
title: "AREAS of a non-reference (1, 'A1') and bare =A1,B2 (SPEC-0004 rows 19-20)"
milestone: M1
lane: functions-lookup
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-1062]
---

## Goal
Decide and match AREAS of a non-reference and the bare formula `=A1,B2`.

## Context
Oracle (T-1061): a workbook holding AREAS(1) or AREAS("A1") is refused by Excel for the web (createSession 501 unsupportedWorkbook, scratch runs, not kept as cases). `=A1,B2` stored in a file loads and evaluates to #VALUE! (probes/multiarea-bare); whether Excel refuses it when typed is unprobed (Excel for the web cannot type). The engine says AREAS(1)=1; decide the load behaviour (cf. the T-0138 refusal precedent in references.md) and match. Spec: docs/specs/SPEC-0004-multi-area-references.md; evidence: docs/excel-semantics/references.md "Multi-area references in functions".

## Acceptance criteria
- [ ] AREAS of a number or text literal and the bare comma formula follow the recorded behaviour, documented in references.md
- [ ] `make check` green; reviewers approve.

## Out of scope
-
