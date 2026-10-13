---
id: T-1481
title: "TEXT complexity limit: probe @ followed by unquoted text then quoted text (follow-up of T-1455)"
milestone: M1
lane: functions-text
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
TEXT's complexity model prices `@` followed by unquoted then quoted text the way Excel does, so probes/text-format-length-limit-k A30 passes.

## Context
The T-1455 model (formatter/complexity.rs; docs/excel-semantics/number-formats.md "TEXT format code complexity limit") fits every probed cell of probes c to k but one: `TEXT("x","@ "&""""&REPT("a",124)&"""")` works in Excel and costs 251 in the model, while `"d "` and `"dd "` + 124 quoted fail as modeled (k A21:A23), and `"@"` + 125 quoted works and 126 fails (i A26:A27, j A30). Only one cell shows it, so probe first: `"@ "` + 124 and 125 quoted on text and on a number, `"@-"` and `"@  "` (two spaces) + quoted, and `"@"&REPT(" ",83)` vs 84 on text. Then refit only what `@` costs. In the same round, probe the inferred parts of the T-1455 model: `ss`, `ddd`, `mmm` and `A/P` before 125/126 closing quoted characters, quoted text followed by a color or a condition, and a condition with quoted text in a second section.

## Acceptance criteria
- [ ] A new probe round pins what `@` costs before unquoted text; the model and the semantics entry follow it.
- [ ] k A30 leaves the known failures (or the remaining miss is listed with evidence).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Letters as literals (T-1450), conditions (T-0936).

## Outcome
Probe rounds l and m (excel-graph, 71 cells) pinned `@`: in a section whose closing literal run holds quoted text it is refunded 3 (its 2 plus 1), where other free tokens are refunded 2. Model, tests and semantics entry updated; also pinned `ss`, `ddd`, `mmm`, `dddd`, `mmmm` (free) and `A/P` (not free). Eval: probes known failures 2982 -> 2986 (k A30 fixed; 5 new-coverage failures in the new probes, filed as T-1486).
