# Status

The steward routine owns the snapshot and the critical-lane line and rewrites them daily; agents may
still add or remove "Needs owner" entries. PRs no longer append log lines (decision 0024): the record
of merged work is `git log --first-parent main` plus each task's Outcome, and the steward lists what
merged since its last run.

## Snapshot — 2026-10-22 (M5 is current)

| Suite | Cases | Cells | Bit-exact | Known failures |
|---|---|---|---|---|
| ironcalc-calc | 219 | 72,223 | 98.85% | 831 (was 890) |
| poi | 336 | 27,209 | 99.57% | 130 (was 131) |
| gen-m5 | 65 | 22,400 | 99.95% | 12 (was 165) |
| gen-m6 | 80 | 23,000 | 99.30% | 163 (newly gated) |
| gen-m7 | 38 | 10,991 | 97.86% | 870 (newly gated, T-2225) |
| gen-m2 | 30 | 12,000 | **100%** | 0 |
| gen-m1 | 15 | 3,750 | **100%** | 0 |
| gen-m10 | 20 | 1,690 | 97.22% | 47 (was 54); displayed text 1,148 (was 1,347) |
| probes | 1,206 | 77,123 | 97.40% (12 accepted deviations; 3 cases pending) | 2,020 (was 1,599; 105 new cases) |
| smoke | 6 | 46 | 100% | 0 |
| fixtures | 17 | 248 | **100%** (was 96.37%) | 0 (was 9) |
| corpus-bench | 912 | 134,348 | 99.63% | 14 |
| gen-m8a | 12 | 3,577 | 100% | not gated |
| gen-m8b | 9 | 3,368 | 91.78% | not gated |
| gen-m9 | 25 | 998 | 99.30% | not gated |
| probes-numeric | 177 | 127,085 | 73.00% (99.68% within 1e-9) | not gated |

`make eval` on `main` (b23f5f81) shows 0 regressions in every gated suite. poi and corpus-bench were fetched for this run. The first full run shared the machine with two `make check` runs (the orphan-PR updates below), and 5 corpus-bench workbooks hit the 60 s case timeout. A quiet rerun (`make eval-lower-visible-lock`) recorded no timeouts, and it lowers the visible-parity lock from 3,852 to 2,947 (decision 0093).

**Fifteen-digit parity: not met (coverage 112/506, visibly-off cells 9288)**. This is the first run that records the line, so there is no earlier line to compare (decision 0094).

CI on `main` is green at b23f5f81. No CI run on `main` failed since the last snapshot; one was cancelled by the next push.

**The nightly fuzz run failed (2026-10-22, run 39718987994).** `formula_eval` crashed on `DATEVALUE(Sheet1:Sheet1!B1:BM1)`.
- It trips the `debug_assert!` in `model.rs` (a larger-than-1x1 array reaches a scalar-context cell).
- I reproduced it on b23f5f81 in a debug build. `LEN(...)` and `ABS(Sheet1:Sheet1!B1:C1)` crash the same way, but the plain range `Sheet1!B1:BM1` does not.
- The 2026-10-21 run was green, so this is a regression from yesterday's merges.
- Filed **T-2351** (engine, P0, opus), with the reproducer.

**M2's exit is not met: M0–M2 probes (decision 0051) have 5 known failures** (442 cases, 14,026 cells, 14,017 bit-exact, 4 deviations).
- T-2228 fixed last run's 12 subnormal cells.
- The 5 failures today:
  - `probes/index-zero-index-scalar-shape` (4): owned by T-2191 (functions-lookup), which this run raised to P1;
  - `probes/legacy-if-arm-other-references` (1): owned by T-2326 (engine), whose branch is active.
- The frozen m1-probes list is still at 0.

Where M5's exit stands ("Milestone exit (decision 0085)" in the report):
- Catalog coverage: 133/135. **Not met.** INFO and SHEETS have Excel evidence but no gen-suite cells where they are the outer call. The planner filed T-2353 (research), which depends on T-0063 (#1443).
- gen-m5: 22,400 scored cells at 0.99946, 12 known failures against an allowance of 22. **Met** (165 last run).
- ironcalc-calc M5 row: 0 failing. **Met.**
- M5 probes: 214 cases, 2 known failures. **Not met.** Both are in `probes/index-reference-args`, owned by T-2137 (#1450).
- `make exit-owners MILESTONE=M5` found one unowned failing cell: gen-m5/eb942f4a59b1c004 Calc!A115, `NUMBERVALUE("25:00",…)`. The planner filed T-2352 for it. The check now reports 0 unowned cells.

Hygiene: lint-debt 91 allow entries (= last run) · duplicated lines 3,121 (was 3,147) · oversized files 11 (= last run).

Since the 2026-10-21 snapshot (steward PR #1425), 98 commits landed on `main` through b23f5f81. None was reverted.
- All 98 were squash-merged through PRs on GitHub (committer GitHub). The four titles without `(#n)` match merged PRs: T-1755 #1502, T-2220 #1498, T-2276 #1479, T-2158 #1474. No direct pushes to `main`.
- Task ids merged:
  - T-0062, T-0064, T-0439, T-0969, T-1484, T-1591, T-1592, T-1654, T-1721, T-1748, T-1753, T-1755, T-1836, T-1839, T-1878, T-1890, T-1932, T-1943, T-1960, T-1965, T-1969, T-1973, T-1996
  - T-2043, T-2064, T-2081, T-2083, T-2084, T-2101, T-2130, T-2145, T-2150–T-2153, T-2155–T-2162, T-2166, T-2167, T-2169, T-2180, T-2187, T-2190, T-2194, T-2195, T-2197–T-2200, T-2202, T-2206, T-2208
  - T-2220, T-2225, T-2228, T-2234, T-2235, T-2237, T-2240, T-2245, T-2251, T-2259, T-2263–T-2269, T-2275, T-2276, T-2278–T-2281, T-2284, T-2285, T-2298, T-2300, T-2304, T-2306–T-2308, T-2316, T-2317, T-2323, T-2324, T-2337, T-2341
  - Task-only PRs: #1385, #1426.

Blocked: 30 (29 last run).
- New: T-2325 (desktop table edits, M14, decision 0039).
- Can be unblocked: **T-1699**. Its blocker said "Unblock when T-2195 is done", and T-2195 is now done. It is blocked on its pushed branch `agent/engine/T-1699-this-row-eval-time` (draft #1368), so it must be unblocked on that branch (decision 0048). My push to that branch was refused by this session's permission policy, so it is still blocked. The opus worker can unblock it when it resumes the branch.
- The rest are unchanged: Phase 2 (T-0400, T-0401, T-0402, T-0423, T-1066, T-0441), time-boxed fits with no new hypothesis (T-0968, T-1528, T-1529, T-1592), T-1099/T-1284 (GEOMEAN), T-0431 (FUSE corpus), T-2037 (needs a routine change) and the M14 desktop-oracle tasks (decision 0039).

Open PRs: 14 at 05:56 UTC. Two agent PRs had been open longer than 24 h at the start of the run: #1230 (T-1835) and the draft #1368 (T-1699). #1413 (T-1731), #1417 (T-2117) and #1421 (T-1735) passed 24 h during the run.
- #1230 (T-1835) conflicts with `main`. Merging `main` in locally fixes 34 probe cells but **regresses 16 gen-m7 cells**:
  - 15 cells: EXPAND lifts `pad_with`. T-1960's evidence says an array pad_with is one `#VALUE!`.
  - 1 cell: WRAPCOLS with a 2-D pad_with array.
  - I pushed nothing and commented the blocker on the PR. Main's T-1835 also owns new T-1960 cells, so the PR goes back to the functions-lookup lane.
- #1413 (T-1731): another session (`01Umo3rB…`) merged `main` in and pushed at 06:49 UTC, so it is live and I left it alone.
- #1417 (T-2117) and #1421 (T-1735) conflict with `main`. This run merged `main` into both; see the PR for whether those merges were pushed. Neither PR was merged by this run: no reviewers have run on the merged diffs.
- #1368 (T-1699) is a draft whose task is blocked on its branch. Its blocker is gone (see Blocked).
- No duplicate task PRs. #1210 (T-1943) closed after #1347 merged.
- No task is `Blocked because: review-round limit`.

Queues at the start of the run:
- functions: 7 ready (≥ 3), so the gap-triager did not run. The sonnet buffer is 23 (≥ 10).
- M5 ready by lane: engine 0, harness 0, research 0, quality 5, api 2.
  - Engine and research show 0 only because their lanes are busy (active branches); they have 4–6 and 11 unclaimed todo tasks.
- The planner ran for M5 and filed:
  - T-2352 (functions-text, P1): NUMBERVALUE reads "25:" as 25 hours. It owns the one unowned exit cell. T-2151's Outcome wrongly said that cell was already fixed.
  - T-2353 (research, P1): Excel-backed gen-m5 cells with INFO and SHEETS as the outer call, for M5 coverage.
  - It filed no harness task: the only M5-relevant harness work is T-0063, which is already in flight.

Upstream IronCalc: four new commits under base/, xlsx/ or bindings/python.
- 1fd0ff4f extends the Excel-authored `LOG_LOG10_LN.xlsx`. Filed **T-2350** (functions-math, sonnet) to take it into ironcalc-calc.
- 7e271963 and 34b8bb6e bump dependencies. Upstream's statrs 0.18 → 0.19.1 bump changed a CHISQ result (its test now expects 6.999999999 instead of 7), so T-2350 says not to take statrs 0.19.
- eb9ca75b is a version bump only.
- UPSTREAM.md is triaged through 7e271963.

Oracle: excel-graph credentials are present (`rustsheet-oracle status`: excel-graph OK).
- probes: 0 written, 1 failure. `typed-if-one-argument` (T-1990) is still refused with 501 `FileCorruptTryRepair`. The other 2 pending cases are the unfillable M10 locale cases.
- ironcalc-calc: 0 written, 216 skipped (Excel-authored).
- poi's 205 pending cases are web-vs-desktop conflicts (decision 0019), so they were not refilled.

Spot-audit: the last one was 2026-10-16, so none is due (next on or after 2026-10-23).

**Critical lane:** engine, harness, research

(Ordered list read by both tier workers, which then fall back to `functions`. The current milestone is M5.
- engine: T-2351, the P0 fuzz crash (directive 5). Then T-2326 (M0–M2 probe) and T-2137 (M5 probes, #1450).
- harness: T-0063 (#1443), the generator `--functions` flag that T-2353 needs for M5 coverage.
- research: T-2353 (INFO/SHEETS coverage), then T-2252 and the queued M5 probes.
- The other M5 exit work is in the functions lanes: T-2191 (M0–M2, functions-lookup, P1) and T-2352 (functions-text). The workers reach them through `functions`.
- quality and api have no M5 exit work. perf stays out: Phase 2.)

## Needs owner

- **Direct pushes to `main`** (2026-10-18, continuing 2026-10-19). Session `01Xbmds2…` has now pushed fifteen T-1778 commits straight to `main` with no PR (6f85cc9 … 892a53ca reported last run; fc3f993d, c00142bb, 0b5ce7a4, 36217c63, 756f849f, 37d303dd, 32ac084d and c763916b since), so the reviewer agents never saw them. CI passed on each and nothing needs reverting. New this run: **32ac084d edits a protected path, `.claude/skills/excel-probe/SKILL.md`**, without the decision record the governance gate demands of a PR. It is a two-line pointer to the new `docs/research/excel-kernel-playbook.md`, and both read as benign research guidance. Still, agent instructions changed with no review. Recommended fix: a branch ruleset on `main` that requires a pull request (the steward and workers already merge through PRs, so nothing changes for them). If these pushes were your own interactive work, remove this item. Update 2026-10-20: no direct push to `main` since the last run; all 109 commits came through PRs.
- **Steward regenerates the parity page** (2026-10-19, T-2037). `make parity-report` (T-2041, decision 0079) refreshes docs/parity.md after `make eval`; running it daily needs an owner change to the steward routine and its copy in docs/process/routines.md.

excel-desktop needs are not owner items (decision 0039): T-1016, T-1051 and the other desktop tasks wait in M14 (Phase 4), tracked by T-1074 and T-1075.

## Log
- 2026-10-02 — Owner set `main` as the GitHub default branch and enabled the routines (repo attached).
- 2026-10-01 — `main` created. Routines must be created in the claude.ai UI with the repo attached (docs/process/routines.md).
- 2026-10-01 — M0 bootstrap: fork, eval harness, oracle client, generator P0, CI, agent process.
- 2026-10-02 — T-0200: shared range-number iterator; duplicated lines 9153→8829; eval 0 regressions, 0 fixed.
- 2026-10-02 — T-0002: pinned external suite fetcher (`rustsheet-eval fetch`); eval 0 regressions, 0 fixed.
- 2026-10-02 — T-0201: criteria/wildcard matching consolidated in functions/criteria.rs; fixed COUNTIF "<>text" on non-text cells; eval 0 regressions, 0 baseline fixes (+2 new spec cases pass).
- 2026-10-02 — T-0003: suite `poi` (Apache POI, fetched, pinned); 334 cases, 3,526 cells, 82.25%; baseline +633 known failures (new suite, 0 regressions); filed T-0111 (timeouts), T-0112 (imports).
- 2026-10-02 — T-0204: coercing range walk shared by VARA/STDEVA family and GCD/LCM; duplicated lines 8829→8610; eval 0 regressions, 0 fixed.
- 2026-10-02 — T-0007: pending cases now run crash-only (panic/timeout/error reported, per-suite crashes column); eval 0 regressions, 0 fixed.
- 2026-10-02 — T-0205: D-function criteria routed through shared criteria grammar (CriteriaFlavor::Database); eval 0 regressions, 0 fixed (+1 new smoke case passes).
- 2026-10-02 — T-0005: transitive volatile skipping + METADATA-pinned clock (NOW/TODAY now scored); NOW() keeps sub-second precision; ironcalc-calc +2 scored cells pass; eval 0 regressions, baseline unchanged.
- 2026-10-02 — T-0008: `rustsheet-gen --policy numeric-rules` → suite `probes-numeric` (2,738 hypothesis-tagged probe cells, pending oracle); eval 0 regressions, 0 fixed.
- 2026-10-02 — T-0202: catalog extended with 18 M2 functions + pair-repeat groups in generator; eval 0 regressions, 0 fixed (tools-only).
- 2026-10-03 — T-0902: eval snapshot reader decodes _xHHHH_ escapes (CHAR 29 cells fixed; ironcalc-calc known failures 1909 -> 1880)
- 2026-10-03 — T-0100: SPEC-0001 public API v0 (facade crate) accepted; filed T-0113; docs only, eval unchanged.
- 2026-10-03 — T-0900: BESSELJ/BESSELY bit-exact for |x|<8 via NR approximations (ironcalc-calc known failures 1880 -> 1761); |x|>=8 filed as T-0908
- 2026-10-03 — T-0113: new `rustsheet` facade crate with typed ErrorKind/XlError/RefError and single ERROR.TYPE table; eval 0 regressions, 0 fixed.
- 2026-10-03 — steward: health snapshot; filed T-0114..T-0117 (api/harness ready queues), T-0910/T-0911 (IronCalc ports); UPSTREAM.md triaged through 9b1ae3fd. Open PRs #11, #13 are blocked docs-only drafts from other sessions (left untouched).
- 2026-10-03 — T-0910 (partial): TIMEVALUE of "date time" text returns serial fraction as Excel does (ironcalc-calc known failures 1761 -> 1760); fractional seconds still need oracle evidence
- 2026-10-03 — T-0114: API v0 `refs.rs` (SheetId/CellRef/RangeRef/Target, pure A1 parse/print); eval 0 regressions, 0 fixed.
- 2026-10-03 — T-0912: MOD uses exact remainder (fmod) as Excel; ironcalc-calc known failures 1760 -> 1756 (fixed 4, 0 regressions)
- 2026-10-03 — T-0115: API v0 typed Value/Array and engine cell conversion; eval 0 regressions, 0 fixed.
- 2026-10-03 — T-0914 (partial): SINH/COSH/TANH use Excel's exp formulation (ironcalc-calc known failures 1756 -> 1731, fixed 25, 0 regressions); T-0916 (IRR/RATE/XIRR) found not fittable without the oracle; filed T-0917 for the rest
- 2026-10-03 — T-0111: VLOOKUP search column bounded by last used row; 2 POI timeout cases re-included (poi known failures 633 -> 655 from 24 newly scored cells, 0 regressions); TANH 0.9<=|x|<1 uses e^2x form (fixes main's poi G1360 regression); baseline also records ~26 crash-only POI cases (filed T-0118, T-0119)
- 2026-10-03 — T-0911 (ROW/COLUMN part): ROW/COLUMN of multi-cell references return vectors (ported ironcalc b6d34abc); ironcalc-calc 21 newly checked cells, 0 failures, 0 regressions; rest filed as T-0918
- 2026-10-03 — T-0919: @ROW/@COLUMN skip materialising whole-column vectors (165 ms -> 0.17 ms); eval delta none
- 2026-10-03 — T-0918: ported ironcalc INDIRECT (R1C1 + INDIRECT.xlsx) and text-on-arrays fixes; ironcalc-calc +311 newly checked cells with 0 failures, 0 regressions; remaining ports filed as T-0920
- 2026-10-04 — T-0920 (partial): ported POWER-accepts-ranges (daf5f6c6) and implicit-intersection export (9212095d); ironcalc-calc +51 newly checked cells (47 pass, 4 NumericUlp baselined), 0 regressions; finite-number parse and --A1:B4 remain
- 2026-10-04 — T-0118: importer no longer panics on POI workbooks (typed errors, optional style sections); poi known failures 655 -> 647, 0 regressions beyond 4 crash-kind relabels (decision 0012); duplicated lines 8610->8420.
- 2026-10-04 — T-0913: eval snapshot reader decodes rich-value `vm` errors (#CALC!/#SPILL! cached as #VALUE!); ironcalc-calc known failures 1735 -> 1727 (8 fixed), 0 regressions; engine unchanged
- 2026-10-04 — T-0101: removed lifetime from Model/UserModel/Parser/Lexer and the Python binding's leak helper; eval delta none (0 fixed, 0 regressions)
- 2026-10-04 — steward: health snapshot; queues healthy, no new upstream commits, no oracle credentials.
- 2026-10-04 — T-0923: TIME divides before dropping whole days (+7 cells, ironcalc-calc 97.60%); triaged T-0922..T-0929 (no oracle needed)
- 2026-10-04 — T-0102: API v0 `Workbook` read path + xlsx load/save with zip-bomb limits; eval adapter on the facade; eval 0 regressions, 0 fixed.
- 2026-10-04 — T-0924: shared accurate erf/erfc + normal CDF/PDF kernel (ERF*, NORM.DIST/S.DIST, LOGNORM.DIST, GAUSS, Z.TEST); ironcalc-calc known failures 1720 -> 1632 (88 fixed), 0 regressions
- 2026-10-04 — T-0103: API v0 `Workbook::edit`/`set` with undo-log rollback and diff-based `ChangeSet`; eval adapter builds specs via `edit`; eval 0 regressions, 0 fixed.
- 2026-10-04 — T-0104: Python bindings on API v0 (`rs.Workbook`, `edit()` transactions, `XlError`, typed exceptions, GIL released); eval unaffected; filed T-0931 (parser nesting stack overflow).
- 2026-10-04 — T-0928: EXPON.DIST cumulative uses -expm1; ironcalc-calc known failures 1632 -> 1626 (6 fixed), 0 regressions
- 2026-10-04 — T-0930: API edit follow-ups (diff split into edit/diff.rs, sheet-position map, error classifier next to RefError, engine take/restore tests, evaluation counter, bench moved); eval 0 regressions, 0 fixed.
- 2026-10-04 — T-0926: GAMMALN/GAMMALN.PRECISE via libm lgamma; ironcalc-calc known failures 1626 -> 1578 (48 fixed), 0 regressions; 12 cells need oracle
- 2026-10-04 — T-0004 (partial): excel-graph oracle live (app-only, `Sites.Selected` site); fixed site drive addressing; T-0004, T-0106..T-0110, T-0203 unblocked.
- 2026-10-04 — T-0932: eval wall time 72.5 s → 71.1 s across T-0930; edit bench median baseline (100k formulas): recalc 13.4 ms, tracked set 16.5 ms, diff overhead 3.0 ms; KeyHasher deduplicated; eval 0 regressions, 0 fixed.
- 2026-10-04 — T-0009: excel-graph sidecars for probes (3) and probes-numeric (4); probes 71.11% (13 new-coverage known failures), probes-numeric 60.19% (not gated); 0 regressions; excel-semantics: 9 entries VERIFIED (snapping corrected), 4 new; inputs with ≥1E+308 literals fixed (Excel rejects them). T-0106/7/8/10 now ready.
- 2026-10-04 — Owner updated the routine prompts to the text in `docs/process/routines.md` (verified identical). Decision 0014: Excel is always right; every quirk found is recorded in docs/excel-semantics/.
- 2026-10-04 — T-0013 (+ T-0010): every oracle workbook carries an embedded canary (poisoned cached values) and is refused unless Excel recalculated it; canary exposed that Excel-authored workbooks were never saved by the Graph session (old flow returned cached values) — fixed with a post-calculate no-trace edit; Graph calls retry dropped connections (T-0010); eval delta none
- 2026-10-04 — T-0004 (partial): smoke oracle-filled (canary-verified); Excel disproved D-function criteria trimming (engine fixed, criteria.md entry); eval now lets oracle values replace hand values keyed without sheet quotes; smoke 38/38 vs Excel, 0 regressions
- 2026-10-04 — Owner review: filed T-0016..T-0023, T-0402 (model tiers, board conflicts, task ids, milestone-order triage, POI oracle fill, evidence coverage, fuzzing, reviewer models, perf gate); T-0203 and T-0004 raised to P0; critical lane engine, harness, research, perf (API v0 done, api lane empty).
- 2026-10-04 — T-0022: security/architecture/quality reviewers on Sonnet, fidelity/performance pinned to Opus (decision 0015); eval delta none.
- 2026-10-04 — T-0017: `make new-task` assigns collision-free ids (all origin/* branches counted; decision 0016); eval delta none.
- 2026-10-04 — T-0016: docs/tasks/BOARD.md no longer committed (git-ignored, `make board` renders locally; decision 0017); eval delta none.
- 2026-10-04 — T-0023: tasks carry `tier: sonnet | opus`; sonnet (hourly) and opus (2-hourly) workers replace the lane workers; stuck sonnet runs hand off (decision 0018). Needs owner: re-apply docs/process/routines.md. Eval delta none.
- 2026-10-04 — T-0004 done: oracle calibration over ironcalc-calc (71,765 cells, 99.96% bit-exact; 7 day-first-locale cached cells, 4 web-vs-desktop INDIRECT, 18 last-digit float); canary pins en-US; probes/date-text-locale added (probes known failures 13 -> 18, new coverage); filed T-0024
- 2026-10-04 — T-0108: Excel 1900 leap-year serials in DATE/DAY/MONTH/YEAR/formatting; probes known failures 18 -> 15 (A1-A3 fixed), ironcalc-calc 1578 -> 1578, 0 regressions; filed T-0125
- 2026-10-04 — T-0108 follow-up: added probes/dates-early-1900 (oracle-filled, 16 cells, 4 known failures -> T-0125); fixed EOMONTH early serials and DATE(1900,1,0)
- 2026-10-04 — T-0019: excel-graph sidecars for 40 POI workbooks (`oracle_dir`, committed in eval/suites/poi/oracle); poi pending 237 -> 198, known failures 647 -> 959 (new coverage, 0 regressions; 257 web-vs-desktop conflicts left pending, decision 0019); harness now parses JSON floats exactly (serde_json `float_roundtrip`; oracle values had read 1 ulp off, probes-numeric 60.19% -> 68.08%); quirk: SIN/COS at multiples of 90°.
- 2026-10-04 — T-0206: probes/criteria-operands oracle-filled (25 cells); fixed <,>,<=,>= with boolean/error operands and D-function explicit '=' exactness; 0 regressions, probes 25/25 new cells pass
- 2026-10-05 — T-0028: agents check PR mergeability after every push, event and merge to main, and resolve conflicts at once (decision 0020); `make check` runs typos; eval delta none.
- 2026-10-05 — T-0015: spec2xlsx output deterministic (pinned core.xml creation time); eval delta none.
- 2026-10-05 — T-0027: shared-formula members shown shifted in eval show/snapshot (engine `shift_formula_text`); eval delta none.
- 2026-10-05 — T-0203: gen-m2 oracle-filled and gated (30 cases, 12,000 cells, 85.62% bit-exact, 1,726 known failures, new coverage); IF arity fixed in the catalog; T-0208 fixed (oracle `#SPILL!` read by key name; 74 cells); 14 gap tasks T-0209..T-0222; unknown rich errorTypes read as `#UNDECODED!` and stay unscored (poi 951 -> 950, T-0030).
- 2026-10-05 — T-0123: names may contain `\` (en-US lexer, identifier validity); poi known failures 950 -> 935 (15 fixed, 0 regressions).
- 2026-10-05 — T-0031: salvaged stale draft PRs #11/#13/#38 (T-0905, T-0907 restored; financial.md; T-0901/T-0925 findings) and closed them; eval delta none.
- 2026-10-05 — Log frozen here (decision 0024, T-0033); later work: `git log --first-parent main` and the steward's daily summary.
- 2026-10-05 — Owner re-applied the T-0023 routines (sonnet hourly, opus every 2 h); T-0032: sonnet queue buffer (≥10 ready sonnet tasks; `make ready-tasks`, decision 0021).
