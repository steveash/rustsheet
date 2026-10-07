---
id: T-0052
title: "Prefix every future function in the case builder itself"
milestone: M0
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: [T-0040]
---

## Goal
Every function call in a case spec reaches Excel under its stored name (`_xlfn.`, `_xlfn._xlws.`),
whatever other functions share the formula.

## Context
tools/src/rustsheet_tools/spec2xlsx.py prefixes the engine exporter's future functions itself and
leaves the rest to XlsxWriter's `use_future_functions`. XlsxWriter (3.2.9) skips any formula that
already contains `_xlfn.`, so one engine-prefixed call disables it for the whole formula:
`=TRIMRANGE(A2)+DAYS(1,2)` is stored `_xlfn.TRIMRANGE(A2)+DAYS(1,2)`, and a bare future function
is `#NAME?` in Excel (docs/excel-semantics/xlsx-storage.md, "Newer functions need `_xlfn.`").
Names only XlsxWriter prefixes today: DAYS, FILTERXML, FORECAST.LINEAR, GAMMA.DIST, GAUSS, IMAGE,
IMSECH, WEBSERVICE (plus ANCHORARRAY, handled separately, and QUERYSTRING/SINGLE). XlsxWriter's
regexes also leave `.` unescaped, which turned NORMSDIST into NORM.DIST; T-0040 worked around those
two names (`_XLSXWRITER_MISREAD`). Fix: one prefix table in the builder (engine list plus the
missing names, ideally from eval/catalog), applied with the builder's own tokenizer, and
`use_future_functions` off. Changing bytes changes `input_sha256`, so check which committed
sidecars would go stale (probably none if no spec mixes the two lists).

## Acceptance criteria
- [x] A unit test: `=TRIMRANGE(A2)+DAYS(1,2)` and `=NORMSDIST(0)+GAUSS(0)` store every call
  with its prefix (and NORMSDIST as itself).
- [x] No committed spec case changes its built bytes, or its sidecar is refilled.
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine export (`Function::to_xlsx_string`).

## Outcome
The case builder owns every future-function prefix: one table (the engine exporter's list plus
`_BUILDER_ONLY_PREFIXES`, the names only XlsxWriter knew), applied with the builder's tokenizer, and
XlsxWriter's `use_future_functions` is off. `=TRIMRANGE(A2)+DAYS(1,2)` now stores `_xlfn.DAYS`;
NORMSDIST/NORMSINV are stored bare, as Excel stores them (the `_XLSXWRITER_MISREAD` workaround is
gone). A test checks the table still covers XlsxWriter's own list. Of 298 committed specs, only
probes/catalog/normsdist-stored-form and availability-27 changed bytes; both were refilled (normsdist-stored-form also gained A4 `_xlfn.NORMSDIST(0)`, keeping the prefixed-classic evidence reproducible)
(excel-graph 2026-10-07) with identical values. Eval delta: none (0 regressions). Filed T-0411
(the engine exporter writes five newer names bare).
