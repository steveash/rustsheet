# 0077 — T-1785 lands above its 5% per-row bench target; T-1932 owns the driver overhead

- **Status:** accepted (2026-10-19, owner approval, T-1785). It relaxes one row of SPEC-0011's
  performance table (the T-1785 row, "per-row unlifted `=DAY(A{r})` over ≥ 100k rows, within 5%").
- **Context:** T-1785 moves the date and time functions, MROUND, SQRTPI, MUNIT and RANDBETWEEN onto
  the SPEC-0011 driver with no result change in any suite. Its bench criterion is not met: on
  `cargo bench -p rustsheet --bench math_lift -- "DAY, per row"` callgrind puts `Model::evaluate` at
  2.109G instructions on main and 2.308G on the branch (+9.4%; wall-clock min about 16.4 ms vs
  19.4-26 ms on a noisy 4-core box; the full table is below), after the branch's `numbers_fn` fast path took 55 instructions
  per call off. The rest is the driver's fixed per-call cost (`lift_call_with`, `read_lift_arg_with`,
  the `dyn` kernel), which ABS and every other driver function already pay; T-1773 met its 5% when a
  per-row cell cost 42 ms, and other work has since made the cell cheaper, so the same cost is a
  larger share. Removing it means changing the shared hot path under the T-1110 stack-frame
  constraint, which T-1932 (perf, P1) was filed to do. The owner chose to land T-1785 now rather
  than hold it on T-1932 (2026-10-19).
- **Decision:** T-1785 merges with the measured overhead recorded in its Outcome. Its bench
  criterion moves to T-1932, whose acceptance already measures DAY per row against main before
  T-1785. SPEC-0011's T-1785 row says so. Nothing else is relaxed: the eval criterion (0 fixes,
  0 regressions in every suite), `make check` and the reviewers still apply in full.
- **Consequences:** measured on main 39385aff vs the T-1785 branch (2026-10-19):

  | `math_lift` case (callgrind `Model::evaluate`, 11 recalcs) | main | T-1785 | change |
  |---|---|---|---|
  | DAY, per row (100k) | 2.107G | 2.306G | +9.5% |
  | HOUR, per row | 2.697G | 2.364G | -12.3% |
  | DATE, per row | 3.135G | 3.447G | +10.0% |
  | `SUM(DAY(A1:A1000000))` | 16.77G | 18.95G | +13.0% |
  | ABS, per row (control, already on the driver) | 2.126G | 2.126G | 0.0% |

  DAY and DATE per row and lifted DAY are 9.5-13% slower until T-1932 lands; HOUR (and MINUTE and
  SECOND, the same kernel shape) is faster, since `lift_numbers` built an operand `Vec` per call;
  cells already on the driver are unchanged. T-1932's acceptance covers every slower row. A later migration that misses a bench
  target needs its own decision; this record covers T-1785 only.
