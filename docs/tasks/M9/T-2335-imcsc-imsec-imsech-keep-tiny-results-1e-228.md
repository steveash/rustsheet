---
id: T-2335
title: "IMCSC, IMSEC, IMSECH keep tiny results (1E-228..1E-246) instead of flushing to 0"
milestone: M9
lane: functions-engineering
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
IMCSC, IMSEC and IMSECH keep tiny results where ours flushes to 0, matching Excel.

## Context
Found by T-2043's sweep: `IMCSC("-724.227-562.884i")` is -6.94796703439417E-245 / -6.26263832046024E-246 (sweep E55, F55), IMSEC of it H55, I55, and `IMSECH("-524.561+5.3113i")` is 1.73046857535442E-228 / -2.53533899849465E-228 (K42, L42); ours gives 0. Needs a rule for when the denominator overflow yields 0 vs a tiny value (the existing note says IMSEC/IMCSC/IMSECH answer 0 where the denominator overflows). Numeric-kernel question, hence opus.

## Acceptance criteria
- [ ] E55, F55, H55, I55, K42, L42 of the sweep pass without regressing the overflow-to-0 cells already matched.
- [ ] `make check` green; baseline updated; reviewers approve.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Last-digit kernels (T-2044).
