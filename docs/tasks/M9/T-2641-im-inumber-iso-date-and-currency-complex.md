---
id: T-2641
title: "IM* inumber: a currency prefix on a complex number is read (probes/eng-inumber-reading A5); probe neighbouring whitespace and date shapes"
milestone: M9
lane: functions-engineering
status: todo
priority: P1
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
IM* functions read `"$1+2i"` as the complex number 1+2i, as Excel does.

## Context
Found while doing T-2541 (probes/eng-inumber-reading, excel-graph 2026-10-25). Excel: `IMABS("2024-01-01")` is `#NUM!` (T-2541: text with an inner minus that is not an exponent's stays unreadable; slash dates are read, `IMABS("1/2/2020")` 43832, probes/eng-inumber-date-boundary). `IMABS("$1+2i")` is 2.23606797749979, i.e. the `$` prefix is accepted before a complex number (ours `#NUM!`). `IMABS("$1,234.50")` is 1234.5 and `IMABS("50%")` 0.5 (already matching). Unprobed: leading NBSP, a trailing tab, LF/CR; a `$` on the imaginary part, `€`.
Code: `complex_of` and `parse_complex_number` in complex/mod.rs.

## Acceptance criteria
- [ ] probes/eng-inumber-reading A5 passes; any extra probe cells are oracle-filled.
- [ ] docs/excel-semantics/engineering.md "How an inumber is read" updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Date-dependent cells (a probe must not depend on the current year).

## Visible-parity stage
Target (decision 0114): every visibly-off cell this task owns is within half a unit of the 15th significant digit
of Excel's value, or has Excel's type, error and text. Bit-level criteria still unmet then move to a follow-up
filed with `defer: phase-1b`.

Phase 1 triage (T-0090, eval of 2026-10-27 at `main` 7f6b373a, task ownership as of `main` e5bc60c4): this task owns the visibly-off probes cells of VALUE (3) and IMABS (2). Fixing them lifts VALUE and IMABS to the Phase 1 rate (≥ 99.9% of attributed cells within 15 digits); no other open task owns enough of those cells.

## Outcome
`complex_of` drops a `$` or `€` that stands before a digit (`strip_currency_symbols`): `IMABS("$1+2i")`, `"$1-2i"`, `"1+$2i"`, `"€1+2i"` are 2.23606797749979, `"$i"` and `"$-5"` `#NUM!`; probe cells A20:A26 oracle-filled. probes/eng-inumber-reading A5 passes (baseline entry removed); engineering.md updated. Unprobed: NBSP, trailing tab, LF/CR, £ and ¥ (follow-up filed).

## Handoff
Sonnet run, three excel-fidelity review rounds, all REQUEST_CHANGES; security, architecture and quality APPROVE. State: branch `agent/functions-engineering/T-2641-im-inumber-currency`, `make check` green on the last head, probes/eng-inumber-reading 26/26 (A5 and A20:A26 oracle-filled, excel-graph 2026-10-28), baseline updated.
Oracle facts: `IMABS` of "$1+2i", "$1-2i", "$1+2j", "$ 1+2i", "1+$2i", "€1+2i" is 2.23606797749979, "$3i" is 3, "$i" is `#NUM!`, "$-5" is `#NUM!`.
Open fidelity findings (last round):
- `strip_currency_symbols` (complex/reading.rs) strips a `$`/`€` at the start of the text or after any non-exponent `+`/`-` (spaces allowed). Evidence covers only the text start (A5, A20:A24, A26) and after a `+` (A25). Shapes "1-$2i", "$1+$2i", "1+ $2i" are extrapolated; ours reads all three as 2.23606797749979, but engineering.md "How an inumber is read" lists them as unprobed with "[ours `#NUM!`]", which is wrong.
- Next steps: either probe "1-$2i", "$1+$2i", "1+ $2i", "1E-$3", "$.5+2i" on excel-graph (`rustsheet-oracle fill -b excel-graph --suite probes --case probes/eng-inumber-reading.case.json`; delete the old sidecar first, the fill skips cases that have one) and follow the values, or narrow the code to the evidence (strip only at the start of the text or right after `+`, no space before the sign) and fix the brackets. Then fix the Outcome note, re-run `make check` and reviewers. T-3056 holds the remaining unprobed shapes.
