---
id: T-3651
title: "Base conversions: probe and match digit-count, two's-complement range, places and coerced-argument validation across all 12"
milestone: M9
lane: functions-engineering
status: done
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
The twelve base conversions (BIN2DEC, BIN2HEX, BIN2OCT, DEC2BIN, DEC2HEX, DEC2OCT, HEX2BIN, HEX2DEC, HEX2OCT, OCT2BIN, OCT2DEC, OCT2HEX) give `#NUM!` exactly where Excel does at every digit-count, sign and places boundary and for coerced arguments, with one shared validation path.

## Context
Lead: the 2026-11-01 holdout themes note (docs/research/holdout-themes-2026-11-01.md). It names a class of input that fails, never a case, and it is not Excel evidence: every behaviour change here needs fresh oracle cells from the excel-probe skill, and the probe cases must not copy the note's wording. If the probes show the engine already matches Excel across the class, record that in the Outcome and close the task.

The holdout themes note reports Excel giving `#NUM!` where ours returns a value, at input-length and sign
boundaries and for text-number or coerced arguments, at least in BIN2*. Done: T-1413 and engineering.md "Base
conversions read a blank cell and empty text as 0", "DEC2BIN, DEC2HEX and DEC2OCT reject empty text", T-1816 (error
order), T-1991 (one places-coercion helper), T-2420 (11-digit OCT/HEX text, booleans), T-2544 (non-binary text is
#NUM!). Open: T-3124 (empty text, errors, zero kinds), T-3498 (argument-pair cells). Not swept together: 10-digit
inputs with the sign bit set and clear, 11-digit inputs, leading zeros that make a valid number longer than 10
digits, the two's-complement limits of each target (DEC2BIN -512..511, DEC2OCT and DEC2HEX limits, HEX2BIN and
OCT2BIN of negatives outside the BIN range), places too small for the result, places 0, fractional, negative, over 10,
places given for a negative result, a number as a numeric value (BIN2DEC(101) versus "101"), numeric text with
spaces, an exponent ("1E3") or a decimal point, fractional and negative DEC2* inputs, and booleans. Code:
functions/engineering/number_basis.rs.

## Acceptance criteria
- [x] A probe case (excel-graph) covers the boundaries above through all twelve functions.
- [x] The engine matches every probed cell through one shared validation path; engineering.md states the rules.
- [x] Kinds T-3124 owns are left to it.
- [x] Every quirk found is written up in docs/excel-semantics/engineering.md in the same PR, with its probe cells cited.
- [x] `make check` green; eval known failures only shrink (baseline updated); reviewers approve.

## Out of scope
- BASE and DECIMAL (math).

## Outcome
New probe probes/base-conv-boundaries (422 cells, excel-graph, 2026-11-01) covers every boundary in the Context
through all twelve functions; three date-text cells ("1/2", whose serial moves with the year) were left out.
`number_basis.rs` now has one path for all twelve: `signed_base_value` reads a BIN/OCT/HEX number as at most ten
radix digits (a number as its displayed text) and sign-extends it, `write_base` range-checks against the target
radix and applies `places`, and `number_and_places` is unchanged. The probe showed ours was wrong for BIN2* of numeric
text that is not plain binary digits (`" 101"`, `"+101"`, `"1E2"`, `"1.0"`, `"101.0"`, 11 characters with leading
zeros: values, Excel `#NUM!`), OCT2BIN of 11 characters (not length-checked) and a leading `+` in HEX2*/OCT2*
(`from_str_radix` accepted it). DEC2* (truncate toward zero, numeric text read as a number) and places already matched.
engineering.md "Base conversions: digit text, two's-complement ranges and places (T-3651)" states the rules.
Eval delta: probes +1 case, 422/422 cells pass (baseline records the new case); every other suite unchanged, 0 regressions,
0 fixed elsewhere (no existing gen/probe cell hit the mismatches). Kinds T-3124 owns (empty text, errors, zero) untouched.
