---
id: T-0040
title: "Catalog every Microsoft 365 Excel function with parity scope and oracle availability"
milestone: M0
lane: harness
status: blocked
priority: P1
size: M
tier: opus
depends_on: [T-0039]
---

## Goal
`eval/catalog/functions.yaml` lists every function in current Microsoft 365 Excel (en-US),
not only the 49 M1/M2 ones, so Phase 1 parity (decision 0027) has a definite denominator.
Each function records whether it is in parity scope and whether the oracle can evaluate it.

## Context
Decision 0027: Phase 1 exits when every in-scope function has comprehensive oracle-backed
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

## Blocked because
Tried 2026-10-05 (opus run). Two blockers, both outside what one session can fix:
1. **No en-US function list is reachable from the sandbox.** The egress proxy rejects
   support.microsoft.com and learn.microsoft.com (CONNECT 403, "organization policy"), so the
   Microsoft "Excel functions (alphabetical)" list cannot be fetched. What is reachable locally:
   openpyxl `FORMULAE` (355 names, Excel 2007 era) plus XlsxWriter's `_xlfn.` future-function list
   (154) — 508 names in union — but both miss every recent function (GROUPBY, PIVOTBY, PERCENTOF,
   REGEXTEST/REGEXEXTRACT/REGEXREPLACE, TRIMRANGE, IMAGE, COPILOT, TRANSLATE, DETECTLANGUAGE, PY,
   STOCKHISTORY, FIELDVALUE, ...). A list typed from memory would not meet "sourced from the
   en-US function list".
2. **Availability cannot be probed with zero-argument calls.** A pilot probe case with
   `=ERROR.TYPE(ABS())`, `SUM()`, `NOSUCHFN()`, `XLOOKUP()`, ... made Excel for the web reject the
   whole workbook (Graph `createSession` → 501 `FileCorruptTryRepair` / `unsupportedWorkbook`). An
   arity-invalid call anywhere poisons the file, so each probed name needs a call with a valid
   argument count, i.e. per-function minimum arity — which itself needs the documentation.

Unblock options (owner or steward): allow `support.microsoft.com` / `learn.microsoft.com` in the
environment's network policy (docs: https://code.claude.com/docs/en/claude-code-on-the-web), or
commit a vetted en-US name + minimum-arity list into the repo. Then split this task: (a) catalog
names, `scope`, `oracle` availability (probe with valid-arity calls, in batches so one bad call
cannot hide the rest); (b) `make eval` gap report and T-0020 read `scope` from the catalog (the
eval crate has no YAML reader today, so (b) needs a small design choice: serde_yaml vs a
generated JSON view).
