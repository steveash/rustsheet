# T-1853: does a legacy cell stay legacy after an edit? (excel-graph, 2026-10-18)

## Question

SPEC-0012 rule 6 assumes that a legacy cell stays legacy when a row is inserted above it, when it is
moved, and when it is copied or filled. The cell holds a plain `<f>` holding `=N(IF(1,{2;3}))+A5`, which is `#VALUE!`. The eval harness
cannot express an edit sequence (rustsheet-case/v1 is static), so this was a one-off session against the excel-graph oracle.

## Method

- **Workbook.** Built with spec2xlsx: Sheet1 A5 = 1 and A6 = 10. B2 is `=N(IF(1,{2;3}))+A5` and C2 is `=N(IF(1,A5:A6))+A5`,
  both listed under `legacy_formulas`, so both are written as plain `<f>`.
- **Session.** For each scenario, one fresh upload and one persistent workbook session: the edit, then
  `application/calculate` (FullRebuild), then a scratch-sheet touch, then `closeSession`. The saved file was then read with
  `rustsheet-eval snapshot`.
- **Edits.**
  - `range(address='1:1')/insert` with `{"shift": "Down"}`.
  - `range/copyFrom` (B2:C2 to D2:E2).
  - `range/moveTo` (B2:C2 to F2:G2).
  - `range/autoFill` (B2:C2 to B2:C3).
- The script lived in the session scratchpad. It uses only `GraphBackend._drive` and `_call` from tools/src/rustsheet_tools/oracle/backends.py.

## Results

| scenario | Graph call | saved cell | `<f>` kind | value |
|---|---|---|---|---|
| control (no edit) | none | B2 `N(IF(1,{2;3}))+A5` | normal (plain `<f>`) | `#VALUE!` |
| control | none | C2 `N(IF(1,A5:A6))+A5` | normal | 2 |
| row inserted above | `range/insert` OK | B3 `N(IF(1,{2;3}))+A6` | normal | `#VALUE!` |
| row inserted above | `range/insert` OK | C3 `N(IF(1,A6:A7))+A6` | normal | 2 |
| copy | `range/copyFrom` 400 BadRequest | (unchanged) | none | none |
| move | `range/moveTo` 400 BadRequest | (unchanged) | none | none |
| fill | `range/autoFill` 400 BadRequest | (unchanged) | none | none |

- **Row insert.** The formula's references are adjusted (A5 becomes A6), and the cell stays legacy: it is still saved as a plain
  `<f>` and is still `#VALUE!` with no spill. This matches SPEC-0012 rule 6.
- **Copy, move and fill.** Graph v1.0 refuses all three range operations with BadRequest. Office.js has them, but this Graph
  endpoint does not take them, so excel-graph cannot answer these. They are T-1075 item 12 (M14, decision 0039).
- **Side finding.** If the cells are not listed under `legacy_formulas`, spec2xlsx writes `<f t="array" ref=...>` with
  `cm="1"` (dynamic). That formula then spills and gives 3, so the control must mark legacy cells explicitly.
