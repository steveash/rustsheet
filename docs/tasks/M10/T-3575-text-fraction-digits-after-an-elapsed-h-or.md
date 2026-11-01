---
id: T-3575
title: "TEXT: fraction digits after an elapsed [h] print (TEXT(12345.678,\"[h].0 General\") is \"296296.2 2.500000331\")"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: S
tier: opus
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

## Outcome
`.0`/`.00` after `h`, `hh`, `mm`, `[h]`, `[hh]`, `[m]` is the second fraction (probes/t3575-fraction-after-elapsed,
15 cells, excel-graph). The parser accepts it after any hour/minute/second token; a lone padded `mm.0` reads as a
minute. `m.0`, `mm.0` beside other tokens and `.000` after h/m are unprobed (follow-up: probe them).
Eval delta: probes known failures 3203 -> 3201 (t3419-elapsed-after-date-b A19 and elapsed-no-seconds-boundary A32
fixed; the new case is a fully passing C line); no regressions.

## Handoff
Tried: parser accepts `.0` after any h/m/s token (`takes_second_fraction` in elapsed.rs, used by the `Token::Period`
arm) and a lone padded `mm.0` reads as a minute (`lone_mm_fraction`). Probe t3575-fraction-after-elapsed (15 cells,
filled) passes; `make check` green at 561b85d4d (parser.rs is exactly 1000 lines, the ratchet limit).
Excel-fidelity reviewer requested changes twice. Open findings (round 2):
- `lone_mm_fraction` only checks nothing precedes `mm`, so `mm.0 yyyy` / `mm.0 d` now print a value (was `#VALUE!`);
  unprobed. Narrow to nothing after the fraction, or probe those formats.
- `.000` after h/m/[h] is accepted (limit is still `MAX_SECOND_FRACTION_DIGITS`); only `.0`/`.00` are probed. Cap at 2
  digits unless the previous token is a seconds token, or probe `.000`.
- `h:mm.0`, `hh:mm.0`, `h mm.0`, `h:mm.00` now print (unprobed; only `[h]:mm.0` is evidenced). Add them to the probe.
Next: add probe cells for the above, fill with `rustsheet-oracle fill -b excel-graph --suite probes`, implement what
Excel does, update number-formats.md, re-run the reviewers. Keep parser.rs <= 1000 lines (move code out if needed).
