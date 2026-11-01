---
id: T-3610
title: "Decide how the eval scores an oracle cell whose Excel value varies between fills of the same bytes"
milestone: M9
lane: harness
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A decision record (`make new-decision`) that says how the eval scores an oracle cell whose Excel value differs between
two genuine fills of the same input bytes. Options include keeping it a named known failure, excluding it from scoring,
or requiring N agreeing fills before a value is a target. The task applies the decision to the cells known today.

## Context
- T-3590 (financial.md "AMORDEGRC below period 1"): gen-m9 d912671baa1364aa Calc!A76 is 1 in its 2026-10-28
  excel-graph fill. Probes/amordegrc-a76-bisect-a, the same input sha256 (685de544…), is 0 on 2026-11-01, as is -h.
  The other 412 cells agree. The cell stays an F line in eval/baseline/gen-m9.txt against the 10-28 value. Ours is 0.
- Decision 0112 handled one such class, LN FYL2X near-ties that vary between fills (probes-numeric/t2143-ln-near-ties-fill1..3).
  It keeps them as named known failures and changes no scoring. Its principle is that bits which vary between fills
  are not targets. It is scoped to LN.
- Raised by the excel-fidelity review of T-3590 (PR #2685), MINOR finding 1.

## Acceptance criteria
- [ ] A decision record states the general rule, how a cell is shown to vary (how many fills, by whom, and what
      counts as the same bytes), and where such cells are listed. Re-filling until a run matches us is ruled out.
- [ ] gen-m9 d912671baa1364aa Calc!A76 and decision 0112's LN cells are treated per the decision, and
      `make eval-baseline` records no regression.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Finding why A76 was 1 on 10-28 (T-3590 ruled out what it could).
- Changing decision 0112's LN model.
