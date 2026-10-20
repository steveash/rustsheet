---
id: T-2182
title: "Desktop evidence for INFO type_text values (Excel for the web gives #VALUE!)"
milestone: M14
lane: research
status: blocked
priority: P3
size: S
tier: sonnet
depends_on: [T-1074]
---

## Goal
Desktop Excel values for INFO's type_text arguments are known, so INFO can be implemented and evidenced.

## Context
- From T-2115. Excel for the web does not implement INFO: every type ("recalc", "system", "release", "directory",
  "osversion", "origin", "memavail", "memused", "totmem", "", "x", in any case) is `#VALUE!` on excel-graph
  (2026-10-20; the probe rows were dropped from probes/info-sheets-info-types). The oracle canary now refuses every
  INFO call as web-unsupported, like `CELL("width")` (tools/src/rustsheet_tools/oracle/canary.py). docs/excel-semantics/information.md
  "INFO is not implemented in Excel for the web".
- Decision 0039: desktop-only questions are Phase 4 (M14) and never block. Blocked until `rustsheet-oracle status`
  shows `excel-desktop OK` (T-1074).
- Probe on desktop: each type_text in lower, upper and mixed case, `INFO("")`, `INFO("x")`, `INFO(1)`, `INFO(TRUE)`,
  `INFO(#N/A)`, `INFO({"recalc","system"})`, `INFO("numfile")` (the canary changes the sheet count), with host-dependent
  values ("directory", "osversion", "release", "origin", memory) scored by shape only.

## Acceptance criteria
- [ ] A desktop-filled probe case answers the list above; information.md records the rules.
- [ ] Every engine mismatch is fixed or filed.

## Out of scope
- Excel for the web, which has no INFO.
