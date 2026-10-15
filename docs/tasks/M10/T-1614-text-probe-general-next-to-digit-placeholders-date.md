---
id: T-1614
title: "TEXT: probe General next to digit placeholders, date tokens, % and a second General"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Know (and match) what Excel does with `General` next to digit placeholders, date tokens, `%`, or a second `General` in one section.

## Context
T-1262 made `General` among literals work (`TEXT(5,"General x")` = "5 x", `TEXT(5,"x General")` = "x 5";
probes/text-format-bare-literals A75, A76). Any other company is unprobed and the engine says `#VALUE!`
(`Parser::parse_part`, formatter/parser.rs, the `general_count` check). Semantics: docs/excel-semantics/number-formats.md
"Bare letters and digits in a number format". Probe (excel-probe skill): `TEXT(5,"0 General")`, `"General 0"`,
`"General%"`, `"General,"`, `"d General"`, `"General General"`, `"General."`, `TEXT(-5,"General x")`,
`TEXT(-5,"x General;y")`, `TEXT(1234567890123,"x General")`, and a lowercase `"general x"`.

## Acceptance criteria
- [ ] A probe case covers the formats above (excel-graph) and passes; number-formats.md records the rule (VERIFIED).
- [ ] `make check` green; reviewers approve.

## Out of scope
- `General` in the text (4th) section.
