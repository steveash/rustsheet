---
id: T-1734
title: "Displayed text: two-condition sections pick the section as Excel does"
milestone: M10
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A format with conditions (`[<0]`, `[>=1000]`, `[=0]`, `[<=-1]`, `[>=1]`) picks the section and the sign display as Excel does.

## Context
Codes `[<0]"neg";[>0]"pos";"zero"` (1 → Excel "pos", ours "neg"; 73/80 fail), `[=0]"nil";General` (PI() →
"3.141592654", ours "nil"; -123456.789 → "-123456.789", ours drops the sign), `[<=-1]0;[<1]0.0000;0.0` (-1.5 →
"2", ours "1.5000"; 1234567890 → "1234567890.0"), `[<100]0.0;[>=1000]#,##0;0` (2958465.99999 → "2,958,466",
-0.5 → "-0.5", ours "2958466.0" and "1"), `[>=1]0.00;0.000` (-0.5 → "-0.500", 0.5 → "0.500"). T-0936 covers
`[<>]`, spaces in conditions and an unmatched single condition; this is the basic section choice, which
looks broken for two conditions and for a condition plus General.
Evidence: suite gen-m10 (excel-graph, 2026-10-16, T-1611): `make eval` report, "Displayed text" section, which lists each failing text cell (Excel vs ours). Text cells are scored apart from values and are not baselined (decision 0068); the TEXT() cells on sheet T that use the same codes are value known failures in eval/baseline/gen-m10.txt. Formatter: crates/rustsheet-engine/src/formatter/ (shared by `Workbook::formatted` and TEXT). docs/excel-semantics/number-formats.md.

## Acceptance criteria
- [ ] The gen-m10 text cells under these codes pass in the report's "Displayed text" section (or the remaining ones are filed); the matching TEXT() cells on sheet T leave eval/baseline/gen-m10.txt; no regressions.
- [ ] number-formats.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
