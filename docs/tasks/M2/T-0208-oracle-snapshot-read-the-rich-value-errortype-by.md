---
id: T-0208
title: "Oracle snapshot: read the rich-value errorType by key name (#SPILL! recorded as #VALUE!, 63 gen-m2 cells)"
milestone: M2
lane: harness
status: done
priority: P0
size: S
tier: opus
depends_on: []
---

## Goal
Oracle sidecars hold the error Excel actually produced. Today every `#SPILL!` in the gen-m2 sidecars
is stored as `#VALUE!`, so the expected values for 74 gen-m2 cells are wrong: 63 of them fail
against an engine that already gives `#SPILL!`, and 11 pass only because the engine wrongly gives
`#VALUE!`.

## Context
`crates/rustsheet-eval/src/xlsx_snapshot.rs::read_rich_errors` takes the **first** `<v>` of each
`<rv>` in `xl/richData/rdrichvalue.xml` as the `errorType`. That is only true when the `_error`
structure has `errorType` as its first key. In the Excel-for-the-web files the oracle returns, a
spill error uses the structure
`<s t="_error"><k n="colOffset"/><k n="errorType"/><k n="rwOffset"/><k n="subType"/></s>`
(keys in alphabetical order). The first value is then `colOffset` (0), the decoder finds no error
for it, and the cell keeps its cached `#VALUE!`. Example: the cached oracle workbook for
`gen-m2/78bc88199fc96650` (`eval/.cache/oracle/excel-graph/25b07ead…f28.xlsx`), cell Calc!A61
`=INDEX(Data!E7:E12,0)`, is `t="e" vm="1"` with `<v>#VALUE!</v>`, and its rv is
`<rv s="0"><v>0</v><v>8</v><v>5</v><v>1</v></rv>`, so errorType is 8 (`#SPILL!`). The engine
returns `#SPILL!` here (the spill range A61:A66 is blocked by A62). In the local excel-graph cache,
64 `_error` rich values use this colOffset-first structure, all with errorType 8. One uses an
errorType-first structure (errorType 10).

docs/excel-semantics/arrays.md ("Cached `#CALC!` / `#SPILL!` are stored as `#VALUE!` plus a `vm`
attribute") says "first field is the errorType". That is wrong for this structure and must be corrected
in the same PR: resolve the field through `xl/richData/rdrichvaluestructure.xml` by key name `errorType`.

T-0921 (one vm decoder shared with the importer, resolving through futureMetadata) is related but
separate; do not widen this task into it. Coordinate if T-0921 is in progress.

Gen-m2 cells whose sidecar says `#VALUE!` but whose oracle cell is a vm-coded `#SPILL!` (they fail
today: expected `#VALUE!`, actual `#SPILL!`):
- `gen-m2/04cfd82f54dbc2b2.case.json`: A161
- `gen-m2/1b6ceed7489e0800.case.json`: A134 A194 A198
- `gen-m2/207a5753da48c7e0.case.json`: A156 A249 A328
- `gen-m2/2cfe279fe77a000d.case.json`: A64 A301
- `gen-m2/61cacc22c4244179.case.json`: A364
- `gen-m2/7469c3e920e1ade4.case.json`: A57 A86 A389
- `gen-m2/78bc88199fc96650.case.json`: A61
- `gen-m2/893302d6c5b99e2e.case.json`: A240
- `gen-m2/8eb18149d4245e1a.case.json`: A113 A174
- `gen-m2/96a0952412cefa6c.case.json`: A34 A45 A74 A301
- `gen-m2/a339ea84ef64382e.case.json`: A22 A89 A271 A369
- `gen-m2/a6d4377c7b1b92cd.case.json`: A104 A149 A160 A201 A316
- `gen-m2/b4392a6514a36ade.case.json`: A162 A188 A209 A250 A260
- `gen-m2/caa981b0753f455c.case.json`: A51 A144 A314
- `gen-m2/cbdabe32a32a4012.case.json`: A46
- `gen-m2/ce315bb699e1be6f.case.json`: A134
- `gen-m2/d04d501154afc771.case.json`: A11 A41 A155 A375
- `gen-m2/d0c29dd5c530e28c.case.json`: A296 A328
- `gen-m2/d3e2df0bef2f173a.case.json`: A332
- `gen-m2/dd99be1f2454159f.case.json`: A56 A269
- `gen-m2/e8493fafa2a5378e.case.json`: A237
- `gen-m2/ef5c55a81783b7d5.case.json`: A66 A130
- `gen-m2/f0d1bacec5de8146.case.json`: A256 A303 A323
- `gen-m2/f7e6d351c8538ef5.case.json`: A58 A148 A167 A282
- `gen-m2/f9d4df4efdfbcbd5.case.json`: A51 A57 A206 A392

These 11 pass today only because the expected value is wrong. They will become known failures once the
sidecar is corrected. That is an expectation change, not an engine regression: say so in the PR and
follow docs/process/quality-gates.md for re-baselining.
- `gen-m2/207a5753da48c7e0.case.json`: A132 A297
- `gen-m2/2cfe279fe77a000d.case.json`: A126
- `gen-m2/75840b9c71a50596.case.json`: A29
- `gen-m2/942bdade381b7537.case.json`: A381
- `gen-m2/b4392a6514a36ade.case.json`: A348
- `gen-m2/d04d501154afc771.case.json`: A246 A362
- `gen-m2/d3e2df0bef2f173a.case.json`: A20
- `gen-m2/f7e6d351c8538ef5.case.json`: A106 A296

Also re-derive any other excel-graph sidecar with vm-coded cells (probes, poi, ironcalc-calc
recalcs) and list what changed in the PR.

## Acceptance criteria
- [x] `read_rich_errors` looks up `errorType` by key name through `rdrichvaluestructure.xml`. A unit
      test with inline XML covers both key orders (`errorType` first, and
      `colOffset, errorType, rwOffset, subType`).
- [x] gen-m2 sidecars are re-derived. (Deviation: the spec builder's output had changed since the
      first fill, so 27 of the 30 workbooks were recalculated by the oracle again; every value
      except the 74 decoded cells came back identical.) 58 of the 63 cells above pass; the other
      5 (207a5753da48c7e0 A328, b4392a6514a36ade A162, ef5c55a81783b7d5 A130,
      f7e6d351c8538ef5 A148 A282) now expect the correct `#SPILL!` but fail on lookup ordering
      (`#REF!`; T-0211/T-0213). The 11 cells above are known failures. The gen-m2 baseline is new
      in this PR, so none of this is a regression.
- [x] docs/excel-semantics/arrays.md entry corrected (key-named field, evidence: the cell above).
- [x] `make check` green; reviewers approve.

## Out of scope
- Sharing the decoder with the importer and the futureMetadata hop (T-0921).
- Any engine change to INDEX/spill behavior.

## Outcome
Done inside T-0203's PR (the gen-m2 goldens could not be committed correctly without it). `read_rich_errors` now finds `errorType` by key name through `rdrichvaluestructure.xml` (`rich_value_errors`, unit test); gen-m2 sidecars refilled (27 workbooks recalculated again because the spec builder output changed; values identical apart from these cells): 74 cells `#VALUE!` → `#SPILL!` (58 now pass, 5 fail for lookup reasons, 11 become known failures). The gen-m2 baseline is new in the same PR, so there are no regressions. Other suites' sidecars: values unchanged. arrays.md corrected.
