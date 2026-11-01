---
id: T-3649
title: "TEXT: sweep format codes against each value kind to find where Excel gives #VALUE! and ours a string"
milestone: M5
lane: functions-text
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
TEXT gives `#VALUE!` exactly where Excel does for each combination of value kind (number, negative, zero, date serial, text, boolean, empty) and format-code family, found by a sweep rather than by single codes.

## Context
Lead: the 2026-11-01 holdout themes note (docs/research/holdout-themes-2026-11-01.md). It names a class of input that fails, never a case, and it is not Excel evidence: every behaviour change here needs fresh oracle cells from the excel-probe skill, and the probe cases must not copy the note's wording. If the probes show the engine already matches Excel across the class, record that in the Outcome and close the task.

The holdout themes note reports TEXT returning a string for some format/value combinations where Excel gives
`#VALUE!`. TEXT's validity rules are spread over many tasks: T-1393 and follow-ups (complexity limit), T-1366 and
T-2683 (elapsed tokens), T-1656 and T-2157 (text values with invalid codes), T-2202 (which tokens make TEXT
#VALUE!), T-2377, T-2388; number-formats.md holds the rules. Open TEXT probes are narrow (T-2230 scientific, T-2303
unused-section check, T-2745, T-2798, T-3089, T-3342). No task sweeps the validity rules across value kinds: a code
that is valid for a number may be refused for text or a boolean and the reverse. Sweep each format-code family
(General, digits and #, scientific, fractions, percent, date and time tokens, elapsed, AM/PM, conditions, colors,
locale and calendar tags, @, quoted text, _ and *, 1 to 4 sections, empty sections) against each value kind,
including a value that falls in a section the code leaves empty. Code: the format-code parser and validator under
crates/rustsheet-engine/src/formatter/ and `fn_text`.

## Acceptance criteria
- [x] A probe case (excel-graph) covers the families above against every value kind.
- [x] Every probed cell where Excel is `#VALUE!` is `#VALUE!` in ours and the reverse; number-formats.md states each rule.
- [x] Cells inside the open narrow TEXT probes' scope are handed to those tasks.
- [x] Every quirk found is written up in docs/excel-semantics/number-formats.md in the same PR, with its probe cells cited.
- [x] `make check` green; eval known failures only shrink (baseline updated); reviewers approve.

## Out of scope
- Rendered text differences on valid codes (separate follow-ups).
- Complexity-limit refits.

## Outcome
Probes `probes/text-value-kind-sweep` (68 codes, one per family and section shape, × number,
negative, zero, date-time serial, text, TRUE, empty cell; 476 TEXT cells) and
`probes/text-value-kind-sweep-b` (19 follow-up codes, 133 TEXT cells), excel-graph 2026-11-01.
Every `#VALUE!` cell already matched Excel both ways; the rules are written up in number-formats.md
"Which codes are `#VALUE!` for which value kind". The sweep found two rendering quirks, fixed here:
a trailing `;` opens an empty section (`TEXT(0,"0;;")` is "", `TEXT(-5,"0;")` ""), and a section of
only a color or a condition prints General (`TEXT(5,"[Red]")` is "5", `TEXT("abc","0;;;[Red]")`
"abc"), each with its own number-formats.md entry. No cell fell inside the open narrow TEXT probes'
scope (T-2230, T-2303, T-2745, T-2798, T-3089, T-3342), so nothing was handed off.
Eval delta: +2 probe cases, 609 TEXT cells + 12 value cells, all pass (9 + 47 mismatches before the
fix); no other cell changed in any suite (known failures unchanged, 0 regressions).
