---
id: T-3802
title: "Probe PROPER word starts after lone surrogates and supplementary letters, and word-start lowering of non-letter capitals"
milestone: M5
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-3657]
---

## Goal
PROPER's handling of three unprobed inputs is evidenced by excel-graph cells and the engine matches them: a lone surrogate before a letter, a supplementary-plane letter at a word start and before a letter, and a non-letter capital (Georgian Mtavruli, U+0524) at a word start.

## Context
T-3657 found that PROPER reads letters from Excel's own BMP table (text.md "PROPER word starts: Excel's own letter
table (T-3657)", probes/t3657-proper-letter-sweep) and lowers every non-letter. Three points are inferred, not probed:
1. A supplementary letter is not a letter (`PROPER("a𝐚bc")` is `A𝐚Bc`, probes/t3657-proper-word-starts A60); text.md
   says "probably each surrogate is read alone". Probe a supplementary letter at a word start (`PROPER(UNICHAR(119834)&"x")`)
   and a Deseret capital and small letter (U+10400, U+10428) at a word start and mid-word.
2. A lone surrogate breaks a word (text.md lone-surrogates entry, "unprobed"): probe `PROPER("a"&LEFT(UNICHAR(128512))&"bc")`
   and the low half likewise (the excel-probe skill; text-surrogate-positions shows how lone units are built).
3. Non-letter capitals are lowered at a word start through `simple_case` (only the circled letters are probed,
   t3657-proper-word-starts B21:D21). Probe Mtavruli U+1C90, U+0524 and U+A78D at a word start, mid-word and after a digit.
Code: `proper_case` (functions/text/string_format.rs), `is_proper_letter` (functions/text/proper_letters.rs); case table
from T-3583.

## Acceptance criteria
- [ ] A probe case (excel-graph) covers the three points above; text.md records each result and drops "probably" and "unprobed".
- [ ] The engine matches every probed cell, or each failing cell is owned by a named open task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The BMP letter table (T-3657) and UPPER/LOWER's case table (T-3583).
