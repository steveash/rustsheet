---
id: T-3327
title: "bfrac port tidy-ups from T-2508's review: model's non-converged result, mpmath reference script, hand-off cell lists, NEGBINOM walk-first re-check"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-2508]
---

## Goal
The minor findings of T-2508's review panel (PR #2486, all APPROVE) are closed, so the bfrac port matches its model on
every path and its hand-offs are tracked by the tasks that receive them.

## Context
- `crates/rustsheet-engine/src/functions/statistical/bratio/bfrac.rs`: after `BFRAC_MAX_TERMS` (1,000) terms the port
  returns `None` (today's kernel), while the model (`docs/research/t2464/r2-bratio/bratio_excel.py` `bfrac`) and TOMS
  708 return `brc * r` anyway. The doc comment calls the bound "directive 5", but the loop is bounded either way.
- statistics.md "Incomplete beta L140 and bfrac" cites a 50-digit mpmath reference (Excel's bfrac error up to 82 ulp
  on the t2220 F sweep; F.DIST(0.1,50,100,TRUE) Excel +26.4, port +24.4, Cephes -9.6 ulp) with no committed script.
- T-2508's Outcome names 19 owned cells still visibly off (12 newly off) and hands them to T-2512 and T-3244, but
  neither task file lists them.
- `binom.rs` `negbinom_dist_of`: NEGBINOM.DIST's cdf runs its pmf walk before bfrac (`takes_bfrac`, `walk_first`) on
  the strength of one cell (ironcalc-calc BINOM.xlsx NEGBINOM.DIST!E39); the second `ported_tails` call site after the
  walk has no pointer back to the first.
- Cosmetic: `bratio/mod.rs`'s module doc line 4 runs past the paragraph's wrap.

## Acceptance criteria
- [ ] bfrac returns `brc * r` after its last term as the model does (the doc comment no longer cites directive 5), or
      the comment records why the fallback is kept; the bratio unit tests and `make eval-check` show no change.
- [ ] A script under `docs/research/t2508/` recomputes the statistics.md ulp figures from the case files (mpmath
      `betainc` at 50 digits), and statistics.md cites it.
- [ ] T-2512's task file lists the 19 cells from T-2508's Outcome as its fraction-noise item.
- [ ] `binom.rs` carries `TODO(T-3323)` at the walk-first rule (re-check it when T-3323 scores bfrac) and a one-line
      pointer at the second `ported_tails` call; T-3323 names the re-check in its Context.
- [ ] `bratio/mod.rs`'s module doc is re-wrapped.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any change to bfrac's arithmetic or results beyond the non-converged path (T-2512, T-3323).
