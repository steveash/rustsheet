---
id: T-2477
title: "Probe REGEX lookaround edges: a quantified assertion condition, (?x) space in a condition, a conditional inside a lookbehind"
milestone: M7
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence settles three REGEX* pattern edges that T-2354 left unprobed, and docs/excel-semantics/text.md records each answer.

## Context
Follow-up from the T-2354 fidelity review. Ours, all `#VALUE!` (functions/text/pcre/parse.rs `leading_assertion`,
ast.rs `fixed_length`), unprobed:
1. A quantifier after an assertion condition: `REGEXEXTRACT("b","(?(?=a)?b|c)")`.
2. Whitespace before an assertion condition in `(?x)` mode: `REGEXEXTRACT("ab","(?x)(?( ?=a)ab|c)")`.
3. A conditional inside a lookbehind whose branches have the same fixed length: `REGEXEXTRACT("ab","(a)?(?<=(?(1)a|b))b")`
   (PCRE2 may accept it).
Evidence so far: probes/regex-lookaround-keepout-conditionals (T-2354).

## Acceptance criteria
- [ ] A probe case (excel-probe skill, excel-graph) covers the three items, with ERROR.TYPE of each error; its cells are filled.
- [ ] text.md states each answer, and any mismatch with ours gets a `campaign: visible` task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the matcher: a mismatch gets its own task.
