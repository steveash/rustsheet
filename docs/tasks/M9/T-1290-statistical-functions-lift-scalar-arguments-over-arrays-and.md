---
id: T-1290
title: "Statistical functions whose arguments are all scalar lift over arrays and ranges (probes/stat-argument-modes)"
milestone: M9
lane: functions-stat
status: done
priority: P1
size: M
tier: opus
depends_on: [T-1770, T-1800, T-1801]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
Every scalar argument of the M9 statistical functions lifts elementwise over an array and over a multi-cell range, as in Excel.

## Context
SPEC-0011 rollout step 2 (docs/specs/SPEC-0011-scalar-argument-lifting.md). Enter the functions whose every argument is scalar in `Function::arg_kinds` as `Lift`: the distributions, CONFIDENCE*, STANDARDIZE, FISHER*, PERMUT*, GAUSS, PHI and the rest. Then turn each one into a kernel over values and delete its old static-analysis lines. The scalar arguments next to a data array (LARGE k, PERCENTILE k, RANK number, TRIMMEAN, FORECAST, PROB, Z.TEST, T.TEST) are T-1772's. The general shape, error and range rules are in docs/excel-semantics/arrays.md "Scalar arguments lift: shapes, element errors, and the Analysis ToolPak range rule" (probes/scalar-lift-shapes H15, H16, H20, H21, H34, H35, H38, H40, J8, K8).

probes/stat-argument-modes (excel-graph, 2026-10-09, T-1080; legend in docs/excel-semantics/statistics.md "Statistical function argument modes"): all 302 lift cells are 2 (210 `ROWS(F(..{v;v}..))`, 92 `ROWS(F(..G1:G2..))`) in Excel (distribution
parameters, LARGE/SMALL k, PERCENTILE/QUARTILE k, PERCENTRANK and Z.TEST x, RANK number, T.TEST tails/type, TRIMMEAN
percent, FORECAST x, PROB lower_limit, the FORECAST.ETS target_date and statistic_type). The engine fails all 302: `#N/IMPL!`
for an array (207 cells) and `#VALUE!` for a range (89 cells), plus 6 FORECAST.ETS and BETA.DIST/Z.TEST cells. Reuse the lifting the math and date functions already use (search
before adding a helper); BETA.DIST, Z.TEST and ZTEST are `#VALUE!` even for an array. Split by family if larger than M (T-1772 is the first split).

More cells (T-1289's gen-m9 suite, excel-graph, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/55fc8c4ffbd7c7cc Calc!A5 `=ROUND(SWITCH(..),BETA.DIST(SUMIFS(..),MUNIT(4),..))`: Excel `#SPILL!` (BETA.DIST lifts over MUNIT(4) although its x is `#VALUE!`), engine `#VALUE!`.
- gen-m9/d5bd4272655c5211 Calc!A6 `=AVERAGE(NORMDIST(PRICEDISC(38107,43861,0.5,110),VSTACK("TRUE",..),HLOOKUP(..),AND(..)),0.5,Data!E8:E9)`: Excel `#VALUE!`, engine `#N/IMPL!` (NORMDIST over the VSTACK array).
- gen-m9/1466c66c61189a83 Calc!A1 `=EXPONDIST(INDEX(Data!A1:C3,3,-1e-15),..)`: Excel 0.027981694209681673 (EXPONDIST over the row A3:C3, anchor value), engine `#VALUE!`; also needs T-1407 (INDEX with -1E-15).

## Acceptance criteria
- [x] (subset; the rest moved to T-1848) The lift cells of probes/stat-argument-modes for the functions in scope pass (their baseline lines removed), as do probes/scalar-lift-shapes H15, H16, H20, H21, H34, H35, H38, H40, J8 and K8. Ulp misses in values are named in the Outcome with their owner task.
- [x] gen-m9/55fc8c4ffbd7c7cc A5 and d5bd4272655c5211 A6 pass, and 1466c66c61189a83 A1 once T-1407 has landed, with no new failing gen-m9 cell and no new known failures in eval/baseline/.
- [x] `make check` green; reviewers approve. (`make check` green locally; review is the PR's.)
- [x] The entries it adds make the variant live, and it removes the `#[allow(dead_code)]` on `KernelWork::Weighted` and, if still there, on `ArgKind::Lift` (functions/lift/kinds.rs) (T-1770 left the allow with a `TODO(T-1290)`).

## Out of scope
-

## Outcome
44 statistical functions whose every argument is scalar run on the SPEC-0011 driver, every argument `Lift`:
BETA.DIST, BETADIST, BINOM.DIST, BINOMDIST, BINOM.DIST.RANGE, BINOM.INV, CRITBINOM, NEGBINOM.DIST, NEGBINOMDIST,
CONFIDENCE, CONFIDENCE.NORM, EXPON.DIST, EXPONDIST, F.DIST, F.DIST.RT, FDIST, FISHER, FISHERINV, GAMMA, GAMMALN,
GAMMALN.PRECISE, GAUSS, PHI, LOGNORM.DIST, LOGNORMDIST, LOGNORM.INV, LOGINV, NORM.DIST, NORMDIST, NORM.INV, NORMINV,
NORM.S.DIST, NORMSDIST, NORM.S.INV, NORMSINV, PERMUT, PERMUTATIONA, STANDARDIZE, T.DIST, T.DIST.2T, T.DIST.RT, TDIST,
WEIBULL and WEIBULL.DIST.
- **Work:** the kernels are closed form or call statrs' incomplete beta (140 steps at most) a fixed number of times:
  `Constant`. BINOM.INV/CRITBINOM are `Weighted(64)` (their search halves a `u64`). PERMUT is `Weighted(171)`: its
  product now stops once it is infinite (at most 171 factors), where `PERMUT(1E15,1E15)` looped 10^15 times; the value
  is unchanged. DEVIATION: a lifted PERMUT above 784,890 elements is `#ERROR!` (statistics.md).
- **New probe** probes/stat-later-argument-lift (141 formula cells, excel-graph 2026-10-17; value cells rounded to 12
  places): the last and the optional arguments of each function over a range on the formula's own rows (all lift),
  BETA.DIST's/BETADIST's `[A]`/`[B]` and BINOM.DIST.RANGE's `[number_s2]` over an array, and element values. 138 pass.
  The three that fail are a finding, not a regression: a typed TRUE element reads as 1 in BETA.DIST's and F.DIST's x
  (L1, L13: T-1849) and STANDARDIZE's x (L14: T-1755), where the kernels keep their scalar `#VALUE!`.
- **Code:** the kernels borrow their values and build their own errors with empty messages; the legacy names
  (BETADIST, NORMSDIST, LOGNORMDIST, NEGBINOMDIST, TDIST) share their modern kernel instead of rebuilding argument
  nodes, and each kernel keeps its scalar read and check order (T.DIST.2T reads both arguments before it checks x;
  TDIST checks x before tails). `number_or_error` moved from financial/mod.rs to functions/lift. Their fallback
  arms in static_analysis.rs are deleted (87 lines; the lock goes 1385 → 1208, main being 1295); duplicated lines
  4812 → 4510 (decision 0010 line).
- **Scalar change:** BETA.DIST's `cumulative` given as a one-cell range (`BETA.DIST(0.5,2,3,G1:G1)`) was `#VALUE!` and
  is its value, as Excel (stat-later-argument-lift L11). A 1x1 array argument now gives a 1x1 result, as for the other
  table entries (T-1813).
- **Not entered, filed:** BETA.INV, BETAINV, F.INV, F.INV.RT, FINV, T.INV, T.INV.2T, TINV, CONFIDENCE.T, the CHISQ.*
  family, CHIDIST, CHIINV, GAMMA.DIST, GAMMADIST, GAMMA.INV, GAMMAINV, POISSON, POISSON.DIST, HYPGEOM.DIST and
  HYPGEOMDIST: their statrs quantile searches, incomplete-gamma series and hypergeometric cdf sum loop by value with no
  per-call cap (`Gamma::new(a,1).cdf(a)` 1.6 ms at a = 1E10 and growing as sqrt(a); a hypergeometric cdf of 5E7 terms
  4 s), so T-1848 bounds them first. Z.TEST, T.TEST, PROB, RANK and FORECAST stay T-1772's/T-1813's.
  T-1750 (STANDARDIZE over a range) is done by this task.
- **Eval:** `update-baseline --timeout 240`: 0 regressions; probes known failures 4470 → 4302 on main at b167ec8e (171 F lines removed:
  stat-argument-modes 161, scalar-lift-shapes 8 (H15, H16, H20, H21, H34, H35, J8, K8), scalar-lift-open-a W33,
  stats-evidence-floor-t1623 Q63; 3 added, the new case's L1, L13, L14). Every suite, the non-gated ones too, was
  compared cell by cell against main (both binaries, same suites): 0 new failing cells; fixed also gen-m9
  55fc8c4ffbd7c7cc A5, d5bd4272655c5211 A6 and 95efb37a9874f82e A19, and probes-numeric/t1049-phi A37.
  gen-m9/1466c66c61189a83 A1 already passed on main. Changed failing values in the existing cases: scalar-lift-shapes H38 and
  H40, from `#N/IMPL!` to NORM.S.DIST's known ulp miss (0.8413447460685429, as the scalar H39; T-0901, T-1374). The new
  case scores 37/141 with main's binary and 138/141 here; L1, L13 and L14 change from `#N/IMPL!` to `#VALUE!`.

