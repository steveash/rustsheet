---
id: T-1298
title: "TEXT General rounds wide fixed values to 11 characters"
milestone: M2
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`TEXT(x,"General")` rounds fixed values from 1 on so digits plus the point take at most 11 characters, as Excel does.

## Context
Probe probes/text-general-ties-wide (T-1225) B4:B9: `123456.123456789` → "123456.1235", `22.712470704` → "22.7124707", `9999999999.5` → "10000000000", `99999999999.6` → "1E+11", `12345678901.5` → "12345678902". The engine keeps 9 decimals (`format_general`, formatter/format.rs); about 30 unit tests (test_convert, test_database) pin that form and must be updated. Semantics: docs/excel-semantics/number-formats.md.

## Acceptance criteria
- [ ] Cells B4:B9 of probes/text-general-ties-wide pass; baseline updated.
- [ ] Unit tests that pinned the 9-decimal form follow the 11-character rule.
- [ ] `make check` green; reviewers approve.

## Out of scope
- None.
