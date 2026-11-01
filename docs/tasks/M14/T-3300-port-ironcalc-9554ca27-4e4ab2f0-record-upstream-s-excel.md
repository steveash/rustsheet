---
id: T-3300
title: "Port ironcalc 9554ca27/4e4ab2f0: record upstream's Excel-authored ENCODEURL.xlsx as desktop evidence (not b55c61a5's ~ rule)"
milestone: M14
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Upstream IronCalc's desktop-authored `ENCODEURL.xlsx` is read as Excel evidence: its cached
values are recorded as desktop results for ENCODEURL, next to the web golden we follow
(decision 0039), and the workbook is scored or listed in the ironcalc-calc suite as that decision
allows.

## Context
Upstream added `xlsx/tests/calc_tests/TEXT/ENCODEURL.xlsx` in 9554ca27 and replaced it in
4e4ab2f0 (14,226 bytes at upstream 2128db12; `docProps/app.xml` says `Microsoft Excel` 16.0300,
so it is desktop-authored). 4e4ab2f0 also added `number_to_text` to upstream's
`number_format.rs` for the text of a number argument. Upstream b55c61a5 then made `~`
unreserved and says in a comment that this deliberately differs from Excel, which encodes `~`
as `%7E`: never port that rule (directive 1).
Our `functions/text/encodeurl.rs` returns `#VALUE!` for every argument, the excel-graph golden
(docs/excel-semantics/web.md "ENCODEURL is `#VALUE!` in Excel for the web"); desktop
confirmation is T-1081's (M14). Read the workbook with `rustsheet-eval snapshot`, record the
desktop values (including `~` and number arguments) in that excel-semantics entry and on T-1081,
and add the file to ironcalc-calc only with its cells pinned per decision 0036/0039, never as a
change to the web golden. Upstream's unit tests (`test_fn_encodeurl.rs`) are IronCalc-authored,
not evidence. Triaged by the 2026-10-30 18:47 UTC steward run.

## Acceptance criteria
- [ ] The desktop ENCODEURL values from the workbook are in docs/excel-semantics/web.md and cited on T-1081.
- [ ] The workbook is in the ironcalc-calc suite with its cells scored or pinned, or the task's Outcome says why not; no regressions.
- [ ] The commit carries `Ported-From: ironcalc@4e4ab2f0`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing ENCODEURL's results while the web golden stands (decision 0039).
- Upstream's `~` rule (b55c61a5), which upstream itself says is not Excel's.
