---
id: T-2442
title: "Decode the rich error of errorType 6 (subType 7) from ERROR.TYPE evidence"
milestone: M7
lane: harness
status: todo
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
- [ ] The formula and cell that store (6, 7) are identified, and a probe records ERROR.TYPE and VALUETOTEXT through
      INDEX for it (excel-probe skill).
- [ ] `rich_error_name` maps 6 if the evidence agrees, with a unit test; the affected cases are re-filled with
      `rustsheet-oracle fill --force` and `make eval-baseline` is run.
- [ ] The same probe also reads ERROR.TYPE through INDEX for one errorType 2 rich error with a non-zero subType
      (39 or 133, the EXPAND/GROUPBY cells in the cached gen-m7 recalcs), which T-2417 decoded by the T-0030 rule only.
- [ ] arrays.md records the finding.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Rich errors other than errorType 6.
