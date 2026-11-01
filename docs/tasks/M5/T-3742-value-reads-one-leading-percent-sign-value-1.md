---
id: T-3742
title: "VALUE reads one leading percent sign: VALUE(\"%1.5\") is 0.015"
milestone: M5
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-3647]
---

## Goal
VALUE gives Excel's value for text with a leading percent sign, as NUMBERVALUE already does.

## Context
- Evidence: `VALUE("%1.5")` = 0.015 (ours `#VALUE!`): probes/t3647-numbervalue-percent-and-controls Sheet1!A13,
  excel-graph, 2026-11-01. It is the one known failure of that case.
- T-3647 taught NUMBERVALUE one leading `%`, also after a sign or `(`, refused with currency or a second leading `%`
  (text.md "NUMBERVALUE removes group separators only before the decimal separator"; `strip_leading_percent` in
  functions/text/string_format.rs). Only `VALUE("%1.5")` is evidenced for VALUE: probe `"-%1.5"`, `"%-1.5"`,
  `"(%1.5)"`, `"%%1.5"`, `"%$1.5"`, `"%1.5%"` and `"%"` under VALUE (and the shared coercion: `"%1.5"+0`) with the
  `excel-probe` skill before generalizing, and reuse `strip_leading_percent` where the rules agree.
- The shared reader is `parse_number_text` (coercion.md); a change there reaches every text-to-number coercion, so
  probe the arithmetic coercion too.

## Acceptance criteria
- [ ] A probe case (excel-graph) covers the shapes above under VALUE and under `+0`.
- [ ] The engine matches every probed cell, including probes/t3647-numbervalue-percent-and-controls A13; the rule is
      in docs/excel-semantics (text.md or coercion.md).
- [ ] `make check` green; eval known failures only shrink (baseline updated); reviewers approve.

## Out of scope
- NUMBERVALUE (T-3647).
