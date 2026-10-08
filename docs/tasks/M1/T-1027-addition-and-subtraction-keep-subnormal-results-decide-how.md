---
id: T-1027
title: "Addition and subtraction keep subnormal results; decide how to score kept subnormals"
milestone: M1
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`3E-308-2.5E-308` keeps its subnormal result as in Excel, and the oracle-unscoreable cells are handled by a documented scoring decision.

## Context
Left over from T-0989. probes/subnormal-function-results C69, C73-C75 (and A4, A6, A60-A62, A65) still fail: `arithmetic::plus`/`minus` flush via `finite_or_num` but Excel keeps a subnormal from `+`/`-` (numbers.md "Subnormal results become 0"). Trying it regressed A69, A73-A75 (Graph reports 0.0 for a kept subnormal, so the numeric cells expect 0) and probes/text-number-tiny B3,B5-B8,B18,B20 and text-number-words H6,H7,K1,K4,K5 (`"1e-308"+0` must be 0: flush subnormal text in the number-text cast first). Needs a scoring decision (decision record: protected baseline paths) for cells the oracle cannot score.

## Acceptance criteria
- [x] C69, C73-C75 pass with no regression in text-number-tiny/words.
- [x] Decision record on scoring A4, A6, A60-A62, A65, A69, A73-A75; no hand-edited baseline.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- `arithmetic::plus`/`minus` keep subnormal results (only overflow is `#NUM!`), so `(3E-308-2.5E-308)=0` is FALSE as in Excel.
- `parse_number_text` flushes subnormal number text, so `"1e-308"+0` stays 0 (text-number-tiny and text-number-words do not regress).
- Decision 0042: an excel-graph 0 against a kept subnormal scores as a pass (`graph_hides_subnormal`, runner.rs), because Graph reports both as 0. The `C` companions carry the evidence. The `excel-graph` kind is now one constant (`case::GRAPH_ORACLE`).
- Eval delta: probes fixed 11 (subnormal-function-results now 129/129), regressions 0.
- Follow-up: T-1142 checks that every probe scored through decision 0042 keeps a `(x)=0` companion.
