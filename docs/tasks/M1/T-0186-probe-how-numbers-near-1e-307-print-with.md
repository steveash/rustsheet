---
id: T-0186
title: "Probe how numbers near 1E+307 print with 14 significant digits"
milestone: M1
lane: research
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for how a number near 1E+307 prints in text: 14 or 15 significant digits, and what decides it.

## Context
Found by T-0173: `=(1/(2^-1022))&""` is "4.4942328371558E+307" in Excel while the cell value is 4.49423283715579E+307 (probes/arith-overflow Calc!A16:B16; engine prints 15 digits, a known failure in eval/baseline/probes.txt). Leading hypothesis: the 20-character cap on E-notation text (numbers.md), so include 3-digit negative exponents (1.23456789012345E-100) and 2-digit positive ones (1.23456789012345E+99). Probe several large and small exponents with 15-digit mantissas via `&""`, CONCAT and TEXT; record the rule in numbers.md.

## Acceptance criteria
- [ ] A probe case and oracle sidecar cover values with exponents from 100 to 307 and -300 to -5.
- [ ] numbers.md records the rule VERIFIED; an engine task is filed if the engine differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
