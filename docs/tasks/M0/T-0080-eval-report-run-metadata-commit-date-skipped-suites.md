---
id: T-0080
title: "Eval report run metadata (commit, date, skipped suites) and v15_checkpoint Phase 1 rate on the attributed count (decision 0118)"
milestone: M0
lane: harness
status: done
priority: P1
size: S
tier: opus
depends_on: [T-0068]
spec: docs/specs/SPEC-0022-daily-progress-dashboard.md
---

## Goal
Two changes to report.json, in their own diff:
- it records the commit, date and skipped suites of the run, so the dashboard can tell a complete
  run from a stale one;
- `v15_checkpoint` and its Phase 1 line measure decision 0114's exit with decision 0118's
  attributed count. The stricter count stays, labelled as information.

## Context
- Decision 0118 (owner, 2026-10-25) and SPEC-0022, "Glue-aware attribution lives in the Rust eval
  report" and "Failure handling".
- The attributed and `excel` closeness fields come from T-0068.
- crates/rustsheet-eval/src/v15_checkpoint.rs: `V15Function::rate`, `meets_rate`, `at_rate`,
  `phase1_met` and `phase1_line`.
- main.rs prints "skipping …: not fetched" for an unfetched suite. Record that same reason.
- Determinism: `run.date` and `run.commit` vary per run. On 2026-10-25 no test or golden file
  compares a whole report.json. tools/tests/test_parity_report.py and scripts/test_exit_owners.py
  build synthetic reports, and no file under crates/rustsheet-eval/tests exists. Keep it that way.

## Acceptance criteria
- [x] report.json has `run: {commit, date, skipped_suites: [{name, reason}]}`:
  - `commit` is `git rev-parse --short=8 HEAD`, or null when git fails; a git failure is never
    an error;
  - `date` is the UTC run date (YYYY-MM-DD);
  - `skipped_suites` lists each suite under eval/suites that did not run, with its reason.
- [x] `V15Function` measures its rate over T-0068's `attributed` cells, or over `excel` below
      `EVIDENCE_FLOOR` attributed cells. `meets_rate`, `at_rate`, `phase1_met` and the Phase 1
      line follow it. `covered` still counts every Excel cell that calls the function (decision
      0094).
- [x] The stricter share (all Excel cells that call the function, with decision 0088's `visible`
      blame) stays as information fields, for example `strict_rate`, `meets_strict_rate` and a
      strict at-rate total. report.md prints them on a line labelled "information".
- [x] Unit tests cover:
  - a function that meets the rate on attributed cells but not on the strict count;
  - the fewer-than-20 fallback;
  - `skipped_suites` for an unfetched suite;
  - a null `commit`.
- [x] No test or golden file compares a whole report.json, so `run` cannot make a test flaky.
      Re-check this and say so in the Outcome.
- [x] docs/milestones.md's Phase 1 exit paragraph cites decision 0118 for how a function's cells
      are counted. The steward skill's wording is T-0076's.
- [x] The Outcome lists every function whose `meets_rate` is newly true only because of the
      attribution or the fallback, by comparing the strict and attributed counts on a fresh
      `make eval` of `main` (poi, corpus-bench and calipers fetched). It gives the at-rate totals
      under both counts.
- [ ] `make check` green, with no baseline or lock change; reviewers approve.

## Out of scope
- Changing decision 0094's checkpoint (`met`, `covered`, `visibly_off`) or any gate.
- Rendering (T-0081, T-0072).

## Outcome
- report.json gains `run` (new `crates/rustsheet-eval/src/run_meta.rs`): `commit` from
  `git rev-parse --short=8 HEAD` (null when git fails, never an error), `date` (UTC
  `YYYY-MM-DD`, from the engine's `from_excel_date`, so no new dependency), and `skipped_suites`
  with `not fetched` (main.rs's words) or `not selected` (left out by `--suite`). report.md prints a
  `Run:` line under the title. The timing section moved to run_meta.rs too, so report.rs stays
  at 988 lines.
- `V15Function` gains `attribution`, `rate_cells` and `rate_visible`: the cells decision 0118
  counts, read from `FunctionStat::counted()`, the one rule that also places the T-0068 tier
  (attributed cells, or `excel` below 20 attributed cells). `rate()`, `meets_rate`, `at_rate`,
  `phase1_met` and the Phase 1 line follow them. `covered`, `meets`, `met` and `visibly_off`
  (decision 0094) are unchanged.
- The stricter count stays as information: `strict.meets_rate` per function (a nested struct,
  to stay within clippy's 3-bool limit), `strict_rate()`, `at_strict_rate`, and a report.md line
  that starts "Information, gates nothing:". The table of functions below the bar shows both
  shares.
- One helper, `milestone_exit::meets_rate(within, scored, num)` (false with no scored cell), next
  to `RATE_NUM`/`RATE_DEN`. The generated-suite exit row, both v15 rates and `Tier::of` now use it.
- The parity-tier section no longer reads `v15_checkpoint.functions`. `RunReport::build` computes
  `tiers` (new report.json field: per tier, functions and fallbacks) from
  `Catalog::in_scope_functions()`, and report.md renders that.
- The Phase 1 line keeps its `Phase 1 (decision 0114):` prefix, so the steward skill's copy rule
  still finds it. The skill is unchanged (T-0076 owns its wording). docs/milestones.md's Phase 1
  exit paragraph and docs/architecture/eval-harness.md cite decision 0118.
- **Determinism, re-checked on this branch:** no test or golden file compares a whole
  report.json. crates/rustsheet-eval/tests does not exist. tools/tests/test_parity_report.py and
  scripts/test_exit_owners.py write synthetic reports, and no file under .github reads report.json.
- **Decision 0118 item 6 is enforced by no check.** Nothing verifies that each unattributed
  visibly-off cell has a filed task or a `DEVIATION`. Those are cells whose formula calls two or
  more distinct non-glue functions, or glue-only formulas with two or more distinct calls. The
  Phase 1 line can read `met` while such cells are untracked, so the steward has to check this by
  hand before adding the `**Phase 1:** exited` line.
- **Full run.** This was T-0068's head plus this change (`main` does not have T-0068's counts yet),
  with poi, corpus-bench and calipers fetched and `skipped_suites: []`:
  - before: `Phase 1 (decision 0114): not met (77/506 functions covered with ≥ 99.9% of cells
    within 15 digits)`;
  - after: `Phase 1 (decision 0114): not met (324/506 functions covered with ≥ 99.9% of their
    attributed cells within 15 digits, decision 0118)`;
  - at-rate totals: 324 attributed and 77 strict. The strict count is the old line, unchanged.
    Coverage is 492/506, so 14 functions cannot meet the rate under either count.
  - `meets_rate` is newly true for 249 functions, all because of the attribution
    (`attribution: sole`). None is newly true because of the fallback. The 3 fallback
    functions, RAND, NOW and TODAY (exempt, 28 to 37 Excel cells, none off), meet both counts. The 249: ABS, ACCRINTM, ACOS, ACOT, ACOTH, AGGREGATE, AMORLINC,
    AREAS, ARRAYTOTEXT, ASIN, ASINH, ATAN, ATAN2, ATANH, AVEDEV, AVERAGEA, AVERAGEIF, AVERAGEIFS,
    BESSELK, BESSELY, BIN2DEC, BIN2HEX, BIN2OCT, BINOM.INV, BITAND, BITLSHIFT, BITOR, BITRSHIFT,
    BITXOR, BYCOL, CEILING, CEILING.PRECISE, CHOOSEROWS, CLEAN, CODE, COMBIN, COMBINA, COMPLEX,
    CONCAT, CONFIDENCE, CONFIDENCE.NORM, CORREL, COSH, COTH, COUPDAYBS, COUPDAYSNC, COUPPCD, COVAR,
    CSCH, DATEDIF, DATEVALUE, DAY, DAYS, DAYS360, DB, DDB, DEC2BIN, DEC2HEX, DECIMAL, DEGREES,
    DELTA, DMIN, DOLLAR, DOLLARDE, DOLLARFR, DROP, DSTDEV, DSTDEVP, DSUM, DURATION, DVARP,
    ECMA.CEILING, EDATE, EFFECT, EOMONTH, ERF, ERFC, EVEN, EXPAND, EXPON.DIST, EXPONDIST, FACT,
    FILTER, FIND, FINDB, FISHERINV, FIXED, FLOOR, FLOOR.PRECISE, FORECAST, FORECAST.LINEAR, GAMMA,
    GAMMALN.PRECISE, GCD, GESTEP, HARMEAN, HEX2DEC, HEX2OCT, HLOOKUP, HOUR, HYPERLINK, IMARGUMENT,
    IMDIV, IMLOG10, IMSUB, INT, INTERCEPT, INTRATE, ISBLANK, ISERR, ISEVEN, ISFORMULA, ISNONTEXT,
    ISO.CEILING, ISODD, ISOWEEKNUM, ISPMT, ISTEXT, LARGE, LCM, LEN, LENB, LOG, LOG10, LOGNORM.INV,
    LOGNORMDIST, LOOKUP, MAKEARRAY, MAP, MAXA, MAXIFS, MDETERM, MID, MINA, MINIFS, MINUTE, MINVERSE,
    MIRR, MMULT, MOD, MROUND, MUNIT, N, NA, NETWORKDAYS, NETWORKDAYS.INTL, NOMINAL, NORM.DIST,
    NORMDIST, NORMSDIST, NORMSINV, NOT, OCT2BIN, OCT2DEC, OCT2HEX, ODD, ODDLPRICE, ODDLYIELD,
    PDURATION, PEARSON, PERCENTILE.EXC, PERCENTOF, PERCENTRANK.INC, PERMUT, PI, PMT, PRICEDISC,
    PRICEMAT, PROB, PRODUCT, QUARTILE, QUARTILE.EXC, QUARTILE.INC, QUOTIENT, RADIANS, RANK,
    RANK.AVG, RANK.EQ, RECEIVED, REDUCE, REGEXEXTRACT, REPLACE, ROUND, ROUNDDOWN, ROUNDUP, RSQ, SCAN,
    SEARCHB, SECH, SECOND, SERIESSUM, SHEET, SHEETS, SIGN, SINH, SLN, SLOPE, SORT, SORTBY, SQRT,
    SQRTPI, STANDARDIZE, STDEV, STDEV.S, STDEVA, STDEVP, STDEVPA, STEYX, SUMSQ, SUMX2MY2, SUMX2PY2,
    SUMXMY2, SYD, TAKE, TANH, TBILLEQ, TBILLPRICE, TBILLYIELD, TEXTAFTER, TEXTBEFORE, TEXTJOIN,
    TEXTSPLIT, TIMEVALUE, TOCOL, TOROW, TRANSPOSE, TREND, TRIMRANGE, TRUNC, TYPE, UNICODE, UNIQUE,
    VALUETOTEXT, VAR, VARA, VARP, VARPA, VSTACK, WEEKDAY, WEEKNUM, WORKDAY, WORKDAY.INTL,
    WRAPCOLS, WRAPROWS, XLOOKUP, XMATCH, YEARFRAC, YIELDDISC, YIELDMAT. All have 0 attributed cells
    visibly off, except XLOOKUP (1 of 1,527).
  - Two functions meet the strict rate but not the attributed one. COLUMN has 4 of 53,606 Excel
    cells blamed, but 4 of its 640 attributed cells are off. ROW has 6 of 9,130 blamed, but 6 of
    its 798 attributed cells are off. Their off cells are formulas where they are the only
    non-glue call, and their attributed cells are few. So 324 = 77 + 249 - 2.
  - Tiers (`tiers`, 506 functions): exact 300, ulp4 25, digits15 5, rate999 7, rate99 71,
    below99 98, unscored 0; 3 placed by the fallback. These are T-0068's numbers, unchanged.
- Gates, with CARGO_INCREMENTAL=0: fmt, clippy `--workspace --all-targets -D warnings`, and
  `cargo test --workspace` all pass. `make hygiene`, `make typos` and `make py-check` pass
  (941 tests). `rustsheet-eval check` on every gated suite gives 0 regressions, the visible
  parity count is 4,519 against a lock of 6,039, and no baseline or lock changed. py-bindings was
  not run, because this change does not touch the bindings.
