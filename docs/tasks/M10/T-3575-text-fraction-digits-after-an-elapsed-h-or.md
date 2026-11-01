---
id: T-3575
title: "TEXT: fraction digits after an elapsed [h] print (TEXT(12345.678,\"[h].0 General\") is \"296296.2 2.500000331\")"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
`.0` right after an elapsed `[h]` is a second fraction, as it is after `s` and `[s]`: `TEXT(12345.678,"[h].0 General")`
is "296296.2 2.500000331" in Excel. The engine returns `#VALUE!` because the parser only accepts `.0` after a seconds
token.

## Context
- probes/t3419-elapsed-after-date-b (excel-graph, 2026-10-31, T-3419) Sheet1!A19: Excel "296296.2 2.500000331",
  ours `#VALUE!`.
- The fraction digit is the serial's tenth of a second (12345.678 is 16:16:19.2), and General prints the fraction
  register (number-formats.md "`General` in a date section", the "One register" bullet).
- Probe before implementing: `"[h].0"`, `"[h].00"`, `"[m].0"`, `"[hh].0"`, `"h.0"` and `"mm.0"` (`h.0` may be a
  digit placeholder beside a date token, which is `#VALUE!`), at 12345.678 and at 5.
- Code: the `Token::Period` arm of `Parser::parse_part` (crates/rustsheet-engine/src/formatter/parser.rs), which
  matches only `Second`, `SecondPadded`, `ElapsedSecond` and `ElapsedSecondPadded` before the period.

## Acceptance criteria
- [ ] Probe cells for the questions above are committed with excel-graph sidecars, and the finding is in
      number-formats.md.
- [ ] probes/t3419-elapsed-after-date-b Sheet1!A19 and the new cells pass; their lines leave eval/baseline/probes.txt.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The register rule itself (T-3419, done).

## Visible-parity stage
Decisions 0083 and 0114: the cell is `#VALUE!` against text, so the stage is `semantics`.
