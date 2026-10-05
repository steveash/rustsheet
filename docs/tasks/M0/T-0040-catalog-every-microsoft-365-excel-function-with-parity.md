---
id: T-0040
title: "Catalog every Microsoft 365 Excel function with parity scope and oracle availability"
milestone: M0
lane: harness
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-0039]
---

## Goal
`eval/catalog/functions.yaml` lists every function in current Microsoft 365 Excel (en-US),
not only the 49 M1/M2 ones, so Phase 1 parity (decision 0026) has a definite denominator.
Each function records whether it is in parity scope and whether the oracle can evaluate it.

## Context
Decision 0026: Phase 1 exits when every in-scope function has comprehensive oracle-backed
coverage and passes it. The catalog today stops at M2, so "every function" cannot be measured.
Out of scope are functions whose result depends on systems outside the workbook (see the
decision's list). Some in-scope functions may not run in Excel for the web, the oracle we use
(for example FILTERXML). Probe them rather than assuming: they need another evidence source,
or an explicit scope decision.

## Acceptance criteria
- [ ] Every Excel function name is in the catalog, sourced from the en-US function list and
  checked against the oracle (each name evaluated once in Excel for the web; `#NAME?` means not
  available there). A one-line test fails if a name is listed twice.
- [ ] Each entry has `milestone` (from docs/milestones.md scopes), `scope: in | out`, and for
  `out` a short reason. Functions with no oracle evidence possible carry `oracle: none`.
- [ ] Signatures (params, modes) are filled in for entries the generator already uses, and are
  allowed to stay minimal for the rest. Follow-up tasks are filed for signatures that the
  generator needs per milestone.
- [ ] `make eval` gap report and T-0020 (evidence coverage) read scope from the catalog, so
  out-of-scope functions are not counted against parity.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing any function.
