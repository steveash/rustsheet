# Status

The steward routine owns the snapshot and the critical-lane line and rewrites them daily; agents may
still add or remove "Needs owner" entries. PRs no longer append log lines (decision 0024): the record
of merged work is `git log --first-parent main` plus each task's Outcome, and the steward lists what
merged since its last run.

## Snapshot — 2026-10-19 (M2 is current)

| Suite | Cases | Cells | Bit-exact | Known failures |
|---|---|---|---|---|
| ironcalc-calc | 219 | 72,223 | 98.42% | 1,144 (was 1,243) |
| poi | 336 | 27,290 | 98.29% (204 cases pending: no formulas, unreadable, or web-vs-desktop conflicts) | 481 (was 484) |
| gen-m2 | 30 | 12,000 | **100%** | 0 |
| gen-m1 | 15 | 3,750 | **100%** | 0 |
| gen-m10 | 20 | 1,690 | 93.08% | 117 |
| probes | 984 | 65,174 | 96.98% (4 accepted deviations, 11 cells passed by decision 0042) | 2,006 (was 2,411) |
| smoke | 6 | 46 | 100% | 0 |
| fixtures | 17 | 248 | 96.37% | 9 |
| corpus-bench | 912 | 134,348 (316 scored workbooks, 591 pending) | 99.63% (99.64% within 1e-9) | 14 (was 19); 5 load timeouts (T-0430) |
| gen-m5 | 10 | 400 | 99.25% | not gated |
| gen-m6 | 25 | 1,000 | 99.70% | not gated |
| gen-m7 | 25 | 994 | 99.80% | not gated |
| gen-m8a | 12 | 3,577 | 83.20% | not gated |
| gen-m8b | 9 | 3,368 | 91.78% | not gated |
| gen-m9 | 25 | 998 | 95.99% | not gated |
| probes-numeric | 149 | 80,360 | 91.26% | not gated |

`make eval-check` on `main` (30bee011) shows 0 regressions and 0 unrecorded fixes in every gated suite. poi and corpus-bench were fetched for this run. corpus-bench now scores 316 workbooks (142 last run): T-0432's canary fix and the refill landed.

CI on `main` is green at 30bee011. No CI run on `main` failed since the last snapshot; six were cancelled by the next push. `main` moved on during this run (T-1810 #1154, T-1389 #1143 and others); this PR merges it.

Where M2's exit stands:
- gen-m2: **100%, criterion met**.
- The frozen m1-probes list is at 0 failures (89 cases, 1,858 cells).
- M0–M2 probes: 405 cases, 11,553 cells, 11,504 bit-exact, 4 deviations, **45 known failures** (was 180). The planner checked every cell; each has an open owning task:
  - `sort-error-args` (23): T-1902 (functions-lookup, M7; H57:J71) and T-1839 (H97:H99, moved there by T-1833). SORT/SORTBY are M7 functions; **T-1964** (harness, new) maps the case to M7 in probe-milestones.tsv under a decision record, as 0058 did for `sort-omitted-args`.
  - `subtotal-take-array-selector` (9): **T-1914** (engine, now P0, see fuzz below) and T-1915 (engine).
  - `legacy-array-at-shapes` (6): T-1910 and T-1911 (engine).
  - `legacy-marker-spelling-name-plain` (2): T-1936 (engine). `legacy-match-lookup-array-intersection` (2): T-1935 (engine, in progress).
  - `sheet-ops-insert-row-legacy-cell` (2): T-1852 (engine; T-1851 is done, so it is ready).
  - `date-math-lift-driver-edges` G43 (1): T-1430 (functions-date, M6). DATEDIF is M6; T-1964 maps the cell to M6.
  - Fixed since the last run: all 148 `date-text-glued-month` cells (T-1789) and the 32 legacy N/T cells (T-1713, T-1851).
- ironcalc-calc cells using only M1/M2 functions: 4,253 scored (T-1877 now scores array members by their anchor), 4,251 bit-exact, 2 failing:
  - `LOGICAL/IF_ARRAY.xlsx!Sheet1!C11`: the decision 0074 exception (desktop SIN(4), M14 T-1075). T-1876 settled it.
  - `MATH_AND_TRIGONOMETRY/POWER.xlsx!inside_a_lambda!C3`: a member of `=gelu(A1:C1)`, a workbook LAMBDA name over TANH/POWER (M5/M7). It counts as M2 only because the scope check does not see through defined names. **T-1965** (harness, new) fixes the classification; the 1-ulp gap itself is T-0959 (M9).
- Criteria and aggregate iteration through the shared walks: met.
- Duplication is at 3,630 lines on current `main` (T-1389 locked it), 60.3% below the fork's 9,153 (target ≥ 15%): met.

Hygiene: lint-debt 91 allow entries (= last run) · duplicated lines 3,630 (ratchet 3,630 after T-1389; was 4,328) · oversized files 11 (ratchet locked 12 → 11 in this PR, static_analysis.rs off the list; was 13).

Since the 2026-10-18 snapshot (steward PR #1068), 92 commits landed on `main` through 30bee011, touching 89 task ids. None was reverted.
- 83 came through PRs (one is the steward's #1068).
- **9 were pushed straight to `main` with no PR or review**, all T-1778 research from session `01Xbmds2…`: 892a53ca (reported last run), fc3f993d, c00142bb, 0b5ce7a4, 36217c63, 756f849f, 37d303dd, 32ac084d and c763916b. CI passed on each. Most touch only `docs/research/`, but **32ac084d also edits the protected `.claude/skills/excel-probe/SKILL.md`** (and docs/excel-semantics/README.md) to point agents at a new, unreviewed `docs/research/excel-kernel-playbook.md`, with no decision record. I read the change and the playbook: both are benign research guidance. See "Needs owner".
- Task ids merged:
  - T-0058, T-0059, T-0405, T-0431 (blocked, findings), T-0432, T-0433, T-0434, T-0436, T-0437, T-0939, T-0941, T-1106, T-1155, T-1186
  - T-1284 (blocked, findings), T-1360, T-1373, T-1409, T-1486, T-1511, T-1532, T-1688, T-1713, T-1717, T-1766, T-1768, T-1773, T-1776
  - T-1778, T-1779, T-1785, T-1789, T-1792, T-1797, T-1805, T-1808, T-1817, T-1824, T-1841, T-1845, T-1847, T-1848, T-1850
  - T-1851, T-1853, T-1854, T-1857–T-1863, T-1865, T-1866, T-1868, T-1872–T-1874, T-1876, T-1877, T-1879, T-1882
  - T-1892–T-1894, T-1897, T-1901, T-1904, T-1906–T-1909, T-1912, T-1913, T-1916–T-1918, T-1921, T-1926, T-1929–T-1934, T-1937, T-1938

Blocked: 26 (25 last run). None can be unblocked.
- New: T-1284 (GEOMEAN, time-boxed by decision 0056 after its second attempt) and T-0431 (FUSE corpus: zenodo.org still unreachable from this environment, checked this run).
- T-1713 is done (no longer blocked).
- The Phase 2 tasks T-0400, T-0401, T-0402, T-0423 and T-1066 (decision 0027).
- T-1099 (GEOMEAN last ulp): its dependency T-1108 is done but concluded "no reproducible root model", so the blocker stands.
- Time-boxed fits (decision 0056) with no new hypothesis: T-0968, T-1528, T-1529, T-1592.
- 14 M14 desktop-oracle tasks (decision 0039).

**Nightly fuzz:** run 39196675245 (2026-10-19, c31b96ef) failed on `formula_eval`; `formula_parse` and `xlsx_import` passed.
- The input is `SUBTOTAL(9,TAKE(SWITCH({1,2},1,H1:H2,2,I1:I2),2))`, a deadly signal on the same `debug_assert` as T-1874/T-1882 ("Larger-than-1x1 array reached scalar-context cell", model.rs:928).
- I reproduced it on 30bee011 with a debug engine test, and the IFS twin `SUBTOTAL(9,TAKE(IFS({TRUE,FALSE},H1:H2,TRUE,I1:I2),2))` panics too.
- Deduplicated against the open **T-1914**, which owns exactly these formulas (probe cells only): raised it to **P0, opus**, with the reproducer and a unit-test criterion (decision 0035).

Open PRs: 6 (6 at the end of the last run), all opened in the last 24 h and none a draft: #1153 (T-1829), #1151 (T-1236) and #1144 (T-1910) are open; #1154 (T-1810), #1143 (T-1389) and #1116 (T-1292) merged during this run. The long-standing drafts #698, #553, #436 and #409 are closed (their tasks landed or were blocked with findings). No PR qualified for an orphan merge (decision 0050), and there are no duplicates.

Queues at the start of the run:
- functions: 35 ready. The sonnet buffer is 24, so the gap-triager did not run.
- engine 0, research 0 and quality 0 ready only because their lanes are busy (live branches T-1935, T-1941, T-1775); engine has 10 todo M2 tasks. harness 0, api 1 (M8), perf 20 (Phase 2).
- The planner ran for M2:
  - It traced all 47 M2-gap cells to owners (above), tightened the criteria of T-1902, T-1936, T-1852, T-1430 and T-0959, and filed **T-1964** and **T-1965** (harness).
  - Research has one M2 task (T-1943); quality has no M2 critical-path work. It filed nothing there.

Upstream IronCalc: one new commit under base/, xlsx/ or bindings/python, 507487ab (2026-10-18, "Remove views so the model no longer stores them"). It is a UI view-state refactor with no calculation or import fix, so no port task. UPSTREAM.md is triaged through 507487ab.

Oracle: excel-graph credentials are present.
- probes: 0 sidecars written (the 2 pending cases are M10 locale cases and are not fillable).
- ironcalc-calc: 0 written, 216 skipped.
- poi: 0 written, 292 skipped, with the same 4 workbook failures as before.
- corpus-bench: 0 written, 539 skipped (no formula cells), 4 workbook failures.

Spot-audit: the last one was 2026-10-16, so none is due (next on or after 2026-10-23).

**Critical lane:** engine, harness, research

(Ordered list read by both tier workers, which then fall back to `functions`. The current milestone is M2.
- engine: the P0 T-1914 (fuzz crash + 9 probe cells), then T-1852, T-1910, T-1911, T-1915, T-1935 and T-1936 hold the other M2 legacy cells.
- harness: T-1964 (milestone mapping, needs its decision record approved) and T-1965 (defined-name scope) settle what the M2 gate counts.
- research: T-1943 (INDEX whole-row shape probe).
- quality and api have no M2 work. perf stays out: Phase 2.)

## Needs owner

- **Direct pushes to `main`** (2026-10-18, continuing 2026-10-19). Session `01Xbmds2…` has now pushed fifteen T-1778 commits straight to `main` with no PR (6f85cc9 … 892a53ca reported last run; fc3f993d, c00142bb, 0b5ce7a4, 36217c63, 756f849f, 37d303dd, 32ac084d and c763916b since), so the reviewer agents never saw them. CI passed on each and nothing needs reverting. New this run: **32ac084d edits a protected path, `.claude/skills/excel-probe/SKILL.md`**, without the decision record the governance gate demands of a PR. It is a two-line pointer to the new `docs/research/excel-kernel-playbook.md`, and both read as benign research guidance. Still, agent instructions changed with no review. Recommended fix: a branch ruleset on `main` that requires a pull request (the steward and workers already merge through PRs, so nothing changes for them). If these pushes were your own interactive work, remove this item.

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
