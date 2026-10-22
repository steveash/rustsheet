---
id: T-2343
title: "Probe REGEXEXTRACT with an invalid pattern element in a lifted pattern (gen-m7 A587 is empty in Excel)"
milestone: M7
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
We know what Excel gives for REGEXEXTRACT when an element of a lifted pattern is invalid in PCRE2 (`?b`), and gen-m7/7307132bb8820f17 Calc!A587 matches it.

## Context
Follow-up of T-1973. The cell is `REGEXEXTRACT(Data!C$9,OFFSET(Data!B2:E9,2,3))`, anchored at A581. At A587 the pattern
element is "?b" (Data!E10) and the text is 0.2. PCRE2 rejects "?b" ("quantifier does not follow a repeatable item"), and the
T-1973 parser makes it `#VALUE!`. Excel's grid has an empty cell there. Is the cell really empty or "", and if so, why? A
compile error in one lifted element is otherwise `#VALUE!` in that element only (text.md "TEXTSPLIT, TEXTBEFORE, TEXTAFTER and
REGEX* lift ...").
Probe on excel-graph (excel-probe skill, about 10 cells):
- `REGEXEXTRACT(0.2,"?b")`, `REGEXEXTRACT("b","?b")`, `REGEXTEST("b","?b")`, `REGEXTEST("ab","a{,2}b")`;
- `ISBLANK` and `LEN` of the gen-m7 cell's equivalent;
- the same patterns lifted over a 2x1 array of patterns.
Record the result in text.md, and implement it in functions/text/pcre/parse.rs if it is a parser rule.

## Acceptance criteria
- [ ] A probes case (excel-graph) holds the grid, and text.md states the rule.
- [ ] The gen-m7 A587 line leaves eval/baseline/gen-m7.txt, or text.md explains why it cannot.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The backtracker (T-1900).
