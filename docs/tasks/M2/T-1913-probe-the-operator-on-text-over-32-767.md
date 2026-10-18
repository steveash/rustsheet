---
id: T-1913
title: "Probe the & operator on text over 32,767 units"
milestone: M2
lane: functions-text
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Know what `&` does with an operand over 32,767 UTF-16 units and match it.

## Context
T-1904 probed UPPER, LOWER, PROPER, TRIM, CLEAN, REPLACE and SUBSTITUTE on a 65,534-unit REGEXREPLACE result, but excel-graph refuses the workbook that holds `REGEXREPLACE(...)&"c"`. Try a shape it accepts (one long text, e.g. a reference to a spilled/other cell) or note it as a desktop question (M14). Record the rule in text.md.

## Acceptance criteria
- [ ] probes/text-long-input-* case for `&` has oracle values; ours matches or the gap is baselined; text.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed `&` with a 65,534-unit operand (probes/text-long-input-concat-d, -g, -h, -k, -l, -m, -o, excel-graph): the result truncates to 32,767 units, no error, measured only with the long text as the right operand or inside a longer concatenation; excel-graph refuses the left-operand shapes, so that case is unmeasured. Ours already matches all 7 cases; no engine change. text.md updated. Left-operand question filed as T-1928 (M14, desktop).
