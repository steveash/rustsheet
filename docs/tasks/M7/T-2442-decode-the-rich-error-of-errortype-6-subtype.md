---
id: T-2442
title: "Decode the rich error of errorType 6 (subType 7) from ERROR.TYPE evidence"
milestone: M7
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The rich error `_error(errorType, subType)` = (6, 7) that excel-graph writes is decoded from ERROR.TYPE evidence (or
shown to need another reading), so its cells stop being `#UNDECODED!`.

## Context
Split from T-2417, which decoded errorType 2 (`#VALUE!`, probes/rich-errors-spilled-lambda). A scan of the cached
excel-graph recalcs (eval/.cache/oracle/excel-graph, 2026-10-22) found `_error` (errorType 6, subType 7) values in 17
workbooks, gen-m9 and gen-m7 among them, but the scan did not tie them to a cell and formula. By the rule
errorType = ERROR.TYPE − 1 it would be `#N/A` (ERROR.TYPE 7), but no cell shows that yet. docs/excel-semantics/arrays.md
"Cached rich errors" has the structure notes. `rich_error_name` in crates/rustsheet-xlsx/src/import/rich_errors.rs
maps it to nothing today. Lead: every (6, 7) rv in the cached recalcs (e.g. 354adcd2…, e07a9239…,
d7d6da72…, 652670ad…) carries a fallback `<fb t="e">#N/A</fb>`, Excel's own fallback value, which points to `#N/A`.

## Acceptance criteria
- [x] The formula and cell that store (6, 7) are identified, and a probe records ERROR.TYPE and VALUETOTEXT through
      INDEX for it (excel-probe skill).
- [x] `rich_error_name` maps 6 if the evidence agrees, with a unit test; the affected cases are re-filled with
      `rustsheet-oracle fill --force` and `make eval-baseline` is run.
- [x] The same probe also reads ERROR.TYPE through INDEX for one errorType 2 rich error with a non-zero subType
      (39 or 133, the EXPAND/GROUPBY cells in the cached gen-m7 recalcs), which T-2417 decoded by the T-0030 rule only.
- [x] arrays.md records the finding.
- [x] `make check` green; reviewers approve.

## Out of scope
- Rich errors other than errorType 6.

## Outcome
(6, 7) is REGEXEXTRACT's no-match `#N/A` (the gen-m7 lead `REGEXEXTRACT(..,2)` cells were a different error: a
pattern without capture groups in return_mode 2 is (2, 133) = `#VALUE!`). New probe probes/rich-errors-regex-group
(103 cells, excel-graph, 2026-10-23, all pass): (6, 7) and (6, 9) (a capture group that took no part) have ERROR.TYPE 7,
VALUETOTEXT `#N/A` and ISNA TRUE, directly and through INDEX, and carry an `<fb t="e">#N/A</fb>` fallback; (2, 133), also
with a `ptg` field, has ERROR.TYPE 3 through INDEX. `rich_error_name` maps "6" to `#N/A` (unit tests
`error_codes_follow_the_probes`, `regex_group_errors_are_decoded`); arrays.md records the storage per cell.
Re-fill: probes/groupby-lifted-args2 (its one `#UNDECODED!`, A27, is now `#VALUE!` and passes). No committed sidecar holds
an errorType 6 cell. gen-m7/0624b684889e9228's 22 `#UNDECODED!` cells are stale errorType 2 (pre-T-2417 fill); its re-fill
also scores the 335 spill members of A2004 that T-2415 fixes, so T-2415 does that re-fill (its Context already says so).
Eval delta: probes +1 case (103 cells, all pass), groupby-lifted-args2 46 -> 47 checked cells; 0 regressions,
no known-failure changes.
