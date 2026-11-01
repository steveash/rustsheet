---
id: T-3419
title: "TEXT: an elapsed token after a date token beside General counts from General's clock residue (TEXT(5,\"d [h] General\") is \"5 12 0.5\")"
milestone: M10
lane: functions-text
status: done
priority: P3
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
An elapsed `[h]`, `[m]` or `[s]` token that stands after a date or clock token in a section that also holds `General`
formats like Excel. `TEXT(5,"d [h] General")` is "5 12 0.5" in Excel, and the engine gives "5 120 0.5".

## Context
- probes/t3378-general-in-date-section (excel-graph, 2026-10-31, T-3378) has these cells:
  - probes/t3378-general-in-date-section (1): Sheet1!A59
  - A59 `TEXT(5,"d [h] General")`: Excel "5 12 0.5", ours "5 120 0.5".
- Related cells T-3378 already matches:
  - A60 `TEXT(5,"[h] d General")` is "120 5 0.5". The elapsed token comes first and counts from the serial.
  - A58 `TEXT(5.5,"[h]:mm General")` is "132:00 0.5".
  - A21 `TEXT(5,"[h] General")` is "120 5".
- Hypothesis, UNVERIFIED. Excel holds one value register. A date or clock token splits the serial and leaves General's
  clock residue in the register (0.5 at 5; number-formats.md "`General` in a date section", T-3378). An elapsed token
  after the split counts from the register as a serial: floor(0.5 * 24) = 12.
- Probe before implementing:
  - `"d [m] General"` and `"d [s] General"`.
  - `"d [h]"` without General, which the engine already formats.
  - `"h [h] General"`.
  - `"d [h] General"` at 45000 and at 12345.678.
  - `"General d [h]"`.
  These separate "after a split" from "with General in the section".
- Also probe the unprobed extrapolations T-3378 left in place. These are the excel-fidelity review minors on PR #2565:
  - `TEXT(3000000,"[h] General")` and `TEXT(-5,"[s] General")`, `TEXT(-5,"[m] General")`. A General beside a lone
    elapsed token keeps a past-max or negative serial formattable; only A49 `TEXT(-5,"[h] General")` is probed.
  - `"[s].0 General"`: an elapsed token with fraction digits counts as elapsed-only, so General prints the value.
  - `TEXT(5,"d 0 General")`: the engine and a unit test say `#VALUE!` (a digit beside a date token), with no oracle
    cell yet.
  - `TEXT(45000,"mmm d, yyyy General")`: a literal comma in a date section.
  - `TEXT(12345.678,"s.0 General")` and `TEXT(0.99999,"s.00 General")`: the fraction-digit residue when the time of
    day has seconds (only whole serials are probed).
- Code: the `ParsePart::Date` arm of `format_number` (crates/rustsheet-engine/src/formatter/format.rs) and
  `general_in_date_section` in elapsed.rs.

## Acceptance criteria
- [x] Probe cells for the questions above are committed with excel-graph sidecars. The finding goes into
      number-formats.md "`General` in a date section".
- [x] probes/t3378-general-in-date-section Sheet1!A59 and the new cells pass, and their known-failure lines leave
      eval/baseline/probes.txt (all but t3419-elapsed-after-date-b A19, out of this rule and filed as T-3575).
- [x] `make check` green; reviewers approve.

## Out of scope
- `General` beside digit placeholders (T-1614, stays `#VALUE!`).

## Visible-parity stage
Decisions 0083 and 0114: the cell is text against text with different digits, so the stage is `semantics`.

## Outcome
Excel keeps one value register in a date section. The value General prints is that register, and an elapsed token
after any date or clock token counts from it, with or without General: `TEXT(5,"d [h]")` is "5 12" and
`TEXT(12345.678,"d [h]")` is "18 472". With fraction digits the register is the fraction residue in its own units
(`"s.0 General"` on 12345.678 is "19.2 2.500000331", `"[s].0 General"` on 5 is "432000.0 0.5"). An elapsed token after
a clock token of its own or a larger unit (`h [h]`, `h [m]`, `s [s]`, `mm:ss [m]`) is `#VALUE!`. The same rule
explains the old known failure `TEXT(1.5,"AM/PM [h]")` = "PM 12" (probes/text-elapsed-two-tokens).
- Probes: probes/t3419-elapsed-after-date-general (30 cells), t3419-elapsed-after-date-b (25),
  t3419-elapsed-after-clock-c (14), excel-graph 2026-10-31. Findings in number-formats.md "`General` in a date
  section" and dates.md.
- Code: `date_section_register` (was `general_in_date_section`), `elapsed_token` and `elapsed_after_larger_clock`
  in formatter/elapsed.rs; the `ParsePart::Date` arm of `format_number` counts elapsed tokens after a split from
  the register; `Parser::parse_part` rejects an elapsed token after a larger clock token.
- Follow-up: T-3575 (`"[h].0 General"`, probes/t3419-elapsed-after-date-b A19, still `#VALUE!` here).
