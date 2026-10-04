---
id: T-0124
title: "Unresolvable external-workbook reference without cached data returns #VALUE! (1 known failure)"
milestone: M1
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A formula that references another workbook with no usable cached values gives Excel's error
(`#VALUE!` per the excel-graph oracle) instead of `#REF!`.

## Context
Evidence: `poi/link-external-workbook-b.xlsx` (POI-written, excel-graph sidecar from T-0019)
Sheet0!A1 `=SUM('[link-external-workbook-a.xlsx]Sheet0'!A1:B1)`: expected `#VALUE!`, actual `#REF!`
(file cache `<v>30.0</v>`, ignored for scoring). The package has `xl/externalLinks/externalLink0.xml`
with an empty `<externalBook r:id="rId1"/>`: no sheetNames, no sheetDataSet. The formula uses the
literal file name in brackets, not Excel's `[1]` index form.
Directive 5 (no I/O) means we never open the other file. Decide, from evidence, which error
an unresolved external reference gives: (a) bracketed file name with no externalBook cache,
(b) `[n]` index with a cache (should use the cached values), (c) no externalLink part at all. Use
`excel-probe` for (b) and (c) if no case covers them, and record the rules in `docs/excel-semantics/`.

## Acceptance criteria
- [ ] poi/link-external-workbook-b.xlsx Sheet0!A1 disappears from eval/baseline/poi.txt with no new F lines.
- [ ] Rule (with the cases that are not evidenced marked UNVERIFIED) in `docs/excel-semantics/`.
- [ ] `make check` green; reviewers approve.

## Out of scope
Reading external workbooks (forbidden by directive 5). Import panics (T-0118, done).
