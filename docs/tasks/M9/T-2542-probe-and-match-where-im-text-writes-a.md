---
id: T-2542
title: "Probe and match where IM* text writes a large part in plain notation (IMPOWER('1-2i',59), gen-m9, 1 cell)"
milestone: M9
lane: functions-engineering
status: done
priority: P3
size: S
tier: sonnet
depends_on: [T-1966]
campaign: semantics
---

## Goal
IM* text writes a large part in plain notation where Excel does (`IMPOWER("1-2i",59)` is "-331147245179666000000-252614032879411000000i"), with the cut to E notation probed.

## Context
Gap triage of the T-2258 gen-m9 engineering sweep (docs/excel-semantics/engineering.md "Engineering functions in the gen-m9 engineering sweep (T-2258)" "Large parts in plain notation"). Ours writes -3.31147245179666E+20. T-1966 (done on its pushed branch, not merged) found `IMCOT("1E-20")` is "100000000000000000000" (probes/eng-imtan-overflow B10) and drafted the probe as a task on that branch (numbered T-2186 there); this task replaces that draft, so drop it when T-1966 merges. `1E+200` is E notation (probes/eng-imdiv A19), and T-2056 settled the small parts. Probe parts 1E+15 to 1E+25 (and 123456789012345678901) through IMSUM and IMPRODUCT, real and imaginary.
Code: `complex_part_to_text` in complex.rs.

Owned gen-m9 cells (T-2258 triage, excel-graph, 2026-10-23): 1 known-failure lines in eval/baseline/gen-m9.txt, 1 formulas, 1 visibly off.
Examples:
- gen-m9/e707d43a891ba9b5 Calc!A330 `=IMPOWER("1-2i",59)`: Excel "-331147245179666000000-252614032879411000000i", ours "-3.31147245179666E+20-2.52614032879411E+20i"
Cells:
- gen-m9/e707d43a891ba9b5.case.json (1): Calc!A330

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2258): these 1 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] A probe case with exponents 15 to 25 is oracle-filled and passes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other T-2258 triage families: each has its own task (T-2258 Outcome).
## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2258): every cell this task owns has the wrong type, error, text or value (none is a last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria states it.

## Outcome
IM* parts now write plain notation up to decimal exponent 20 (zero-padded past 15 digits) and E notation from 21 (`complex_part_to_text`). New probe `probes/eng-im-large-part` (36 cells, excel-graph) passes; engineering.md entry resolved. Eval delta: gen-m9 -1 known failure (A330), no new known failures.
