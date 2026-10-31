---
id: T-3378
title: "TEXT: General inside a date or time section prints the half-second rounding offset (TEXT(5,\"d General\") is \"5 0.5\")"
milestone: M10
lane: functions-text
status: done
priority: P2
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
`General` in a section that also holds a date or time token formats like Excel: the date and time tokens print as
usual, and `General` prints a value Excel derives from its half-second rounding step, not the serial. For example,
`TEXT(5,"d General")` is "5 0.5" and `TEXT(45000,"d General")` is "15 0.499999803". The engine says `#VALUE!` today.

## Context
- Gap triage 2026-10-31 (`make eval` on origin/main c51755d6, excel-graph): `python3 scripts/exit_owners.py --all-off`
  finds exactly one visibly-off cell with no open owner, and `--milestone M10` finds the same cell:
  - probes/text-format-general-company (1): Sheet1!A5
  - Sheet1!A5 is `=TEXT(5,"d General")`: Excel "5 0.5", ours `#VALUE!` (TypeMismatch).
  T-1614 (done) probed this case and recorded A5 as an "unexplained Excel oddity" in
  docs/excel-semantics/number-formats.md, in the `General`-among-literals entry.
- Triage probe, 2026-10-31. These are excel-graph values from a scratch case the triager filled and then deleted.
  They are not a committed case yet, and this task commits them (see the acceptance criteria). Formula, then Excel's
  value:
  - `TEXT(x,"d General")` gives "0 0.5" at x = 0, "1 0.5" at 1, "2 0.5" at 2.5, "5 0.5" at 5 and 5.25, "7 0.5" at 7,
    "10 0.5" at 10, "31 0.5" at 31 and "9 0.5" at 100. It gives "15 0.499999803" at 45000 and at 45000.75, and
    `#VALUE!` at -5.
  - Other tokens: `TEXT(5,"dd General")` "05 0.5", `"m General"` "1 0.5", `"yyyy General"` "1900 0.5",
    `"h General"` "0 0.5", `"s General"` "0 0.5", `"General d"` "0.5 5", `"d General x"` "5 0.5 x" and
    `"d general"` "5 0.5".
  - An elapsed token is different: `TEXT(5,"[h] General")` is "120 5", so there General prints the value itself.
  - Already matched by the engine: `"d G"` "5 " (era), `"d Ge"` and `"d e"` "5 1900", `"d Gen"` to `"d Genera"`
    `#VALUE!`, `"d @"` `#VALUE!`, `"d ""General"""` "5 General", and `"d;General"` and `"General;d"` "5".
- Hypothesis. It reproduces all nine numeric points above in Python doubles. General prints
  `(fl(x + 0.5/86400) - x) * 86400`, shown in General's 11-character form (number-formats.md "TEXT(x,"General")
  uses an 11-character fixed-notation limit"). That is the half-second step Excel adds before it splits a serial into
  date and clock parts, measured back in seconds. At x = 45000 it is 0.4999998025596142, printed "0.499999803". At
  x = 10 it is 0.49999999993133315, printed "0.5". The value depends on the integer part only through the rounding of
  `x + 0.5/86400` (45000.75 gives the same digits as 45000).
- Check the hypothesis before implementing it. Probe x where the residue's General text changes. Large serials
  (2958465, 1E6) give a residue of up to about 7e-7, and small fractions (1E-10, 0.99999) give their own residues.
  Also probe 1904 mode if the case format supports it, an `AM/PM` section (`"h AM/PM General"`), `[m]` and `[s]`,
  and `General` beside a fractional-second token (`"s.0 General"`). This is evidence work, not a numeric fit. If a
  probed point breaks the formula, record the counterexample and narrow the rule. Do not fit it.
- Code: the section parser rejects `General` beside a date token (crates/rustsheet-engine/src/formatter/parser.rs,
  `TextToken::General`; complexity.rs lists `General` with the date tokens for the `@` rule). Date sections are
  formatted in formatter/format.rs, with the clock split in date_clock.rs. General's text comes from formatter/general.rs.
  Reuse `general.rs` for the residue's text and do not write a second General printer (directive 3).

## Acceptance criteria
- [ ] A probe case `probes/t3378-general-in-date-section` (excel-graph, tagged M10) holds the triage cells above and
      the probe points from Context. The same PR records every finding in number-formats.md, replacing the "Oddity,
      unimplemented" sentence with the rule and its evidence.
- [ ] probes/text-format-general-company Sheet1!A5 passes, and so does every cell of the new probe case. If a probed
      point stays unexplained after the time box (decision 0056), it is left as a known failure and named in a
      follow-up task. These known-failure lines disappear from eval/baseline/probes.txt with no new ones, apart from
      the new case's lines a follow-up owns.
- [ ] Unit tests in the formatter cover "d General" at 5 and 45000, "[h] General" and "General d".
- [ ] `make check` green; reviewers approve.

## Out of scope
- `General` beside a digit placeholder, a point or a second `General`: these stay `#VALUE!`, already probed and
  matched (T-1614).
- Locale tags and calendar prefixes in front of the section.

## Visible-parity stage
Decisions 0083 and 0114: the owned cell has the wrong type (`#VALUE!` against text), so the stage is `semantics`.
The target is every owned cell and every new probe cell giving Excel's exact text. A wrong TEXT result is visible to
every user of the format.

## Outcome
- The triage hypothesis `(fl(x + 0.5/86400) - x) * 86400` held only at whole serials. The committed probes show the
  real rule: General prints the seconds within the minute of `x + 0.5/86400`, in units of the shown second fraction.
  Beside elapsed tokens only, it prints the value's magnitude. All 44 date and clock cells match to the digit, in
  doubles, with General's 11-character printer (`general_clock_residue` in formatter/elapsed.rs). This is a structural
  rule read off the probes, not a fitted constant.
- The parser accepts one `General` in a date section. A `General` beside a lone elapsed token keeps a negative
  formattable in 1900 mode (`TEXT(-5,"[h] General")` = "-120 5").
- Probe cases: probes/t3378-general-in-date-section (64 cells) and t3378-general-in-date-section-1904 (7 cells),
  excel-graph 2026-10-31. number-formats.md has a new entry, "`General` in a date section", in place of the "Oddity,
  unimplemented" note.
- Eval delta, probes: fixed 1 (text-format-general-company A5), new-coverage failures 1, regressions 0. The two new
  cases pass 70 of 71 cells.
- Follow-up: T-3419 owns the one open cell, A59 `TEXT(5,"d [h] General")` = "5 12 0.5". An elapsed token after a
  date token counts from something else there.
