---
id: T-2830
title: "F.TEST: the 12 cells 1-3 ulp off after bratio's port and its unported branches (T-2681)"
milestone: M9
lane: research
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-2681]
---

## Goal
The F.TEST cells T-2681 moved from bit-exact to 1 to 3 ulp off are bit-exact again, and bratio's
F.TEST branches that T-2681 left on the remainder model are scored and ported where they win.

## Context
T-2681 (decision 0083's ulp allowance) moved these 12 cells from bit-exact to within 3 ulp, none
visibly off: gen-m9/2801c1c11c21e134 A2 (1 ulp), 285d87cea1aaa73c A280 (3), 320d51765af9b6b7
A223 (1), 85f48d5cdfb24555 A154 (2), 8d3588d8df0812df A243 (1), a61e547a1cab994c A11 (1),
cde551ea787f713a A199 (1), fd9fa22d212080d6 A279 (1), probes/ftest-decomposition C7, C13, F7,
F13 (1 each; the same input). Seven are bratio's `bpser` (five deep in the tail, x0 below 1E-13),
five are L140 (the four ftest-decomposition cells at a0 = 2, b0 = 4.5, x0 = 0.284, and A243 at
a0 = 4, b0 = 2) (docs/research/t2681/README.md).
Unported in f_test_tail.rs: TOMS `gsumln` (the density at a shape 1 against one of at most 2,
which keeps the model on 6 L140 cells), and `w1_bpser` for a shape of 1/2 with x0 >= 0.3 (bratio
7 bit-exact of 8 there against the model's 2, `score_bratio.out`).

## Acceptance criteria
- [ ] Each of the 12 cells bit-exact or its residual explained in docs/excel-semantics/statistics.md.
- [ ] `gsumln` and `w1_bpser` scored against the model on the F.TEST sweep, ported if better.
- [ ] `make check` green; reviewers approve.

## Out of scope
- gen-m9/b6593d7dd13f2dac A306 (T-2829).
