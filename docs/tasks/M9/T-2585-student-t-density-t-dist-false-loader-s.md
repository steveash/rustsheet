---
id: T-2585
title: "Student t density T.DIST(.,FALSE): Loader's dt for df >= 3 and the df 1 and df 2 closed forms (T-2513)"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: M
tier: opus
campaign: visible
depends_on: [T-2645, T-2507]
---

## Goal
T.DIST(t,n,FALSE) follows Excel's structure: Loader's saddle-point `dt` in Excel's x87 pieces for df >= 3, and Excel's
closed forms at df 1 and df 2. The T density's visibly-off cells drop to the few that the unidentified per-df constant
leaves.

## Context
- The rule is docs/excel-semantics/statistics.md "Student t density is Loader's dt (T-2513)". In short, every op an x87
  op stored to a double:
  - xx = fl(t·t), q = fl(xx/n), w = fl(1 + q);
  - xx > n: u = fl(fl(LN(w)·n)/2); otherwise u = fl(-bd0(n/2, fl(fl(n + xx)/2)) + fl(xx/2)), with the beta density's
    Excel bd0 (cut 0.2, log form, series with v stored; T-2507 owns it, so share it, do not copy it);
  - E = fl(C - u), and T = fl(fl(EXP(E)/fl(sqrt(w)))·fl(1/sqrt(2π)));
  - C = the exact Loader constant -bd0(n/2, (n+1)/2) + stirlerr((n+1)/2) - stirlerr(n/2), with accurate stirlerr
    (the same one T-2507 needs). Excel's constant is ±1-2 units off it per df, unidentified (T-2586);
  - df 1: fl(1/fl(π·fl(1 + fl(t·t))));  df 2: EXP(fl(-1.5·LN(fl(2 + fl(t·t))))).
- Models: docs/research/t2464/r3-tdensity `tdlib.py` (`lt1`, `loader_dt`; use the exact constant, not `lt1e_offsets`,
  which the verifier refuted) and `s21_small.py`; verifier rescoring in verify-r3-tdensity (`vrun.py`, `vlib.py`).
- Scores, exact / 4 ulp / 15 digits, exact constant (verifier):
  - hold-out t2513-r3-tdensity-hold2 (960): 573/960/960, against CR 266/776/839;
  - verifier hold-out t2513-verify-r3-tdensity-hold (1,000, df 13-50): 505/918/977, against CR 93/555/787 (23
    visibly off against 213);
  - docs/research/t2464/cells.tsv, df >= 3 (18,699): 5,956/18,383/18,631, against CR 3,788/18,075/18,472; df 1 and 2
    (1,077): 1,077/1,077/1,077, against CR 470/1,074/1,075.
  - Per-df constants fitted on the in-sample probes (df 3, 4, 5, 7, 10, 30) raise cells.tsv df >= 3 to
    11,517/18,684/18,697. They are measured offsets, not a rule; a table of them is allowed only with that evidence
    cited, and the acceptance below does not need it.
- Known misses: `T.DIST(±10,1E6,FALSE)` -46 ulp (bd0 series at k = 5E5) and `T.DIST(8,50,FALSE)` (t0969-t-beta-link
  J152).
- Code: `functions/statistical/student_t.rs` `student_t_density` (today double-double cos^(n+1) with a gamma ratio, a
  log1p form from `SERIES_MAX_DF`), called from `t_dist.rs`. Excel's EXP (`x87_exp::excel_exp`) and LN
  (`x87_log10::excel_ln`) exist. Code must not read docs/ (decision 0066): copy the rows the tests need into a fixture.
- T-0969 is blocked on its tails and B; this task takes its density part (T-0969 "Blocked because").

Owned gen-m9 cells (handed from T-2437, excel-graph, 2026-10-28): 1 known-failure line in eval/baseline/gen-m9.txt, 1 formula.
Handed from T-2437 because the outer EXPON.DIST is bit-exact on Excel's inner T.DIST(4199,4,FALSE) = 9.192880751152396E-18
(excel-graph, 2026-10-28, probed by T-2437; ours 9.192880751152399E-18): the cdf at that tiny t is 0.3 * the density.
Examples:
- gen-m9/caa1eed48b1883fe Calc!A382 `=EXPON.DIST(T.DIST(DAYS(14246,10047),4,Data!D$11),Data!A9,Data!B$1)`: Excel 2.757864225345719e-18, ours 2.7578642253457196e-18
Cells:
- gen-m9/caa1eed48b1883fe.case.json (1): Calc!A382

## Acceptance criteria
- [ ] On cells.tsv's 19,776 T.DIST(.,FALSE) cells the engine is within 15 digits on at least 19,708 (df >= 3 at least
      18,631 of 18,699, df 1 and 2 all 1,077) and bit-exact on at least 7,033.
- [ ] On t2513-r3-tdensity-hold2 at least 573/960/960 and on t2513-verify-r3-tdensity-hold at least 505 bit-exact and
      977 within 15 digits (unit test over a committed fixture).
- [ ] df 1 and df 2: bit-exact on all 1,077 cells.tsv cells and on verify-r3-tdensity's 27 + 1,050.
- [ ] The Outcome reports the visible change per suite against today's engine; no gated known-failure line is added.
- [ ] statistics.md's entry gains an "Implemented in" line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The per-df constant's residual and the bd0 series arithmetic (T-2586).
- The Student t tails (T-2509) and T-0969's B prefactor.

## Visible-parity stage
Decision 0083: this task is done when every T.DIST(.,FALSE) cell it owns is within half a unit of the 15th significant
digit of Excel's value, or the cells still off are named in the Outcome and handed to T-2586. With the exact constant
the model leaves 68 of cells.tsv's 19,776 visibly off (CR 229) and 23 of the verifier's 1,000. Bit-level criteria
still unmet then move to a follow-up task without the campaign key.
