---
id: T-2886
title: "Fold F.TEST's bup and L140 dispatch into bratio once T-2506 lands"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-2506]
---

## Goal
F.TEST's both-shapes-above-1 tail (`f_test_tail.rs`) has no copy of a bratio piece: its `bup` and its L140
dispatch are bratio's (`bratio/mod.rs`), and its ln B is one definition with BETA.DIST's or a documented variant of it.

## Context
- T-2681 (#1959) ported F.TEST's tail as TOMS 708 bratio pieces in `functions/statistical/f_test_tail.rs`. Its review
  (reviewer-quality) found bpser duplicated from T-2505's `bratio/mod.rs`; #1959 now reuses `bratio::bpser_with_ln_beta`
  and passes its own ln B, but keeps its own `bup` and λ swap because T-2506 (branch
  `agent/functions-stat/T-2506-l140-bup`, live when #1959 merged) is porting `bup` and L140 into `bratio` for BETA.DIST.
- F.TEST's ln B (`f_test_tail::neg_ln_beta`) differs from `bratio::bpser_neg_ln_beta`: at shape 1/2 it reads GAMMALN(1/2)
  as gamln1(1/2) - LN(1/2) (an ulp apart stored; test `ln_beta_differs_from_beta_dist_bpser`), and above a shape of 8
  it takes the accurate ln B, which BETA.DIST's port does not have yet.
- Keep every pinned cell in `f_test_tail.rs`'s tests at its ulp count (`uv run rustsheet-fitlab f-test` before and
  after).

## Acceptance criteria
- [ ] `f_test_tail.rs` calls `bratio`'s `bup` and dispatch (or the shared λ swap) instead of its own.
- [ ] One ln B definition, or the F.TEST variant documented as a parameter of it with the evidence for each form.
- [ ] No F.TEST cell changes value (eval and `rustsheet-fitlab f-test` unchanged).
- [ ] `make check` green; reviewers approve.

## Out of scope
- New bratio branches (bgrat, bfrac, basym).
