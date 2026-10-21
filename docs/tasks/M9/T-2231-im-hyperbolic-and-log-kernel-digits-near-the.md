---
id: T-2231
title: "IM* hyperbolic and log kernel digits near the overflow edge (710-800)"
milestone: M9
lane: functions-engineering
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The 10 known-failure cells of probes/eng-im-overflow-edges match Excel bit-for-bit.

## Context
T-2077 baselined 10 cells (A6, A9, A14, A17, A47, A58, A95, A104, A107, A109): IMCOSH/IMSINH near 710.4 differ from Excel by several units in the 15th digit (Excel 1.17873946358278E+307, true ...283, ours ...281), which suggests a scaled exp in Excel; IMLOG2/IMLOG10/IMCOT/IMSECH differ by 1 in the last digit. Also probe a tiny nonzero imaginary part with a large real part, a blank inside an IMPRODUCT range, and |y| between 709.78 and 710.4 for IMSIN/IMCOS. Code: functions/engineering/complex.rs.

## Acceptance criteria
- [ ] the 10 cells pass and leave eval/baseline; the unprobed edges above are evidenced
- [ ] `make check` green; reviewers approve.

## Out of scope
-
