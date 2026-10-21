---
id: T-2231
title: "IM* hyperbolic and log kernel digits near the overflow edge (710-800)"
milestone: M9
lane: functions-engineering
status: todo
priority: P3
size: S
tier: opus
campaign: visible
depends_on: []
---

## Goal
The 10 known-failure cells of probes/eng-im-overflow-edges match Excel bit-for-bit.

## Context
T-2077 baselined 10 cells (A6, A9, A14, A17, A47, A58, A95, A104, A107, A109): IMCOSH/IMSINH near 710.4 differ from Excel by several units in the 15th digit (Excel 1.17873946358278E+307, true ...283, ours ...281), which suggests a scaled exp in Excel; IMLOG2/IMLOG10/IMCOT/IMSECH differ by 1 in the last digit. Also probe a tiny nonzero imaginary part with a large real part, a blank inside an IMPRODUCT range, and |y| between 709.78 and 710.4 for IMSIN/IMCOS. Code: functions/engineering/complex.rs.

Visibly-off cells (gap triage 2026-10-21, decision 0083): 10 cells, 0 numeric past half a unit in the 15th digit and 10 with a wrong type, error or text, blamed on IMLOG2 3, IMSINH 2, IMCOSH 2, IMCOT 1, IMSECH 1, IMLOG10 1. Every one is listed above.

## Acceptance criteria
- [ ] Visible parity: the 10 visibly-off cells listed under Context (2026-10-21) have Excel's value to 15 significant digits, or its type, error and text: the 10 lines in eval/baseline/probes.txt disappear (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] the 10 cells pass and leave eval/baseline; the unprobed edges above are evidenced
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (IM* text whose 15th printed digit differs near the overflow edge), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it. Bit-level criteria still unmet when that target is reached move to a follow-up task filed without the key and named in the Outcome (decision 0083).
