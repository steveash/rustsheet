---
id: T-0124
title: "Unresolvable external-workbook reference without cached data returns #VALUE! (1 known failure)"
milestone: M1
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-0030, T-1010]
spec: docs/specs/SPEC-0003-newer-error-kinds.md
---

## Goal
A formula that references another workbook with no usable cached values gives Excel's error
(a vm-coded rich-value error per the excel-graph oracle, errorType 10 / subType 2, cached as `#VALUE!`; T-0030 decoded it as `#BLOCKED!`) instead of `#REF!`. The workbook was written by Apache POI,
so this is web-only evidence. Implement on it: excel-graph is the oracle of record for Phases 1–3
(decision 0039), and the desktop confirmation is item 1 of T-1075 (M14), not a gate.

## Context
Evidence: `poi/link-external-workbook-b.xlsx` (POI-written, excel-graph sidecar from T-0019)
Sheet0!A1 `=SUM('[link-external-workbook-a.xlsx]Sheet0'!A1:B1)`: expected `#BLOCKED!` (T-0030), actual `#REF!`
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
