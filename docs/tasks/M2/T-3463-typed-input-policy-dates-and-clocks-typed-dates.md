---
id: T-3463
title: "Typed-input policy, dates and clocks: typed dates with a four-digit or two-digit year, month names, clocks, AM/PM and date-plus-clock entries"
milestone: M2
lane: harness
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-3462]
---

## Goal
`rustsheet-gen --policy typed-input` (T-3462) also draws typed **dates and clocks**, so the oracle-filled batch
(T-3464) measures the engine's typed date grammar on shapes nobody hand-picked, without depending on the day the
oracle ran.

## Context
- The en-US typed date grammar is spread over several probe-built rules: dates.md "The en-US date-text grammar
  (DATEVALUE, VALUE, date arguments)" (two-digit years below 30 are 20xx; the date must lie in
  1900-01-01..9999-12-31), "Date text with a month name glued to digits (no separator)", "Typed date plus clock
  with a single-letter A or P suffix is a number (T-1994)", "Time text"; research notes
  docs/research/non-glued-typed-input-2026-10-19.md, typed-clock-shapes-2026-10-19.md,
  single-letter-ap-typed-clock-2026-10-19.md, glued-month-typed-input-2026-10-18.md. Code:
  `formatter::date_text::parse_date`, `split_glued_month` (crates/rustsheet-engine/src/formatter/date_text.rs),
  `Model::parse_datevalue_text`.
- **Clock independence is the design constraint.** Year-less text (`7/1`, `Jul-2`, `1-Jul`, `Feb 29`) takes the
  current year of the machine that typed it (dates.md "The en-US date-text grammar", its year-less bullet; the
  oracle's clock is the real date, the engine's is pinned), so its value is not reproducible. Draw only shapes whose year is written: a
  four-digit year, a two-digit year in a three-part date (`1/2/30`, `Jul 1, 29`), or a month-year whose number
  cannot be a day (`Jul-99`, dates.md "Otherwise `n` is a two-digit year"). A pure clock (`10:00`, `25:30`,
  `1:2:3.5 PM`) has no date part and is allowed.
- Strata to add (shares recorded in docs/architecture/synthetic-generator.md with T-3462's):
  - numeric dates `m/d/yyyy`, `m-d-yyyy`, `yyyy-m-d`, `yyyy/m/d`, two-digit years at the 29/30 pivot, invalid
    days (`2/30/2026`), 1900-02-29 (the leap-year bug), year 9999 and 10000, year 1899;
  - month names: full and three-letter, any case, `d-mmm-yyyy`, `mmm d, yyyy`, `d mmmm yyyy`, glued forms with a
    year (`1Jul2026` is UNVERIFIED in dates.md: draw it, the batch answers it);
  - clocks: `h:mm`, `h:mm:ss`, fractional seconds, hours past 24, minutes past 59, `AM`/`PM`/`A`/`P` with and
    without a space, `0:00 AM`, `12:00 AM`, `13:00 PM`;
  - date plus clock in both orders, with one and two spaces;
  - near misses that must stay text (`1/2/3/4`, `Jul 32, 2026`, `10:00 XM`).
- The readers are T-3462's (`=A<n>+0`, `=ISTEXT(A<n>)`, `=CELL("format",A<n>)`); `CELL("format")` records the
  `D1`…`D9` class Excel picked, which the engine assigns from typed input (dates.md marks those formats
  UNVERIFIED in several places).

## Acceptance criteria
- [ ] `rustsheet-gen --policy typed-input --seed 0` also writes at least 150 date and clock entries over the
      strata above, tagged `typed-input/date/...` and `typed-input/clock/...` in `hypotheses`.
- [ ] tools/tests: a test that no drawn entry is year-less (a date part without a written year), one per stratum
      that the drawn text has its shape, and the determinism test of T-3462 still passes.
- [ ] Shown in the PR: `rustsheet-eval run` with the engine standing in for Excel scores the new cases with no
      crash, panic or timeout.
- [ ] docs/architecture/synthetic-generator.md lists the date strata, the shares and the clock-independence rule.
- [ ] `make py-check` green; `make check` green; reviewers approve.

## Out of scope
- Year-less dates and anything else whose value depends on the clock.
- 1904 workbooks and locales other than en-US.
- The oracle-filled batch (T-3464) and any engine change.
