---
id: T-0040
title: "Catalog every Microsoft 365 Excel function with parity scope and oracle availability"
milestone: M0
lane: harness
status: done
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
- [x] Every Excel function name is in the catalog, sourced from the en-US function list and
  checked against the oracle (each name evaluated once in Excel for the web; `#NAME?` means not
  available there). A one-line test fails if a name is listed twice.
- [x] Each entry has `milestone` (from docs/milestones.md scopes), `scope: in | out`, and for
  `out` a short reason. Functions with no oracle evidence possible carry `oracle: none`.
- [x] Signatures (params, modes) are filled in for entries the generator already uses, and are
  allowed to stay minimal for the rest. Follow-up tasks are filed for signatures that the
  generator needs per milestone.
- [x] (moved) `make eval` gap report and T-0020 (evidence coverage) read scope from the catalog, so
  out-of-scope functions are not counted against parity. Split out as T-0051 (part (b); the eval
  crate has no YAML reader, so it needs a design choice).
- [x] `make check` green; reviewers approve (2026-10-07: security, quality, architecture and
  excel-fidelity reviewer agents; fidelity approved after one round of fixes).

## Out of scope
- Implementing any function.

## Outcome
Part (a) done; part (b), the eval reading `scope`, is T-0051.
- **List:** eval/catalog/excel-functions.tsv is the vetted source: 526 names with Microsoft's
  category, minimum argument count, syntax line, page URL and correction notes. Sources are the
  "Excel functions (alphabetical)" page fetched 2026-10-07 (521 names, the `X, XB` rows split, one
  typo `BETA.INVn`) plus five names only in the per-page function index: COPILOT, FIELDVALUE,
  IMPORTCSV, IMPORTTEXT and PY. HIDDEN in that index is "available only in another language
  version", so it is not listed. Minimum arity comes from each function's syntax line, with
  corrections noted: FLOOR.MATH, SHEET, SHEETS, SWITCH, LAMBDA, EUROCONVERT, and REDUCE, SCAN and
  PROB (oracle-verified, see below).
- **Catalog:** eval/catalog/functions.yaml lists all 526: the 35 M2 signatures unchanged, plus 491
  one-line entries (`name`, `category`, `milestone`, `scope`, `reason`, `oracle`), with the rules
  in its header. 505 are in scope and 21 out (decision 0027 list, plus IMPORTCSV and IMPORTTEXT
  as external data). By milestone: M2 37, M5 143, M6 44, M7 32, M9 270 (in+out).
  `Catalog.scoped` skips entries without `params`, so generated suites are unchanged.
  tools/tests/test_catalog.py fails on a duplicate name, on any drift from the TSV, and on bad
  fields. test_catalog_arity.py now checks every signature against the TSV's minimum, not a
  hand-typed dict of 20.
- **Oracle availability:** probes/catalog/availability-01..42 call each of the 503 in-scope
  names once with exactly its minimum argument count, 12 per workbook. INFO and SHEETS are not
  probed: the canary refuses SHEETS()/INFO() workbooks, so they are the only `oracle: none`
  entries. All 503 are recognised except JIS (`#NAME?`, also as `_xlfn.JIS`:
  function-name-forms-2). FILTERXML returns `#VALUE!` for valid input (function-name-forms A7).
  Under decision 0039 those web values are goldens (excel-semantics text.md, web.md); desktop
  confirmation is T-1081 (M14).
- **Found on the way:**
  - Four workbooks were rejected at first. Bisecting gave REDUCE/SCAN/PROB minimum 3
    (excel-semantics/syntax.md).
  - A case-builder bug: XlsxWriter's unescaped `.` rewrote NORMSDIST as `_xlfn.NORM.DIST` and
    NORMSINV as `_xlfn.NORM.INV`. Fixed in spec2xlsx.py and pinned by probes/catalog/normsdist-stored-form.
  - Stored-name quirks: a bare new function is `#NAME?`; eta arguments need `_xleta.`
    (excel-semantics/xlsx-storage.md).
  - The builder's wider reliance on XlsxWriter is T-0052.
- **Follow-ups:** T-0051 (scope in the gap report and evidence coverage), T-1082/T-1087/T-1088 (M5 by category), T-1090, T-1083, T-1080/T-1084/T-1085/T-1086 (M9 by category)
  (generator signatures for M5/M6/M7/M9), T-0052 (builder prefixes), T-1081 (JIS/FILTERXML desktop), T-1089 (xlsx import/export of
  prefixed names).
- **Eval delta:** probes +45 cases / +509 cells, all oracle-filled. 8 new-coverage known
  failures, 0 regressions: BAHTTEXT, DBCS, FILTERXML (2 cells), GETPIVOTDATA, GROUPBY, PHONETIC
  and PIVOTBY return `#NAME?` in the engine (not implemented).
