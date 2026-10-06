---
id: T-0186
title: "Probe how numbers near 1E+307 print with 14 significant digits"
milestone: M1
lane: research
status: done
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
- [x] A probe case and oracle sidecar cover values with exponents from 100 to 307 and -300 to -5.
- [x] numbers.md records the rule VERIFIED; an engine task is filed if the engine differs.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe number-text-exponent-digits (exponents 5..307, -5..-300, 51..98 sweep) filled by excel-graph. Rule: 14 digits when |exp| >= 99, else 15 (not the 20-char cap alone; E+99 already loses one). Recorded in numbers.md; engine task T-0199 filed. Eval delta: none (new probe cells pending engine fix).
