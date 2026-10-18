# Status

The steward routine owns the snapshot and the critical-lane line and rewrites them daily; agents may
still add or remove "Needs owner" entries. PRs no longer append log lines (decision 0024): the record
of merged work is `git log --first-parent main` plus each task's Outcome, and the steward lists what
merged since its last run.

## Snapshot — 2026-10-18 (M2 is current)

| Suite | Cases | Cells | Bit-exact | Known failures |
|---|---|---|---|---|
| ironcalc-calc | 219 | 72,223 | 98.28% | 1,243 |
| poi | 336 | 27,290 | 98.28% (204 cases pending: no formulas, unreadable, or web-vs-desktop conflicts) | 484 |
| gen-m2 | 30 | 12,000 | **100%** | 0 |
| gen-m1 | 15 | 3,750 | **100%** | 0 |
| gen-m10 | 20 | 1,690 | 93.08% | 117 |
| probes | 889 | 58,557 | 95.96% (4 accepted deviations, 11 cells passed by decision 0042) | 2,411 |
| smoke | 6 | 46 | 100% | 0 |
| fixtures | 17 | 248 | 96.37% | 9 |
| corpus-bench | 912 | 45,444 (142 scored workbooks, 765 pending) | 97.53% (98.35% within 1e-9) | 19; 5 load timeouts (T-0430) |
| gen-m5 | 10 | 400 | 99.25% | not gated |
| gen-m6 | 25 | 1,000 | 99.70% | not gated |
| gen-m7 | 25 | 994 | 99.40% | not gated |
| gen-m8a | 12 | 3,577 | 83.20% | not gated |
| gen-m8b | 9 | 3,368 | 91.78% | not gated |
| gen-m9 | 25 | 998 | 94.99% | not gated |
| probes-numeric | 139 | 20,878 | 57.91% | not gated |

`make eval-check` on `main` (cdf7d088) shows 0 regressions and 0 unrecorded fixes in every gated suite. poi and corpus-bench were fetched for this run.

CI on `main` is green at cdf7d088. No CI run on `main` failed since the last snapshot. Five runs were cancelled by the next push.

Where M2's exit stands:
- gen-m2: **100%, criterion met**.
- The frozen m1-probes list is at 0 failures (89 cases, 1,858 cells).
- M0–M2 probes: 379 cases, 10,981 cells, 10,797 bit-exact, 4 deviations, **180 known failures** (was 70). The rise is new coverage: T-1788 added `date-text-glued-month`. All 180 cells have an open owning task (the planner checked them cell by cell):
  - `date-text-glued-month` (148): T-1789 (functions-date, todo; T-1788 is done).
  - `legacy-n-t-reference-args` (25), `legacy-array-result` (5) and `n-array-lift` (2): T-1713's acceptance list. T-1713 (now engine) is blocked on T-1851. The spec T-1727 merged, so T-1851 (engine, todo) is the next step, and T-1852 follows it.
  - Fixed since the last run: `unknown-function-name-reach`, `unknown-function-name-spelling`, `text-ap-locale-inferences` and `t-array-lift` (T-1758, T-1760, T-1762, T-1763, T-1716, T-0985).
- ironcalc-calc cells using only M1/M2 functions: the report scores 3,978 of 3,978 bit-exact, but it leaves out 1,378 array-member cells with no formula of their own. 46 of them fail. The planner traced each one to its anchor. One, `LOGICAL/IF_ARRAY.xlsx!Sheet1!C11`, is a member of `IF(FALSE,A1:A4,B1:C4)` and is in M2 scope. So the criterion is **not quite met**.
  - **T-1877** (harness, sonnet) scores member cells by their anchor's formula.
  - **T-1876** (research, opus) settles the target for desktop SIN(4) in IF_ARRAY C4, E4 and C11. Desktop cached ...283 there, but the web and RustSheet give ...282.
- Criteria and aggregate iteration through the shared walks: met (last run).
- Duplication is at 4,328 lines, 52.7% below the fork's 9,153 (target ≥ 15%): met.

Hygiene: lint-debt 91 allow entries (= last run) · duplicated lines 4,328 (ratchet 4,328; was 6,155) · oversized files 13 (ratchet 13; was 14).

Since the 2026-10-17 snapshot (steward PR #988, 82bd38d), 86 commits landed on `main` through cdf7d088, touching 79 task ids. None was reverted.
- 80 came through PRs.
- **6 were pushed straight to `main` with no PR or review**: T-1778's research plan (6f85cc9, 4700e68, cbed44b, 80c5abc, 67da446, 226050a), all from session `01Xbmds2…`. A seventh, 892a53ca, landed during this run. They touch only `docs/research/t1778-erf-kernel-research-plan.md`, and CI passed on each. See "Needs owner".
- Task ids merged:
  - T-0427, T-0985, T-1236, T-1237, T-1241, T-1242, T-1253, T-1271, T-1272, T-1290, T-1295, T-1297, T-1308
  - T-1347–T-1355, T-1361, T-1372, T-1383, T-1384, T-1413, T-1431, T-1517, T-1600, T-1605, T-1626, T-1631
  - T-1687, T-1698, T-1700, T-1703, T-1706, T-1716, T-1717, T-1727, T-1747, T-1758, T-1760, T-1762, T-1763
  - T-1767, T-1769–T-1772, T-1774, T-1777, T-1778, T-1780–T-1784, T-1786–T-1788, T-1791, T-1793, T-1794, T-1796
  - T-1800–T-1802, T-1804, T-1807, T-1815, T-1818, T-1822, T-1825, T-1826, T-1831, T-1840, T-1842

Blocked: 25 (= last run). None can be unblocked.
- T-1713 now waits on T-1851 (todo) as well as the done spec T-1727.
- The Phase 2 tasks T-0400, T-0401, T-0402, T-0423 and T-1066 (decision 0027).
- T-1099 (GEOMEAN last ulp).
- Time-boxed fits (decision 0056) with no new hypothesis: T-0968, T-1528, T-1529, T-1592.
- 14 M14 desktop-oracle tasks (decision 0039).

**Nightly fuzz:** run 39094842217 (2026-10-18, 052af1ed) failed on `formula_eval`. Last night's MAP timeout (T-1769) is fixed.
- The input is `IMSECH(CHOOSE({1,2},{1,2,3},{4;5})`. It is a deadly signal: the `debug_assert` "Larger-than-1x1 array reached scalar-context cell" at model.rs:928.
- I reproduced it on cdf7d088 with a debug facade test. `IMABS(...)` and `IMSECH(CHOOSE({1,2},1,{4;5}))` panic too, but `ABS(CHOOSE(...))` and `IMSECH({1,2,3})` do not.
- Filed **T-1874** (functions-engineering, P0, opus).
- `formula_parse` and `xlsx_import` passed.

Open PRs: 6 (9 at the end of the last run).
- Over 24 h, all drafts:
  - #698 (T-1486): its branch moved at 05:43 UTC today (another session merged `main` and added probe rounds), so it is live and left alone.
  - Blocked drafts with a `status: blocked` task on the branch and no branch movement: #553 (T-1284), #436 (T-1186), #409 (T-1155).
  - So no PR qualified for an orphan merge (decision 0050).
- Under 24 h: #1066 (T-1848, merged by its session during this run), and #1060 (T-1373, a draft handed off to opus).
- No duplicates.

Queues at the start of the run:
- functions: 27 ready. The sonnet buffer is 22, so the gap-triager did not run.
- engine 23, harness 5, perf 18, api 1, quality 1, research 0 (busy: a branch moved in the last 2 h, with T-1768, T-1847 and T-1853 todo).
- The planner ran for M2:
  - It filed **T-1877** and **T-1876** (above).
  - api has no M2 work (its ready task T-1671 is M8).
  - quality's open M2 work is T-1779.

Upstream IronCalc: no new commits under base/, xlsx/ or bindings/python since 297553dc (triaged last run).

Oracle: excel-graph credentials are present.
- probes: 0 sidecars written (the 2 pending cases are M10 locale cases and are not fillable).
- ironcalc-calc: 0 written, 216 skipped.
- poi: 0 written, 292 skipped, with the same 4 workbook failures as before: 47804 CELL("width"), 52575_main unreadable, bug67784 and sample-beta refused by Graph.
- corpus-bench: not refilled. T-0427 filled it on 2026-10-17. 182 of its 186 pending workbooks wait on the canary fix T-0432 (harness, ready), so a refill would write nothing new.

Spot-audit: the last one was 2026-10-16, so none is due (next on or after 2026-10-23).

**Critical lane:** engine, harness, research, quality

(Ordered list read by both tier workers, which then fall back to `functions`. The current milestone is M2.
- engine: T-1851, then T-1852, unblocks T-1713, which holds the 32 legacy M0–M2 probe failures.
- harness: T-1877 brings the ironcalc-calc M2 criterion's 1,378 member cells into scope.
- research: T-1876 (desktop SIN target for IF_ARRAY), T-1847 and T-1853 (legacy probes feeding T-1851/T-1852).
- quality: T-1779 (TODO hygiene gate).
- The 148 `date-text-glued-month` cells are reached through `functions` (T-1789, functions-date), and so is the P0 T-1874 (functions-engineering).
- api has no M2 work. perf stays out: Phase 2.)

## Needs owner

- **Direct pushes to `main`** (2026-10-18). Session `01Xbmds2…` pushed seven T-1778 research-plan commits straight to `main`, from 6f85cc9 to 892a53ca (one during this run), with no PR, so the reviewer agents never saw them. They are docs-only and CI passed on each, so nothing needs reverting. But `main` evidently accepts direct pushes, which bypasses the review gate this project relies on. Recommended fix: a branch ruleset on `main` that requires a pull request. The steward and workers already merge through PRs, so this changes nothing for them. If the pushes were your own interactive instruction for the external review, remove this item.

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
