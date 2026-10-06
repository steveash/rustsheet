---
id: T-0176
title: "Decision: how cells failing by an accepted DEVIATION count toward milestone exit (probes/resource-limits C12, C14)"
milestone: M1
lane: research
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
A decision record that says how a cell failing *by design* under an accepted DEVIATION
(decision 0014: directive 5 beats directive 1) is treated by the milestone exit criteria that demand
100% bit-exact on a suite, so M1 can exit with probes/resource-limits C12 and C14 still differing
from Excel, or so those cells stop being probes.

## Context
docs/milestones.md M1 exit: `smoke`, `probes` and `gen-m1` at 100% bit-exact. probes/resource-limits
C12 `=ROWS(SEQUENCE(1000,1001))` and C14 `=ROWS(SEQUENCE(1048576))` are known failures by design:
Excel gives 1000 and 1048576, RustSheet gives `#ERROR!` above 1,000,000 array cells to bound
allocation (docs/excel-semantics/arrays.md "Array results above 1,000,000 cells are #ERROR!",
Status: DEVIATION). The Phase 1 exit text says each remaining failure needs "a filed task or a
`DEVIATION` entry", while the M1 line says 100%, so the two definitions disagree, and the steward
cannot tell whether M1 is exitable once T-0134 and T-0221 land. Other DEVIATION entries
(text.md, coercion.md decision 0033, syntax.md nesting cap, engineering.md, xlsx-storage.md) may have
cells in later suites with the same question.

Options to weigh (decide one, with reasons): (a) milestone exit counts known failures that a
`DEVIATION` entry cites as accepted, and the steward checks them against the entries;
(b) the case format marks such cells (for example a per-cell `deviation = "decision 0014"` tag)
so the harness scores them in a separate column and the gate still pins their current value;
(c) raise the limit so Excel's values fit, if a bounded design exists (cost to state); (d) move the
cells out of the gated probes suite. Constraints: CLAUDE.md directive 2 (no weakened gates, no
hand-edited baselines), decision 0002 (strict scoring), decision 0014.

## Acceptance criteria
- [x] `make new-decision` record in docs/decisions/ choosing one option, with the rejected options
      and why; it says how the steward verifies M1 exit for these two cells.
- [x] docs/milestones.md M1 (and the Phase 1 exit paragraph if it changes) states the rule in one
      sentence, citing the decision; if the decision needs harness or case-format work, that task is
      filed (harness lane) and named in the decision.
- [x] `make check` green; reviewers approve (Excel-fidelity reviewer included).

## Out of scope
- Implementing any harness change the decision calls for (file it).
- Changing the 1,000,000-cell limit itself.
- Editing the probes/resource-limits C12/C14 cases or the M1 exit text before this decision lands.

## Outcome
Decision 0036 chooses option (b): a per-suite `deviations` file (case, cell, decision, DEVIATION
entry, pinned by-design value), modelled on decision 0032's `cached_overrides` so xlsx suites are
covered too. Listed cells are scored in their own column, with the pinned value gated, and left out
of bit-exact. Options (a) (interim only), (c) and (d) are rejected with reasons. docs/milestones.md
(Phase 1 exit and M1) states the rule. Until the harness work lands, the interim steward rule
accepts only probes/resource-limits C12/C14. Filed T-0191 (harness, M1, P1, opus) for the
implementation. Docs only; eval unchanged.
