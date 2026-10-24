---
id: T-2583
title: "NETWORKDAYS.INTL with a blank weekend cell and an error in holidays is #NUM!, the weekend check first (gen-m9, 1 cell)"
milestone: M6
lane: functions-date
status: done
priority: P2
size: S
tier: sonnet
depends_on: [T-2271]
campaign: visible
---

## Goal
NETWORKDAYS.INTL with a blank weekend cell is `#NUM!` even when holidays hold an error, as Excel does.

## Context
From T-2436's probe of the untraced gen-m9 cells. dates.md (T-2271's rule: a blank weekend cell is `#NUM!`); this cell adds that the weekend check comes before the holidays' error (`#N/A` element). T-2271 owns the blank-weekend change; land this after it.

T-2436 probe (excel-graph, 2026-10-24; probes/t2436/<case>, each gen-m9 cell's sub-expressions on its Data sheet): `NETWORKDAYS.INTL(31,4886,<blank>,{1,2;#N/A,""})` is `#NUM!` (ours `#N/A`): the blank weekend cell's `#NUM!` (T-2271's rule) before the holidays' error.
Owned gen-m9 cells (T-2436 probe, excel-graph, 2026-10-24): 1 known-failure line in eval/baseline/gen-m9.txt, 1 formula.
Examples:
- gen-m9/c2974c061384fdd9 Calc!A149 `=FDIST(1,7,(NETWORKDAYS.INTL(31,4886,Data!E5,{1,2;#N/A,""})+BESSELK(12,4)))`: Excel #NUM!, ours #N/A; inner `NETWORKDAYS.INTL(31,4886,Data!E5,{1,2;#N/A,""})` Excel #NUM!, ours #N/A
Cells:
- gen-m9/c2974c061384fdd9.case.json (1): Calc!A149

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2436): this known-failure line disappears from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] The rule's quirks are in docs/excel-semantics/ with the probe cells as evidence; new probe cells are gated or baselined.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Cells of the same functions owned by other tasks.

## Outcome
Already passing on `origin/main`: gen-m9/c2974c061384fdd9 Calc!A149 is not in eval/baseline/gen-m9.txt (T-2271's weekend coercion landed after the T-2436 probe). Added unit test `networkdays_intl_blank_weekend_error_beats_the_holidays_error` and marked the dates.md entry implemented. Eval delta: none.
