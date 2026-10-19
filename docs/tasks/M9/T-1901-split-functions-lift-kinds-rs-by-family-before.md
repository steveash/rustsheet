---
id: T-1901
title: "Split functions/lift/kinds.rs by family before it passes the 1000-line limit"
milestone: M9
lane: quality
status: in-progress
priority: P2
size: S
tier: opus
depends_on: [T-1873]
---

## Goal
The SPEC-0011 lifting table can take new entries without squeezing comments: functions/lift/kinds.rs is split so each
part stays well under the 1000-line file-size limit, with no change to any entry's kinds or weight.

## Context
After T-1841 and T-1873 kinds.rs is 999 lines (scripts/check_file_size.py limit 1000). T-1873 kept it under by moving
the HYPGEOM.DIST cost measurements into the `MAX_TERMS` doc in statistical/hypegeom.rs and replacing the
`// --- X (T-nnnn) ---` / `// --- End X ---` markers in `lifting_table_functions!` with an inline `// T-1873` (PR #1086
architecture review). The next lifting entry would push it over.
A split by family (for example financial ATP, statistical, engineering, the rest), with the `lifting_table_functions!`
macro, `ArgKinds`, `Lift`, `KernelWork` and `MAX_LIFTED_CELL_VISITS` in a shared parent, keeps each section's weight
constants and their measurement comments next to its entries.

## Acceptance criteria
- [ ] kinds.rs is split into a parent and per-family modules, each well under 1000 lines; the table's entries, kinds and
  weights are unchanged (`eval/catalog/lift-kinds.json` regenerates byte-identical with `rustsheet-eval lift-kinds --write`).
- [ ] The T-1873 entries get back the `// --- X (T-1873) ---` / `// --- End X ---` markers the other sections use.
- [ ] No entry added to quality/oversized-files.json; `scripts/check_duplication.py` does not rise.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing any kind or weight, or adding entries.
