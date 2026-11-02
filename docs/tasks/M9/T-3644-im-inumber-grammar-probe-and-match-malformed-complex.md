---
id: T-3644
title: "IM* inumber grammar: probe and match malformed complex text (signs, exponents, suffix, spacing, repeated parts)"
milestone: M9
lane: functions-engineering
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
IM* functions read, or refuse with Excel's error, every malformed complex-number text the same way Excel does, across the grammar: signs, exponents, the i/j suffix, spacing and repeated or missing parts.

## Context
Lead: the 2026-11-01 holdout themes note (docs/research/holdout-themes-2026-11-01.md). It names a class of input that fails, never a case, and it is not Excel evidence: every behaviour change here needs fresh oracle cells from the excel-probe skill, and the probe cases must not copy the note's wording. If the probes show the engine already matches Excel across the class, record that in the Outcome and close the task.

The holdout themes note points at how Excel validates and parses complex strings. engineering.md "How an inumber is
read" lists the probed shapes: T-1766 (empty text, mixed i/j, booleans), T-2420 (leading plus), T-2541/T-2641
(dates, currency prefixes, edge whitespace), open T-3059 (NBSP/LF/CR after spaces, "1E$3", "12$3i"). Not swept as a
grammar: "++1", "+-1", "1+-2i", "1--2i", "i", "-i", "+i", "2ii", "i2", "1+2i3", "1e5i", "1E+5+1E-5i", "1e", "1e+",
".5i", "5.i", "1.2.3", "1 +2i", "1+ 2i", inner tabs, uppercase "I"/"J", "1+2I", full-width digits, leading zeros,
"0x1", "1,000+2i", "1+2i " with trailing space, "1i+2", two real parts, a percent part ("50%+2i"), and the same
shapes as a number-typed cell value. Code: `complex_of` and the parser in functions/engineering/complex/parse.rs
(and reading.rs).
Probe the shapes through IMREAL, IMAGINARY and one function that reformats (IMSUM of one argument) so both the
parse and the printed result are covered.

## Acceptance criteria
- [ ] A probe case (excel-graph) covers the shapes above through IMREAL, IMAGINARY and IMSUM.
- [ ] The engine matches every probed cell; engineering.md "How an inumber is read" lists each shape with its result.
- [ ] Shapes T-3059 owns are left to it.
- [ ] Every quirk found is written up in docs/excel-semantics/engineering.md in the same PR, with its probe cells cited.
- [ ] `make check` green; eval known failures only shrink (baseline updated); reviewers approve.

## Out of scope
- T-3059's whitespace and `$` shapes.

## Outcome
Four excel-graph probe cases (2026-11-02):
- probes/eng-inumber-grammar: 65 inputs.
- probes/eng-inumber-grammar-parts: 54 inputs.
- probes/eng-inumber-grammar-digits: 6 inputs with 16 or more digits.
- probes/eng-inumber-grammar-value: VALUE, `+0` and IMREAL side by side. Tagged M2.

The inputs go through IMREAL, IMAGINARY and IMSUM.

What they show: Excel splits inumber text at the first inner `+`/`-` that is not an exponent's sign, and reads each part as VALUE reads it. That covers sign runs, commas, percent, parentheses, dates, clock text and 15-digit truncation.

What changed: `split_inumber` (complex/reading.rs) replaces the lexer-based `parse_complex_number` and `has_inner_minus`. The tab/NBSP/LF/CR edge rule now applies to each part.

Eval delta: 0 fixed, 0 regressions. The grammar case passes 260/260 and the digits case 30/30. 24 new-case cells are baselined: eng-inumber-grammar-parts B18:D18, B20:D20, B32:D32 and eng-inumber-grammar-value B1:D3, B8:D9. They are VALUE's own number grammar ("1,0000", "%1", "1,"), not the IM* reader. They are recorded in coercion.md "Group commas and a leading percent sign" and handed to follow-up T-3811. So the "matches every probed cell" criterion is met except for those VALUE cells.

engineering.md "How an inumber is read" lists every shape with its result. T-3059's whitespace and `$` shapes are untouched.

