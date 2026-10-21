---
id: T-2205
title: "Probe IMAGE with a grid source, a scalar error option and a wider array option"
milestone: M1
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's result for IMAGE with an array source, a scalar error option and a wider array option is known, and the engine matches it.

## Context
Found in T-1962 review: `offline_args` (functions/information/offline_errors.rs) returns `error_grid` in the source's shape as soon as a scalar option is an error beside a grid source, so it never broadcasts against a wider array option: `IMAGE({url,url},"alt",#DIV/0!,{5,5,5})` is 1x2 of `#DIV/0!` today. Excel may give 1x3 (`#DIV/0!`, `#DIV/0!`, `#DIV/0!` or `#N/A` in the third column). The scalar-source case already lifts (information.md, T-1427). Use the `excel-probe` skill; read the shape with ROWS/COLUMNS and each element with ERROR.TYPE(INDEX(...)).

## Acceptance criteria
- [ ] A probe case for grid source + scalar error option + wider array option (row and column shapes) has excel-graph values; information.md records the rule.
- [ ] The engine matches (or a follow-up is filed); `make check` green; reviewers approve.

## Out of scope
- STOCKHISTORY's first-error rule.

## Outcome
Probed (excel-graph, probes/rich-errors-image-grid-error-option): a scalar error option beside a grid source and a wider array option lifts over the broadcast shape (`IMAGE({url,url},"alt",1/0,{5,5,5})` is 1x3: `#DIV/0!`, `#DIV/0!`, `#N/A`). `offline_args` now defers the error for `fn_image` when an array option exists; the rule is in information.md. Eval: probes +1 case, 25 cells pass, no regressions.
