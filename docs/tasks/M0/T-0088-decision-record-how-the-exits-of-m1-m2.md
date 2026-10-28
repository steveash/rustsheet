---
id: T-0088
title: "Decision record: how the exits of M1, M2, M7, M8, M9, M10 and M11 are measured (decision 0085 for every Phase 1 milestone)"
milestone: M0
lane: research
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
A decision record says how the exit criteria of every Phase 1 milestone not yet measured (M1, M2, M7, M8, M9, M10 and
M11) are read from the eval report, as decision 0085 did for M5 (and M6 by inheritance), so the steward can tell
from a run whether each exit is met and add the Phase 1 exit line (docs/milestones.md: "The exit criteria of M1,
M2, M5-M10 and M11 are met").

## Context
- `crates/rustsheet-eval/src/milestone_exit.rs`: `MILESTONES = ["M5", "M6"]`, with the comment "M7 to M11 need
  their own decision first". `scripts/exit_owners.py --milestone M7` fails with "report.json has no milestone_exit
  row for M7", so nobody can list the failing exit cells of those milestones.
- Each milestone's exit criteria in docs/milestones.md are prose. Inputs the report already computes: catalog
  coverage per milestone, the generated suites (gen-m1, gen-m2, gen-m7, gen-m8a, gen-m8b, gen-m9, gen-m10), the
  probe milestones (decision 0051: M0-M2, M7, M8, M9, M10, M11 rows), the frozen m1-probes list (decision 0043), the
  ironcalc-calc function scope rows, and the real-world suites (calipers, poi, corpus-bench) for M11. M2's criteria
  were settled by T-1472, decision 0058 and decision 0074; M1's frozen list by decision 0043.
- Decision 0114 reads every "bit-exact" in a Phase 1 exit as "not visibly off".
- At the 2026-10-27 eval the probe milestones have 55 (M7), 70 (M8), 817 (M9) and 199 (M10) visibly-off cells.

## Acceptance criteria
- [x] `docs/decisions/NNNN-*.md` (via `make new-decision`) defines, for each of M1, M2, M7, M8, M9, M10 and M11,
      the measures (which suites, rows and thresholds; where an exit says "bit-exact", the visibly-off reading) and
      the named-exception file it may use (decision 0087), citing each milestone's prose exit criterion.
- [x] It weakens no gate (no baseline, ratchet or CI check changes) and names T-0089 as the implementation.
- [x] docs/milestones.md points each milestone's exit criteria at the decision.
- [x] `make check` green; reviewers approve.

## Out of scope
- Implementing the rows (T-0089).

## Outcome
Decision 0146 defines the exit measures of M1, M8, M9, M10 and M11 (M2 keeps decision 0135, M7 decision 0140), read at
visible parity from rows the report computes or that T-0089 adds: M1 from gen-m1, smoke and the `m1-probes` list;
M8 from gen-m8a and gen-m8b, a parser-classified poi subset at 99.5% and the M8 probes; M9 from the cumulative
ironcalc-calc row at 99.95%, `v15_checkpoint`'s per-function rate for every M9 function and the M9 probes; M10 from
gen-m10's displayed-text items and the M10 probes' value and text cells; M11 from corpus-bench at the fixed 99.8%
over at least 200,000 cells and the M11 probes. docs/milestones.md cites it in each of those sections. On the
2026-10-28 numbers M1 and M11 are met, M8, M9 and M10 are not. T-2889, T-2890, T-2891 and T-0487 asked the same
questions per milestone and are dropped as superseded; T-0089 carries the thresholds. No code, baseline or gate
change; eval delta 0.
