---
id: T-2668
title: "Probe IMCSCH tiny results and subnormal IM* parts past the denominator overflow"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for two rules T-2335 left inferred: whether IMCSCH keeps normal tiny parts past its denominator overflow as IMSECH does, and whether a subnormal part of other IM* results prints as 0.

## Context
T-2335 made IMSEC, IMCSC and IMSECH keep normal tiny parts (trig parts times `2e^-|v|`) where the
denominator overflows (probes-numeric im-last-digit-sweep E55:I55, K42:L42), and flushed every
subnormal IM* part to 0 in the complex number-to-text (`Complex`'s Display in
functions/engineering/complex/mod.rs; evidence: IMEXP, gen-m9/b5aef3645cec1b95 A154). IMCSCH still
answers 0 (`reciprocal_result` in complex/trig.rs) and the flush is UNVERIFIED for other functions
(engineering.md "Tiny results" and the IM* last-digit sweep entry). Use the excel-probe skill.

## Acceptance criteria
- [ ] A probe case with Excel values for `IMCSCH("-524.561+5.3113i")`, `IMCSCH("400+1i")`,
      `IMCSCH("-724.227-562.884i")` and subnormal-part cases of other functions (e.g.
      `IMDIV("1E-300+1i","1E+10")`, `IMPRODUCT("1E-200+1i","1E-120")`, `COMPLEX(1,1E-310)`).
- [ ] engineering.md records the rules as VERIFIED (or corrected); implementation follow-ups filed if ours differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing a changed IMCSCH kernel (file a follow-up).
