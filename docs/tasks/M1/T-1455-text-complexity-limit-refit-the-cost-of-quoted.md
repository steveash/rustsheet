---
id: T-1455
title: "TEXT complexity limit: refit the cost of quoted text next to tokens and unquoted text (follow-up of T-1449)"
milestone: M1
lane: functions-text
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
TEXT's complexity model prices quoted text by what surrounds it, and matches every probed cell of probes/text-format-length-limit c to j.

## Context
The 250-budget model (formatter/complexity.rs; docs/excel-semantics/number-formats.md "TEXT format code complexity limit") prices a quoted character at 2 regardless of context. Round j (probes/text-format-length-limit-j, T-1449) shows context matters:
- after `d`, a quoted string alone may reach 125 (`d` free), but `"d"`+120 quoted+2 spaces works and +3 fails, `"d"`+122 quoted+1 space fails, `"d"`+110 quoted+8 spaces works and 9 fails, `"d"`+100 quoted+16 spaces fails: a quoted→unquoted switch costs about 4 if `d` costs 0;
- quoted then a token: 122 quoted+`d` works, 123 fails; `d`+121 quoted+`d` works, 122 fails; 124 quoted+`@` (on text) fails;
- `"[>5]0"`+118 quoted is `#VALUE!` (j A37:A42) though `"[>5]0"`+79 spaces works;
- 120 quoted+2 dashes works, 3 fails; unquoted→quoted costs nothing (50 spaces+50 quoted works).
A free MILP over per-boundary weights overfits; look for a structural explanation (e.g. a quoted run is stored as its own record with a header unless it closes the section).

## Acceptance criteria
- [ ] A model with few, explainable weights matches all probed cells of c to j (or the remaining misses are listed with evidence).
- [ ] `complexity.rs` and the semantics entry follow it; T-1434/T-1449 known failures shrink.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Conditions (T-0936), letters as literals (T-1450).
