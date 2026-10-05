---
id: T-0910
title: "Port IronCalc TIMEVALUE fractional-seconds fix"
milestone: M9
lane: functions-date
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Upstream IronCalc accepts fractional seconds in TIMEVALUE (`Ported-From: ironcalc@cead0e28`,
follow-up test cleanup `13dbb103`). Port it, but only after confirming Excel's behavior.

## Acceptance criteria
- [x] Excel evidence for `TIMEVALUE("10:11:12.5")` and neighbours (cached values in an
  Excel-authored workbook, or oracle via `excel-probe`); cite it in `docs/excel-semantics/`.
- [x] Spec case(s) added under `eval/suites/`; implementation matches; `make check` green.
- [x] Commit message carries `Ported-From: ironcalc@cead0e28`.

## Progress
Partial (2026-10-03): the "date time" text case (TIMEVALUE.xlsx B18) is fixed with Excel-cached
evidence. Fractional seconds are still open: no Excel evidence exists in-repo and the oracle
had not been set up (it is live since 2026-10-04; use `excel-probe`).

## Outcome
Done (2026-10-05). Four oracle probes (excel-graph; `probes/timevalue-fractional-seconds`,
`-fractional-seconds2`, `-component-limits`, `-ampm-limits`, 113 TIMEVALUE cells) showed Excel's
clock grammar is much wider than upstream's fix: `M:S.f`, millisecond half-up rounding on the
digits, components up to 9999, minutes/seconds past 59 without AM/PM, single-letter `A`/`P`,
spaces around `:`/`.`. TIMEVALUE now parses clock text with one evidence-backed
`parse_clock_text` (replacing the 24:00/23:60 normalisation helpers and the time-only chrono
patterns) and returns `serial - floor(serial)`, bit-exact with Excel. Behaviour was re-derived
from the oracle rather than applying the upstream patch, which handles fractional seconds only.
Two upstream unit tests that asserted non-Excel #VALUE! results (`25:00`, `14:70`, milliseconds)
were corrected to the oracle values. Rules: `docs/excel-semantics/dates.md#time-text`.
Eval: probes +4 cases; 112/113 TIMEVALUE cells pass (`"10:11.5:12"` oddity baselined); the 89
text-to-number coercion cells (`0+A1`, `--A1`) are new-coverage known failures → T-0146 (and
T-0920 for the `--x` parser fold). No change in other suites.
