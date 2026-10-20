---
id: T-0061
title: "Fifteen-digit closeness bucket in the eval report and parity page, with a ratchet on visibly-off cells"
milestone: M0
lane: harness
status: done
priority: P1
size: M
tier: opus
campaign: visible
depends_on: []
---

## Goal
The eval report and the parity page show how many cells and functions are visibly off from
Excel: numbers more than half a unit in the 15th significant digit away, and wrong types, errors
or text. A gate keeps that count from growing. The visible-parity campaign (decision 0083) needs
this to measure progress and to say when it is done.

## Context
- `crates/rustsheet-eval/src/closeness.rs` (T-2036) has five buckets: pass, ulp1, ulp4, close
  (within 1e-9 relative) and off.
  - "close" hides numbers right to only 10 digits.
  - "off" mixes numbers with 2 correct digits and cells of the wrong type or text.
- `tools/src/rustsheet_tools/parity_report.py` renders these buckets on docs/parity.md (decision
  0079).
- Measured by hand on the 2026-10-20 run (scripts in the T-0060 session):
  - visibly-off numeric cells: 4,613 (13–14 digits right 2,458; 10–12 digits 962; 6–9 digits
    516; fewer than 6, or 0 against nonzero, 677);
  - non-numeric mismatches: 1,459;
  - cells within half a unit that still print differently at 15 digits: 612. They sit on a
    rounding boundary, so they count as visible parity but not ulp parity.
  - Attribution: blame the specialised function in a formula, not glue such as IF, SUM, DATE or
    INDEX. Otherwise IF and SUM look visibly off because of what they wrap.

## Acceptance criteria
- [x] `Closeness` splits the current `close` and `off` buckets:
  - `digits15`: within half a unit in the 15th significant digit of the expected value, more than
    4 ulp apart;
  - `visible_numeric`: numbers further than that, and 0 against nonzero;
  - `visible_other`: wrong type, error, text or boolean.

  The definitions are documented, with unit tests on boundary values: exactly half a unit, an
  expected 0, and subnormals.
- [x] report.json carries the new counts per function and per suite, and report.md lists the
      functions with visibly-off cells, worst first.
- [x] The parity page shows a "Visibly off" column for functions and cells. The landing summary
      states how many in-scope functions are visibly off.
- [x] A ratchet file (for example `quality/visible-parity.json`, protected; decision record)
      holds the in-scope visibly-off cell count. `make check` fails when the count grows. The
      ratchet updates through `make eval-baseline`.
- [x] The run of this PR's `main` reproduces the hand counts above within the attribution
      difference, explained in the Outcome.
- [x] `make check` green; reviewers approve.

## Out of scope
- Fixing any function.
- Changing which cells pass or fail the existing gate.

## Visible-parity stage
This task is the campaign's measuring stick. After it lands, run the gap-triager so it files the
remaining visibly-off functions that no tagged task covers yet. Examples: WEIBULL.DIST,
NORM.S.DIST tails, LOGNORM.DIST, F.DIST, ODDFPRICE, AMORDEGRC, LINEST, LOGEST, GROWTH and the
IM* functions.

## Outcome
- **Buckets** (`crates/rustsheet-eval/src/closeness.rs`): `close` and `off` became `digits15`,
  `visible_numeric` and `visible_other`. `within_fifteen_digits` defines the half unit and is
  unit-tested at the boundary: exactly half a unit at 1e15 and 1e16, an expected 0, subnormals
  and non-finite values.
- **Report**: report.json carries per-function `closeness` and `visible` (the visibly-off cells
  blamed on the function rather than on glue, `closeness::GLUE`). Each suite carries `closeness`
  over its in-scope cells. report.md has a "Visibly off (fifteen digits)" section with the counts
  per suite and the blamed functions, worst first.
- **Parity page** (`parity_report.py`): six buckets, a "Visibly off" column per function and per
  suite, a "Visibly off" table, and a landing summary that says how many in-scope functions are
  visibly off. docs/parity.md is not regenerated here; the next `make parity-report` on `main`
  picks the new columns up.
- **Ratchet**: decision 0088. The lock is `eval/baseline/visible-parity.json`, not
  `quality/`. The count moves in most function PRs, and each change under `quality/` needs a
  decision record. A full `rustsheet-eval check` (`make eval-check`) holds the count to the lock;
  `update-baseline` (`make eval-baseline`) records drops, and rises that new cases account for.
  The lock is 1,825: the in-scope visibly-off cells of the vendored gated suites (probes 1,248,
  ironcalc-calc 323, gen-m5 178, gen-m10 67, fixtures 9). Merging `main` changed it twice: three
  gen-m5 fixes took it from 1,787 to 1,784, then T-2115 (#1327) added 41 visibly-off probe cells
  (SHEETS and INFO, owned by T-2181 and T-2190) and took it to 1,825. Those cases were already in
  `main`'s baseline, so `update-baseline` could not count them as new coverage. The rise was
  recorded once with `--allow-regressions` (decision 0088, "Bootstrap"); every baseline was
  unchanged, with 0 regressions. The lock read/write code shared with
  the evidence floor moved to `lock_file.rs`.
- **Hand counts**: I ran this harness on the engine and suites at 35a5e71, the commit
  decision 0083 measured. The vendored suites (no poi or corpus-bench) give 4,581 visibly-off
  numeric cells (hand count 4,613) and 1,459 wrong type, error or text cells (hand count 1,459),
  plus 867 `digits15` cells. Blamed functions: 114 with a visibly-off number (hand count 110), 134
  with a wrong type or text (131), 192 in all (186). The differences come from the glue list and
  the in-scope rule: a cell that calls an out-of-scope function is left out.
- **Current `main`**: the same vendored suites give 3,375 numeric and 1,451 other. GAMMA, GAMMALN,
  LN, BESSELJ/BESSELY and the Student-t family were fixed since 35a5e71. With poi and
  corpus-bench added, the totals are 3,412 numeric, 2,260 other and 853 `digits15`, over 266
  blamed functions.
- Eval delta: none. The baselines and gate verdicts are unchanged; only the new lock is added.

