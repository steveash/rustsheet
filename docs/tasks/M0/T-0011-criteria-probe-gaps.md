---
id: T-0011
title: "Probe the remaining criteria-grammar gaps (<>text, ~ escapes, case, = on blanks)"
milestone: M0
lane: research
status: todo
priority: P2
size: S
depends_on: []
---

## Goal
Oracle evidence for the criteria rules T-0009 could not verify from the existing probe cells.

## Context
docs/excel-semantics/criteria.md: `"<>text"` is still UNVERIFIED (probes/criteria-grammar has no
such cell); the escapes / case / trailing-tilde entry is a hypothesis; D-function criteria trimming: the both-sides-padded case is settled (not trimmed; smoke/d-function-criteria J2, T-0004 smoke fill), leading-only / trailing-only spaces are unprobed; the `"="`
entry does not cover truly blank cells (the probe range has none).

## Acceptance criteria
- [ ] New cells in a probes case (new case or extension of criteria-grammar) covering: `COUNTIF(r,"<>abc")` over mixed numbers/booleans/errors/blanks/text; `"~*"`, `"~?"`, `"~~"`, and `"abc~"` against a cell holding "abc~"; D-function criteria with leading/trailing spaces (trimming); mixed-case wildcard and equality; `"="` over a range with a truly blank cell.
- [ ] `rustsheet-oracle fill -b excel-graph --suite probes` sidecar committed; new known failures recorded via `make eval-baseline`.
- [ ] criteria.md entries updated (VERIFIED or corrected) citing the cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
Engine changes for any mismatch found (file engine/functions tasks).
