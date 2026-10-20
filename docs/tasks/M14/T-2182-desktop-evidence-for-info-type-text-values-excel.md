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
- From T-2115. On Excel for the web every INFO type ("recalc", "system", "release", "directory", "osversion",
  "origin", "memavail", "memused", "totmem", "", "x", in any case) is `#VALUE!` (probes/info-sheets-info-types
  Alpha!D25:D49, excel-graph, 2026-10-20). Those are the goldens for Phases 1-3 (T-2184 matches them); this task
  confirms or contradicts them on desktop. docs/excel-semantics/information.md "INFO is #VALUE! in Excel for the web".
- Decision 0039: desktop-only questions are Phase 4 (M14) and never block. Blocked until `rustsheet-oracle status`
  shows `excel-desktop OK` (T-1074).
- Probe on desktop: each type_text in lower, upper and mixed case, `INFO("")`, `INFO("x")`, `INFO(1)`, `INFO(TRUE)`,
  `INFO(#N/A)`, `INFO({"recalc","system"})`, `INFO("numfile")` (the canary changes the sheet count), with host-dependent
  values ("directory", "osversion", "release", "origin", memory) scored by shape only.

## Acceptance criteria
- [ ] A desktop-filled probe case answers the list above; information.md records the rules.
- [ ] Every engine mismatch is fixed or filed.

## Out of scope
- Excel for the web (T-2184).
