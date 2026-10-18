---
id: T-1856
title: "Refuse LET names and defined names Excel refuses at load: Turkish dotted İ and circled letters such as Ⓐ"
milestone: M8
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The engine refuses (as Excel does) a LET name or defined name that Excel refuses at load, such as the Turkish dotted `İ` and the circled letter `Ⓐ`, for formulas entered through the API as well as xlsx.

## Context
T-1431 (docs/excel-semantics/references.md, T-1387 paragraph) found that Excel for the web refuses at load (createSession 501 FileCorruptTryRepair) a single-cell workbook whose LET names `İ` or `Ⓐ` in any spelling of the use; the engine accepts them (`is_valid_a1_identifier` in crates/rustsheet-engine/src/expressions/utils/mod.rs uses `char::is_alphabetic`, which accepts `Ⓐ` through Other_Alphabetic; `İ` lowercases to two code points). T-1387 set the precedent for refused forms (duplicate LET names give `#ERROR!`). Do not guess the rule: probe `rustsheet-oracle recalc -b excel-graph` single-cell workbooks for which characters Excel accepts as the first and later characters of a name (LET, LAMBDA, defined name) and fit one rule; check formula entry vs. xlsx load.

## Acceptance criteria
- [ ] Refusal probes recorded in references.md; the engine returns `#ERROR!` (or the repo's refused-name error) for the refused names; unit tests; no change to the accepted forms in probes/let-name-non-ascii*.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
