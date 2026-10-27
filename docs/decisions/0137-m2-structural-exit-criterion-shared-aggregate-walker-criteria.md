# 0137 — M2 structural exit criterion: shared aggregate walker, criteria matcher and duplication verdict

- **Status:** accepted (2026-10-27, T-2910).
- **Context:**
  - M2's last exit criterion is "One implementation of aggregate-argument iteration and criteria
    matching, used by every consumer. Duplication baseline reduced by ≥ 15%" (docs/milestones.md).
  - No tool measures this criterion. T-2822 (decision 0135) keeps it out of the report's M2 exit row because it
    is not a cell measure. This record is the audit the steward reads instead. It audits `main` at 11592ab5 and
    makes no code change.
  - "Consumer" here means a worksheet function that folds a variable list of arguments (ranges, arrays,
    scalars) into one aggregate, or that matches cells against a criterion or a wildcard pattern.
- **Decision:** all three parts are met. M2's structural criterion holds at 11592ab5.

  **1. Aggregate-argument iteration: met.**
  - Every variadic aggregate runs on the SPEC-0009 walker (functions/util/aggregate_args.rs,
    `walk_aggregate_args` / `for_each_aggregate_number`). Each one states its differences as a policy constant.
  - Every other walk below has its own loop or helper for a difference the code cites evidence for.
    None is a copy of the walker.
  - `rg "let mut reference_error" crates/rustsheet-engine/src/functions` finds nothing outside the walker,
    which is SPEC-0009's own check.

  | Function(s) | File | Verdict |
  |---|---|---|
  | SUM | math_and_trigonometry/sum.rs (`SUM_ARGS`) | on the walker |
  | MIN, MAX | math_and_trigonometry/mathematical.rs (`MIN_MAX_ARGS`) | on the walker |
  | PRODUCT | math_and_trigonometry/product.rs (`PRODUCT_ARGS`) | on the walker |
  | SUMSQ | mathematical.rs via `for_each_value_skip_empty` (`SKIP_EMPTY_ARGS`) | on the walker |
  | AVERAGE | statistical/count_and_average.rs (`AVERAGE_TOP_LEFT_ARGS`) | on the walker |
  | AVEDEV, MEDIAN, HARMEAN, SKEW, SKEW.P, KURT, DEVSQ, GEOMEAN, VAR.S/P, STDEV.S/P | `for_each_value` (`AVERAGE_ARGS`) in count_and_average.rs, devsq.rs, geomean.rs, variance.rs | on the walker |
  | AVERAGEA, MINA, MAXA | `for_each_value_a` (`A_ARGS`); it first reads linked-workbook cells itself | on the walker |
  | VARA, VARPA, STDEVA, STDEVPA | statistical/variance.rs (`VARA_ARGS`) | on the walker |
  | COUNT, COUNTA | statistical/count.rs (`COUNT_ARGS`, `COUNTA_ARGS`) | on the walker |
  | AND, OR, XOR | logical/and_or_xor_not.rs (`LOGICAL_ARGS`) | on the walker |
  | IMSUM, IMPRODUCT | engineering/complex/mod.rs (`COMPLEX_FOLD_POLICY`) | on the walker |
  | SUBTOTAL | subtotal.rs `subtotal_walk_areas` | differs: a position walk that skips filtered rows, hidden rows and nested SUBTOTAL cells unevaluated, with references only (T-1163, T-1182); a SPEC-0009 non-goal |
  | AGGREGATE | aggregate.rs `aggregate_collect_area` | differs: options 0-7 skip hidden rows, nested SUBTOTAL/AGGREGATE cells and errors by position, then call fn_sum, fn_large etc.; a SPEC-0009 non-goal |
  | SUMPRODUCT | math_and_trigonometry/sumproduct.rs `sumproduct_factors` | differs: element-wise pairing of equal-shaped arrays, not a fold (T-2150, SPEC-0014); a SPEC-0009 non-goal |
  | COUNTBLANK | statistical/count.rs | differs: counts blank positions of references only (T-1181, T-1199, T-1207) |
  | GCD, LCM | math_and_trigonometry/gcd_lcm.rs `gcd_lcm_impl` | differs: blanks count as 0, and `#NUM!` for a negative or > 2^53 argument is deferred behind any other error (probes/gcd-lcm-multinomial-error-order, T-0438, T-2294) |
  | MULTINOMIAL | math_and_trigonometry/multinomial.rs | differs: same deferred negative check (gen-m6/2c1a6e93a1974f48 Calc!A35) |
  | NPV | financial/mod.rs `npv_cash_flows` (on `for_each_number_in_areas`) | differs: a range or array error is the result only when no direct argument errs (probes/npv-power-edges) |
  | LARGE, SMALL, RANK.*, PERCENTILE.*, QUARTILE.*, PERCENTRANK.*, TRIMMEAN, MODE.*, FREQUENCY, F.TEST, T.TEST, Z.TEST | statistical/chisq.rs `values_from_range`, `values_from_areas`, `values_from_array`, `numbers_from_ranked_arg`; prob.rs `numeric_flat*` (built on `values_from_*`); mode_functions.rs `collect_mode_values` | differs: these read one data array, not a list of aggregate arguments. They form one shared reader family in chisq.rs, layered and not copied. Their rules are evidenced: union counting (probes/multiarea-rank), RANK's non-reference array (probes/rank-nonref-*), MODE's blank cell `#VALUE!` (T-0947). A SPEC-0009 non-goal |

  **2. Criteria matching: met.**
  - functions/criteria.rs holds the only operator-prefix parser (`criterion_operator`: `<=`, `>=`, `<>`, `<`,
    `>`, `=`) and the only `*`/`?`/`~` matcher (`WildcardPattern`, `wildcard_text_matcher`).
  - Outside `expressions/`, the only `"<>"`, `"<="` or `">="` literals in the engine are in criteria.rs.

  | Function(s) | Call site | Verdict |
  |---|---|---|
  | SUMIF, SUMIFS, COUNTIF, COUNTIFS, AVERAGEIF(S), MINIFS, MAXIFS | statistical/if_ifs.rs `build_ifs_criteria` → `build_criteria` | on criteria.rs |
  | COUNTIF(S) cache and index | statistical/ifs_cache.rs → `indexed_criterion`, `CriterionOp` | on criteria.rs |
  | D-functions | database.rs `criteria_cell_matches` → `build_criteria_with(.., CriteriaFlavor::Database)` | on criteria.rs |
  | MATCH (type 0), VLOOKUP and HLOOKUP (exact) | lookup_and_reference/lookup_table.rs `exact_match` → `WildcardPattern`, `folded_units` | on criteria.rs |
  | XMATCH and XLOOKUP (match_mode 2) | lookup_and_reference/xmatch.rs → `wildcard_text_matcher` | on criteria.rs |
  | SEARCH, SEARCHB | text/position.rs → `WildcardPattern::from_units`, `find_from` | on criteria.rs |
  | Conditional-format text rules | conditional_formatting.rs `apply_cf_text` | not a worksheet consumer: it compares literally, and whether Excel applies wildcards there is unprobed (criteria.md "Conditional-format text rules and wildcards", T-1075 item 20, desktop evidence per decision 0039) |

  **3. Duplication: met, down 68.0%.**
  - Measured with today's scripts/check_duplication.py (window 10) on worktrees of both commits.
  - M2's start is a08a9c76 (T-0004, #2), the parent of 7646134c (T-0200, #3). T-0200 was the first M2
    structural task to merge. The status log's first duplication line (9,153 →
    8,829) is T-0200's own measurement, and today's script reproduces both numbers (9,153 at a08a9c76, 8,829 at
    7646134c), so the window and the script are unchanged for this purpose.
  - At 11592ab5 (`main`, this PR's base) it measures 2,931, the committed quality/duplication.json value.
  - The reduction is (9,153 − 2,931) / 9,153 = 68.0%, more than the 15% required. The count covers the whole
    repository, so the reduction includes work outside M2's functions. The criterion asks only for the baseline
    reduction, which this is.

- **Consequences:**
  - The steward reads M2's exit from the report's M2 row (decision 0135) together with this record. The
    structural criterion needs no re-audit unless a new consumer is added outside the walker or criteria.rs.
    The reviewer agents already check for that on every PR (CLAUDE.md directive 3).
  - No follow-up tasks: the audit found no copy. Some functions in the "differs" rows could later move onto a
    walker policy, if their evidenced rules fit one (SPEC-0009 non-goals). Nothing in M2 requires it.
  - SPEC-0009's consumer table lags the code in two rows. DEVSQ and AVEDEV no longer skip empty arguments
    (T-1748), and VARA etc. rank errors (T-1696). The spec gains an as-built note for both.
