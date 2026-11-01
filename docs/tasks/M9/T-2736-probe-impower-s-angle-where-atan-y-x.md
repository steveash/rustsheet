---
id: T-2736
title: "Probe IMPOWER's angle where atan(y/x) errs: zero or subnormal real part, overflowing y/x"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence fixes IMPOWER's angle for a base whose real part is zero or subnormal or whose y/x overflows, where Excel's ATAN2 kernel `atan(y/x)` cannot be used directly, flushes a subnormal angle, or meets a -0 imaginary part.

## Context
T-2610 made `fn_impower` (functions/engineering/complex/mod.rs) take its angle from `excel_atan2` (`atan(y/x)`,
IMARGUMENT's kernel; gen-m9/74779cff2810ccf4 A379). Where that kernel errs, ours falls back to libm `atan2` with no evidence
(docs/excel-semantics/engineering.md, "1i" when a part is not exactly 1). Probe forms such as `IMPOWER("1E-310+1i",2)`,
`IMPOWER("1E-310-1i",3)`, `IMPOWER("1E-300+1E+300i",1)`, `IMPOWER("1E-320+1E-320i",2)` and `IMPOWER("-1E-310+1i",2)`,
next to IMARGUMENT of the same bases. Also the kernel's other changes over libm `atan2`: a subnormal `atan(y/x)` is
flushed to 0 (`IMPOWER("1E20+1E-300i",1)`, ours "1E+20" since T-2610) and x < 0 with y = -0 is +π (`IMPOWER("-1-0i",1)`,
`IMPOWER("-1-0i",3)`).
IMSQRT takes the same kernel since T-2536 (decision 0188 R6), with the same libm `atan2` fallback and the +π for x < 0
and y = -0; probe its forms next to IMPOWER's: `IMSQRT("-4-0i")`, `IMSQRT("1E-310+1i")`, `IMSQRT("-1E-310-1i")`,
`IMSQRT("1E-300+1E+300i")`. Also `IMTAN("5-0i")`: since T-2774 a -0 imaginary part counts as real and takes TAN's
kernel, unprobed.

## Acceptance criteria
- [ ] A probe case under eval/suites/probes with oracle values covers those bases.
- [ ] engineering.md records the result; an implementation task is filed with `make new-task` if ours differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing a fix (follow-up task).
