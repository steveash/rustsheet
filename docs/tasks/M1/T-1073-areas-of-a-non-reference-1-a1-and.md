---
id: T-1073
title: "AREAS of a non-reference (1, 'A1') and bare =A1,B2 (SPEC-0004 rows 19-20)"
milestone: M1
lane: functions-lookup
status: done
priority: P3
size: S
tier: opus
depends_on: [T-1062, T-1163]
---

## Goal
Decide and match AREAS of a non-reference and the bare formula `=A1,B2`.

## Context
Oracle (T-1061): a workbook holding AREAS(1) or AREAS("A1") is refused by Excel for the web (createSession 501 unsupportedWorkbook, scratch runs, not kept as cases). `=A1,B2` stored in a file loads and evaluates to #VALUE! (probes/multiarea-bare); whether Excel refuses it when typed is unprobed (Excel for the web cannot type). The engine says AREAS(1)=1; decide the load behaviour (cf. the T-0138 refusal precedent in references.md) and match. Spec: docs/specs/SPEC-0004-multi-area-references.md; evidence: docs/excel-semantics/references.md "Multi-area references in functions".

## Acceptance criteria
- [x] AREAS of a number or text literal and the bare comma formula follow the recorded behaviour, documented in references.md
- [x] `make check` green; reviewers approve.

## Out of scope
- Other functions' union support (their own tasks under SPEC-0004).

## Handoff (resumed)
Resumed by an opus run on 2026-10-08 (tier raised to opus at handoff). The handoff proposed one shared reference-only argument check; T-1163 built it (decision 0045).

## Outcome
`AREAS(1)` and `AREAS("A1")` are handled by T-1163 (PR #413, decision 0045, merged before this PR): they are parse errors, as Excel refuses them, and a reference operand that evaluates to a value is `#VALUE!`. The bare `=A1,B2` turned out to be a precedence question. Outside every parenthesis and argument list, Excel reads the comma as the union operator, binding tighter than arithmetic and looser than the space. Its operands must be references (scratch probes, excel-graph). The parser now does the same (`parse_bare_union`, `OpUnionKind::bare`), and these formulas print as written (references.md "Bare union"). Eval delta: probes/multiarea-bare H1 is fixed, the new case probes/multiarea-bare-precedence passes all 20 cells, probes/multiarea-xname now passes, and probes/union-defined-name now loads, with 2 of its 5 cells passing (the other 3 belong to T-1065). Regressions: 0.

