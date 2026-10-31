---
id: T-3419
title: "TEXT: an elapsed token after a date token beside General counts from General's clock residue (TEXT(5,\"d [h] General\") is \"5 12 0.5\")"
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
- Code: the `ParsePart::Date` arm of `format_number` (crates/rustsheet-engine/src/formatter/format.rs) and
  `general_in_date_section` in elapsed.rs.

## Acceptance criteria
- [ ] Probe cells for the questions above are committed with excel-graph sidecars. The finding goes into
      number-formats.md "`General` in a date section".
- [ ] probes/t3378-general-in-date-section Sheet1!A59 and the new cells pass, and their known-failure lines leave
      eval/baseline/probes.txt.
- [ ] `make check` green; reviewers approve.

## Out of scope
- `General` beside digit placeholders (T-1614, stays `#VALUE!`).

## Visible-parity stage
Decisions 0083 and 0114: the cell is text against text with different digits, so the stage is `semantics`.
