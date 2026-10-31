---
id: T-3445
title: "fold_char follows Excel's comparison fold, not Rust's case tables (probes/t3298-case-mapping-sweep C and D)"
milestone: M2
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3444]
---

## Goal
MATCH (type 0), VLOOKUP(..,FALSE), HLOOKUP(..,FALSE), XMATCH(..,2) and wildcard criteria fold case with Excel's
comparison table, so every cell of probes/t3298-case-mapping-sweep column C and D passes.

## Context
- Failing cells: probes/t3298-case-mapping-sweep Calc!C8, C10, C15, C17:C21, C25:C27, C32:C48 and D8, D10, D15,
  D17:D21, D25:D27, D32, D34:D48.
- lookup.md "MATCH, VLOOKUP and HLOOKUP exact-match keys fold case one character to one" has the block summary
  (T-3298); T-3444 pins the per-character rule and the weightless characters that compare equal to one another.
- Code: `fold_char` and `fold_case` (crates/rustsheet-engine/src/functions/util/collation.rs), reached through
  `folded_units` (`ExactKey`) and `WildcardPattern`. Today they take Rust's `to_lowercase`, which folds Cherokee,
  Mtavruli, Latin Ext-D and the supplementary scripts that Excel keeps apart.
- The indexed exact search (`ExactKey`) must stay consistent with the linear one under the new fold, including the
  weightless-equal rule; keep the fold per UTF-16 unit or scalar consistent with surrogate pairs (they never fold).

## Acceptance criteria
- [ ] Calc!C1:D48 of probes/t3298-case-mapping-sweep pass and leave eval/baseline/probes.txt, 0 regressions;
      probes/t3133-lookup-criteria-case-fold keeps passing.
- [ ] Unit tests cover one character per block class (folds, stays apart, weightless).
- [ ] lookup.md's section says "Implemented in" with this task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- UPPER and LOWER (T-3443). Sorting and the `=` collation unless T-3444 shows they share the table (then a follow-up).
