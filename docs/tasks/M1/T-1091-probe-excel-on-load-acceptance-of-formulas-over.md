---
id: T-1091
title: "Probe Excel on-load acceptance of formulas over 8,192 characters (chains past 4,096 operators)"
milestone: M1
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for whether Excel loads and evaluates `=1+1+...` chains longer than 8,192
characters, and what happens at the real on-load limit.

## Context
T-1060 found Excel for the web evaluating an 8,193-character formula loaded from a file, so the
published 8,192 limit is not enforced on load. RustSheet refuses chains past `MAX_VALUE_OPERATORS`
(4,096); a longer chain Excel accepts would be refused. See docs/excel-semantics/syntax.md
("Formula nesting depth cap"); code: crates/rustsheet-engine/src/expressions/parser/depth.rs.

## Acceptance criteria
- [ ] Probe case with `=1+1+...` chains of 8,196, 8,200, 16,384 and 32,767 characters, oracle-filled.
- [ ] syntax.md records the on-load limit (or that none was found) and, if Excel accepts more than
  4,096 operators, a follow-up task proposes the change (decision record needed: DEVIATION).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing `MAX_VALUE_OPERATORS` in this task.

## Outcome
Probed 8,194, 8,195, 8,196, 8,200, 16,384 and 32,767 character `=1+1+...` formulas, one workbook
each: Excel for the web refuses all of them on load (`FileCorruptTryRepair`). The writer stores
formulas without `=`, so Excel enforces exactly 8,192 stored characters on load. The cases were not
kept (Excel cannot load them, so no sidecar can exist); the finding is in
docs/excel-semantics/syntax.md. The operator cap never refuses a chain Excel accepts; RustSheet
accepts over-length formulas Excel refuses, filed as T-1093. Eval delta: none.
