# Status

The steward routine owns the snapshot and the critical-lane line and rewrites them daily; agents may
still add or remove "Needs owner" entries. PRs no longer append log lines (decision 0024): the record
of merged work is `git log --first-parent main` plus each task's Outcome, and the steward lists what
merged since its last run.

## Snapshot — 2026-10-20 (M5 is current: M2's exit criteria are met)

| Suite | Cases | Cells | Bit-exact | Known failures |
|---|---|---|---|---|
| ironcalc-calc | 219 | 72,223 | 98.54% | 1,056 (was 1,144) |
| poi | 336 | 27,290 | 98.36% | 462 (was 481; 4 fixed ASINH cells, recorded by T-1775 during this run) |
| gen-m2 | 30 | 12,000 | **100%** | 0 |
| gen-m1 | 15 | 3,750 | **100%** | 0 |
| gen-m10 | 20 | 1,690 | 95.38% | 78 (was 117) |
| probes | 1,061 | 68,836 | 97.62% (4 accepted deviations, 11 cells passed by decision 0042; 3 cases pending) | 1,666 (was 2,006) |
| smoke | 6 | 46 | 100% | 0 |
| fixtures | 17 | 248 | 96.37% | 9 |
| corpus-bench | 912 | 134,348 | 99.63% | 14; 5 load timeouts (T-0430) |
| gen-m5 | 10 | 400 | 99.50% | not gated |
| gen-m6 | 25 | 1,000 | 99.90% | not gated |
| gen-m7 | 25 | 994 | 99.90% | not gated |
| gen-m8a | 12 | 3,577 | 83.20% | not gated |
| gen-m8b | 9 | 3,368 | 91.78% | not gated |
| gen-m9 | 25 | 998 | 98.90% | not gated |
| probes-numeric | 153 | 80,719 | 91.41% | not gated |

`make eval-check` on `main` (380ffed2) shows 0 regressions in every gated suite. poi had 4 unrecorded fixes, which T-1775 (#1164) recorded during this run. poi and corpus-bench were fetched for this run.

CI on `main` is green at 380ffed2. No CI run on `main` failed since the last snapshot; 17 were cancelled by the next push. The nightly fuzz run 39366329072 (2026-10-20, 5ef2d6f1) passed all three targets.

**M2's exit criteria are all met**, so M5 is now the current milestone (docs/milestones.md phase order):
- gen-m2: 100%.
- M0–M2 probes (decision 0051): 427 cases, 12,147 cells, 12,143 bit-exact, 4 deviations, **0 known failures** (was 45). The frozen m1-probes list is also at 0.
- ironcalc-calc cells using only M1/M2 functions: 4,245 scored, 4,244 bit-exact. The 1 failing cell is `LOGICAL/IF_ARRAY.xlsx!Sheet1!C11`, the decision 0074 exception (T-1075, M14). T-1970 fixed the defined-name scope, so POWER's `inside_a_lambda!C3` no longer counts as M2.
- One criteria/aggregate walk: met. Duplication is 3,268 lines on current `main` (T-1775 locked it), 64% below the fork's 9,153.

The M2 todo tasks that are left (T-1939, T-1943, T-1964, T-1969, T-1981, T-2008, T-2018, T-2100, and perf T-1599) are follow-up probes, not exit gaps.

Where M5's exit stands (the planner checked this run):
- M5 probes: 202 cases, 8,549/8,549 bit-exact. Met.
- Generated suite ≥ 20k cells at ≥ 99.9%: **not met**. gen-m5 has 400 cells, calls 61 of the 131 generator-ready M5 functions, and is mostly M2 calls. Filed T-2108, T-2109 and T-2110 (harness: generator focus and edge pools) and T-2114 (research: generate and fill ≥ 20k cells, then gate the suite).
- ironcalc-calc at 100% for M5 categories: **not measured yet**. `function_scope.rs` stops at M2. Filed T-2111 (harness: one cumulative row per milestone). The planner estimates about 53 failing cells in scope, all last-digit math families. They are owned by T-0915, T-0917, T-0929 and T-1944, but those are tagged M9.
- Catalog coverage: every function is catalogued. INFO and SHEETS have no Excel cells because the oracle canary refuses them. Filed T-2112 (harness: a coverage report), T-2113 (harness: narrow the canary) and T-2115 (research: probe INFO/SHEETS).
- T-2116 (research, opus) is a decision record on how M5's exit is measured. It covers the cumulative scope, whether decision 0074 extends to IF_ARRAY C4/E4, and moving T-0915/T-0917/T-0929/T-1944 to M5.

Hygiene: lint-debt 91 allow entries (= last run) · duplicated lines 3,268 (ratchet 3,318 → 3,268, locked by T-1775 during this run) · oversized files 11 (= last run) · evidence floor: 3 in-scope functions under 20 Excel cells (= lock).

Since the 2026-10-19 snapshot (steward PR #1160), 109 commits landed on `main` through 380ffed2, touching 104 task ids. None was reverted.
- All 109 came through PRs and were squash-merged on GitHub. 15 have no `(#n)` in the title; I checked them, for example T-1799 is #1257. No commit was pushed straight to `main` this time.
- T-2107 (#1283) merged during this run as 4d9b7754 (decision 0081: reviewer-performance off, a 50% bench regression budget). Its PR says the owner asked for this on 2026-10-20. See "Needs owner".
- Task ids merged:
  - T-0408, T-0425, T-0438, T-0440, T-0927, T-0959, T-0969, T-1254, T-1385, T-1402–T-1406, T-1408, T-1428, T-1430, T-1453
  - T-1637, T-1708, T-1723, T-1724, T-1751, T-1764, T-1765, T-1795, T-1798, T-1799, T-1806, T-1813, T-1816, T-1819–T-1821, T-1823, T-1827, T-1829
  - T-1832, T-1834, T-1837, T-1852, T-1903, T-1911, T-1914, T-1915, T-1936, T-1945, T-1947, T-1949, T-1951–T-1953, T-1955–T-1959, T-1967
  - T-1970, T-1971, T-1976, T-1979, T-1982, T-1984–T-1990, T-1992, T-1994, T-1995, T-1997–T-1999, T-2001–T-2004, T-2009–T-2012, T-2014
  - T-2029–T-2031, T-2036, T-2041, T-2042, T-2052, T-2053, T-2060, T-2061, T-2063, T-2071, T-2075, T-2076, T-2078, T-2079, T-2087, T-2088, T-2090
  - Five task-only queue refills (#1197, #1215, #1218, #1220, #1263).
- The P0 fuzz task T-1914 is done.

Blocked: 28 (26 last run). None can be unblocked.
- New: T-0441 (data tables), which needs the M3 graph (Phase 2), and T-2037 (the steward regenerates the parity page), which needs an owner routine change.
- The rest are unchanged: Phase 2 (T-0400, T-0401, T-0402, T-0423, T-1066), time-boxed fits with no new hypothesis (T-0968, T-1528, T-1529, T-1592), T-1099/T-1284 (GEOMEAN), T-0431 (FUSE corpus) and 14 M14 desktop-oracle tasks (decision 0039).

Open PRs: 9 after T-2107 (#1283) and T-1898 (#1167) merged during this run. None has been open 24 h yet (the oldest, #1164, opened 2026-10-19 07:23 UTC), so no orphan merge was possible (decision 0050).
- Drafts: #1164 (T-1775), #1226 (T-0439), #1245 (T-1484).
- Open: #1202 (T-1993), #1209 (T-1996), #1210 (T-1943), #1212 (T-1878), #1230 (T-1835), #1284 (T-2101).
- No duplicate task ids.

Queues at the start of the run:
- functions: 7 ready (≥ 3), so the gap-triager did not run. The sonnet buffer is 17 (≥ 10).
- engine 0, harness 0, research 0, quality 1, api 2 (M12), perf 19 (Phase 2).
- The planner ran for M5 and filed T-2108–T-2116 (above). Ready now: harness 5, research 3. Engine, quality and api have no M5 exit gap, so it filed nothing there.

Upstream IronCalc: one new commit under base/, xlsx/ or bindings/python, 2331734c ("fix: apply default font color to existing templates"). It only restyles 14 `xlsx/tests/docs` workbooks and fixes no calculation or import, so no port task. UPSTREAM.md is triaged through 2331734c.

Oracle: excel-graph credentials are present (`rustsheet-oracle status`: excel-graph OK).
- probes: 0 written, 1 failure. `typed-if-one-argument` (T-1990) is refused with 501 `FileCorruptTryRepair`, as its Outcome records. The other 2 pending cases are the unfillable M10 locale cases.
- ironcalc-calc: 0 written, 216 skipped.

Spot-audit: the last one was 2026-10-16, so none is due (next on or after 2026-10-23).

**Critical lane:** harness, research

(Ordered list read by both tier workers, which then fall back to `functions`. The current milestone is M5.
- harness: T-2108 → T-2109 → T-2110 make the generator able to reach the ≥ 20k-cell criterion. T-2111 and T-2112 make the ironcalc-calc and coverage criteria measurable. T-2113 lets INFO/SHEETS be filled.
- research: T-2116 (measurement decision), then T-2114 (generate and fill gen-m5), plus T-2115, T-2059 and T-2062 (probes).
- The M5 failures themselves sit in functions-math and functions-text (T-0915, T-0917, T-0929, T-1944), which the workers reach through `functions`.
- engine, quality and api have no M5 exit work. perf stays out: Phase 2.)

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
