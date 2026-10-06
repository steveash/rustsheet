---
id: T-1025
title: "WEEKNUM out-of-range return_type reuses the last valid start weekday: model or decline (decision)"
milestone: M1
lane: research
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Decide, with a decision record, whether the engine models Excel's stale WEEKNUM return_type state
or declines it as a DEVIATION, and implement the choice.

## Context
Found by T-1023 (excel-fidelity review). In Excel a WEEKNUM return_type >= 22 or < 0 is not an
error: it reuses the start weekday of the last WEEKNUM with a valid type (1, 2, 11-17, 21; 21
leaves "start Monday") evaluated earlier in the same recalc, and is `#NUM!` only when none has
run. On a fresh load Excel evaluates bottom to top, so `A1=WEEKNUM(100,99)`, `A2=WEEKNUM(100,2)`
gives 16, 16 while the swapped layout gives 16, `#NUM!`. ISOWEEKNUM does not set the state.
Evidence: probes/weeknum-stale-return-type (known failures), docs/excel-semantics/dates.md
"WEEKNUM return_type edge cases". The engine currently always gives `#NUM!` (`fn_weeknum`,
`TODO(T-1025)`); modelling it needs Excel's calc order, which RustSheet does not reproduce.

## Acceptance criteria
- [ ] A decision record (`make new-decision`) chooses model or DEVIATION, with reasons.
- [ ] dates.md and the code comment follow the decision; probe cells pass or stay as documented known failures.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Reproducing Excel's full calc order.
