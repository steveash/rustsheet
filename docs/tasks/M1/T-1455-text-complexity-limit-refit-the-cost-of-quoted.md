---
id: T-1455
title: "TEXT complexity limit: refit the cost of quoted text next to tokens and unquoted text (follow-up of T-1449)"
milestone: M1
lane: functions-text
status: done
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
- [x] A model with few, explainable weights matches all probed cells of c to j (or the remaining misses are listed with evidence).
- [x] `complexity.rs` and the semantics entry follow it; T-1434/T-1449 known failures shrink.
- [x] `make check` green; reviewers approve.

## Out of scope
- Conditions (T-0936), letters as literals (T-1450).
## Outcome
Refit TEXT's complexity model with three context rules and two corrected weights, found with an
integer min-miss search over 463 probed cells and checked out of sample by a new probe round
(probes/text-format-length-limit-k, 43 cells, excel-graph):
- quoted text pays 4 extra when a token or placeholder follows it directly, or when the section's
  closing literal run goes on with unquoted text after it;
- in a section whose closing literal run holds quoted text, date/time letters and `@` are free;
- a condition costs 3 more in a section that also holds quoted text;
- a color costs 2 (was 1), `AM/PM` 3 (was 2).
Every probed complexity cell of c to j now matches; k misses one (`"@ "` + 124 quoted on text, T-1481).
Eval delta: probes known failures 2853 -> 2834 (fixed 20 in -f, -i, -j; 1 new-coverage failure, k A30).
