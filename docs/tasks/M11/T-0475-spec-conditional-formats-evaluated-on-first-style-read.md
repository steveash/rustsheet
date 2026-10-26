---
id: T-0475
title: "Spec: conditional formats evaluated on first style read"
milestone: M11
lane: perf
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A reviewed spec for evaluating conditional formats on the first style read instead of at load and
at every `evaluate()`: the public receivers that change, and what the change can and cannot be seen
to do.

## Context
T-0468 asked for a short spec if the public receivers change. They do:
`Model::get_extended_style_for_cell` and `UserModel::get_extended_cell_style` need `&mut self`. The
measure is corpus-bench 41978, which spends 184 s at load and 191 s per `evaluate()` in
`evaluate_conditional_formatting` (T-0430).

## Acceptance criteria
- [x] docs/specs/SPEC-0021-conditional-formats-on-first-style-read.md covers the flag, the
      receivers, every caller, when the evaluation happens and what it can see, the alternatives
      and the tests; T-0468 points at it.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- docs/specs/SPEC-0021-conditional-formats-on-first-style-read.md (accepted):
  - a `cf_stale` flag replaces the four eager calls (two load sites, the end of `evaluate()`,
    `set_theme`);
  - `Model::get_extended_style_for_cell`, `UserModel::get_extended_cell_style` and the Python
    binding's method take `&mut self` and evaluate when the flag is set;
  - the spec lists what the first read can see and do after `Model` or paused `UserModel` edits:
    the edits, formula cells it evaluates or reads as `#ERROR!`, cached values after load, and
    `evaluate()`'s start values for cycles.
- Reviews:
  - security APPROVE after the residual cost was stated and T-0476 filed (a bound with a
    DEVIATION entry);
  - quality and architecture APPROVE after the binding was listed as a caller and the flag sites
    were named;
  - Excel-fidelity APPROVE after the snapshot invariant was replaced by the accepted differences
    and their tests.
- Filed T-0476. T-0468 now depends on this spec, and must record the load-value change in
  docs/decisions/ and check eval for cycle start values.
