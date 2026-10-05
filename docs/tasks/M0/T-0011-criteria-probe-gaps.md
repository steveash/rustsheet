---
id: T-0011
title: "Probe the remaining criteria-grammar gaps (<>text, ~ escapes, case, = on blanks)"
milestone: M0
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Oracle evidence for the criteria rules T-0009 could not verify from the existing probe cells.

## Context
docs/excel-semantics/criteria.md: `"<>text"` is still UNVERIFIED (probes/criteria-grammar has no
such cell); the escapes / case / trailing-tilde entry is a hypothesis; D-function criteria trimming: the both-sides-padded case is settled (not trimmed; smoke/d-function-criteria J2, T-0004 smoke fill), leading-only / trailing-only spaces are unprobed; the `"="`
entry does not cover truly blank cells (the probe range has none).

## Acceptance criteria
- [x] New cells in a probes case (new case or extension of criteria-grammar) covering: `COUNTIF(r,"<>abc")` over mixed numbers/booleans/errors/blanks/text; `"~*"`, `"~?"`, `"~~"`, and `"abc~"` against a cell holding "abc~"; D-function criteria with leading-only / trailing-only spaces and a padded operand after the operator (`">"&" 1 "`, i.e. `> 1 `), to tell "no trimming" from "trim, but a padded operand stays text"; mixed-case wildcard and equality; `"="` over a range with a truly blank cell.
- [x] `rustsheet-oracle fill -b excel-graph --suite probes` sidecar committed; new known failures recorded via `make eval-baseline`.
- [x] criteria.md entries updated (VERIFIED or corrected) citing the cells.
- [x] `make check` green; reviewers approve.

## Out of scope
Engine changes for any mismatch found (file engine/functions tasks).

## Outcome
New probe `probes/criteria-gaps` (22 cells, excel-graph 2026-10-05). Findings in
docs/excel-semantics/criteria.md, all now VERIFIED:
- `"<>abc"` counts numbers, booleans, errors and truly blank cells (10 of 12).
- `~` escapes only `*` and `?`. Elsewhere it is a literal tilde, and `~~` is **not** an escape:
  `"~~"` does not match "~", and `"a~~c"` does not match "a~c". A trailing `~` is literal.
- Matching is case-insensitive.
- `"="` counts truly blank cells.
- D-function criteria: leading spaces make the criterion plain text (`" >1"` → 0). Spaces around
  the numeric operand after the operator are ignored (`">1 "`, `"> 1"`, `"> 1 "` → 6), and the
  same holds in COUNTIF. Plain-text prefix matches keep their spaces.

The 4 cells where the engine differs (padded numeric operand: C16–C18, C22) are recorded as known
failures; follow-up T-0238. Eval: probes known failures 20 → 24 (4 new known failures, all in the new case), 0 regressions.
