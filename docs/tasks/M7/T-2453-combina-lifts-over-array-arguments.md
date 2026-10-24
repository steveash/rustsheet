---
id: T-2453
title: "COMBINA lifts over array arguments"
milestone: M7
lane: functions-math
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: visible
---

## Goal
COMBINA lifts over array and range arguments, as Excel does.

## Context
Evidence: probes/t2414-gen-m7-unexplained A191-A271 and docs/excel-semantics/numbers.md "COMBINA lifts over arrays". Ours is `#N/IMPL!` (array) or `#VALUE!` (range) because COMBINA has no lifting-table entry (see how T-2410 lifted ROMAN: `LIFT_1_OF_2`).

Owned gen-m7 cells (T-2414):
- f19581ac44da1653 A3357: Excel #VALUE!, ours #N/IMPL!

## Acceptance criteria
- [x] probes/t2414-gen-m7-unexplained cells A191, A201, A211, A221 (+A222), A251, A261 (+A262:A267), A271 pass; lines leave eval/baseline/probes.txt.
- [x] Visible parity: gen-m7/f19581ac44da1653 A3357 (`=VSTACK(COMBINA(WRAPROWS(..),4),CHOOSE(3,..))`, Excel #VALUE!) passes and its line leaves eval/baseline/gen-m7.txt.
- [x] `make check` green; reviewers approve.

## Out of scope
- Arguments the probe did not cover.

## Visible-parity stage
Decision 0083: every cell this task owns has the wrong type, error or value, so the task is in the campaign; done when they have Excel's.

## Outcome
COMBINA lifts both arguments (`lift_call`; its own `COMBINA` table entry, `Weighted(1 << 10)` for combin's overflow-bounded loop). Removed 14 lines from eval/baseline/probes.txt (t2414-gen-m7-unexplained) and 1 from gen-m7.txt (f19581ac44da1653 A3357).
