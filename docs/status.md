# Status

The steward routine owns the snapshot and the critical-lane line and rewrites them daily; agents may
still add or remove "Needs owner" entries. PRs no longer append log lines (decision 0024): the record
of merged work is `git log --first-parent main` plus each task's Outcome, and the steward lists what
merged since its last run.

## Snapshot — 2026-10-17 (M2 is current)

| Suite | Cases | Cells | Bit-exact | Known failures |
|---|---|---|---|---|
| ironcalc-calc | 217 | 72,203 | 98.20% | 1,300 |
| poi | 336 | 27,290 | 98.27% (204 cases pending: no formulas, unreadable, or web-vs-desktop conflicts) | 488 |
| gen-m2 | 30 | 12,000 | **100%** | 0 |
| gen-m1 | 15 | 3,750 | **100%** | 0 |
| gen-m10 | 20 | 1,690 | 93.08% | 117 |
| probes | 797 | 46,257 | 93.58% (4 accepted deviations, 11 cells passed by decision 0042) | 3,004 |
| smoke | 6 | 46 | 100% | 0 |
| fixtures | 17 | 248 | 96.37% | 9 |
| corpus-bench | 912 | 0 (crash-only, 907 pending until T-0427) | — | 5 |
| gen-m5 | 10 | 400 | 99.25% | not gated |
| gen-m6 | 25 | 1,000 | 99.50% | not gated |
| gen-m7 | 25 | 994 | 97.08% | not gated |
| gen-m8a | 12 | 3,577 | 83.20% | not gated |
| gen-m8b | 9 | 3,368 | 91.78% | not gated |
| gen-m9 | 25 | 998 | 92.79% | not gated |
| probes-numeric | 138 | 20,446 | 57.00% | not gated |

`make eval-check` on `main` (ea76d91f) shows 0 regressions and 0 unrecorded fixes in every gated suite, poi included (fetched for this run, with corpus-bench).

CI on `main` is green at ea76d91f. No CI run on `main` failed since the last snapshot (one run, 837be632, was cancelled by the next push).

Where M2's exit stands:
- gen-m2: **100%, criterion met**.
- ironcalc-calc cells using only M1/M2 functions: criterion met.
- The frozen m1-probes list is at 0 failures (89 cases, 1,858 cells).
- M0–M2 probes: 366 cases, 10,439 cells, 10,365 bit-exact, 4 deviations, **70 known failures** (was 29). The rise is new coverage, not regressions: T-1713 (#958) and T-1756 (#981) added probes whose failing cells all have open tasks. All 29 failures from the last snapshot are fixed.
  - `legacy-n-t-reference-args` (37) and `legacy-array-result` (19): T-1728 (engine; PR #985 merged during this run at ba903077, so 6 of these cells are fixed on `main` now), T-1717 and the spec T-1727 (engine). T-1713 (functions-logic) stays blocked on T-1727.
  - `unknown-function-name-reach` (5): T-1760, T-1762, T-1763 (engine).
  - `text-ap-locale-inferences` (4): T-1716 (functions-text).
  - `n-array-lift` (2): T-1713, after T-1727.
  - `unknown-function-name-spelling` (2): T-1758 (engine).
  - `t-array-lift` (1): T-0985 (functions-text).
  - T-1716 had been filed in M10 and T-0985 in M9, although their cells count toward M2 (decision 0051). This PR moves them to M2.
- Criteria matching and aggregate-argument iteration: T-1677 (AND/OR/XOR onto the shared walk) merged. The planner confirmed that SUM, AVERAGE, COUNT, COUNTA, MIN, MAX, AND, OR and XOR all go through the shared walk, and every *IF(S), D-function, MATCH and XMATCH consumer goes through criteria.rs: **met**. The private walks left (GCD/LCM, MULTINOMIAL, CONCAT/TEXTJOIN, COUNTBLANK, CHISQ, date serials, SUMX*) are outside SPEC-0009's scope.
- Duplication is at 6,155 lines, 32.8% below the fork's 9,153 (target ≥ 15%), so the number is met.

Hygiene: lint-debt 91 allow entries (= last run) · duplicated lines 6,155 (ratchet 6,155, tightened since the last run) · oversized files 14 (ratchet 14).

Since the 2026-10-16 snapshot (steward PR #900, cd16ff51), 81 PRs merged to `main` through ea76d91f: 78 task PRs and 3 task-filing PRs (#917, #919, #965). None was reverted.
- T-0426, T-0901, T-0916, T-1291, T-1307, T-1322–T-1327, T-1356, T-1439, T-1535, T-1554, T-1557
- T-1569, T-1607, T-1608, T-1610, T-1611, T-1616, T-1620–T-1624, T-1628, T-1632
- T-1644–T-1646, T-1651, T-1656–T-1659, T-1663, T-1667, T-1669, T-1670, T-1672–T-1677
- T-1678–T-1682, T-1685, T-1693–T-1697, T-1702, T-1704, T-1705, T-1709, T-1710
- T-1712–T-1715, T-1718–T-1720, T-1722, T-1729, T-1739, T-1744–T-1746, T-1756, T-1759

Blocked: 25 (24 last run, plus T-1713). None can be unblocked.
- T-1713 waits on the spec T-1727 (todo).
- The Phase 2 tasks T-0400, T-0401, T-0402, T-0423 and T-1066 (decision 0027).
- T-1099 (GEOMEAN last ulp).
- Time-boxed last-digit fits (decision 0056) with no new hypothesis: T-0968, T-1528 (attempt 3 recorded in #917, no gain), T-1529, T-1592.
- The M14 desktop-oracle tasks T-0171, T-0198, T-1016, T-1051, T-1074, T-1075, T-1081, T-1174, T-1204, T-1277, T-1299, T-1310, T-1367 and T-1553 (decision 0039).

**Nightly fuzz:** run 39000004030 (2026-10-17, 22a43ad7) failed on `formula_eval` with a timeout. The previous night's OOM (T-1681) is fixed.
- The input is `SUM(MAP(A1:A1000,LAMBDA(rSMU,((B1:B100000)))))`.
- I reproduced it on ea76d91f with a release facade build: 5.3 s for 1,000 rows, linear in rows × range height. `BYROW` and `MAKEARRAY` with the same LAMBDA take under 1 ms.
- Root cause: `fn_map` passes each call's result through `lambda_result_value`, which materialises a multi-cell range into a full array before the element is stored.
- Filed **T-1769** (functions-logic, P0, opus).
- `formula_parse` and `xlsx_import` passed.

Open PRs: 9 at the end of the run (5 at the start of the last one).
- #891 (T-1316) was the one agent PR open over 24 h and not a blocked draft. Its `hygiene` check had been red since 2026-10-16 because zip 0.6.6 was yanked, which T-1668 fixed on `main`, and its own fidelity review was unconfirmed.
  - This run merged `main` into it. It also dropped the T-0057 task file it carried, a duplicate of T-1668, and fixed the Outcome's cell count.
  - All four reviewers (security, architecture, quality, excel-fidelity) re-ran on that diff and approved. `make check` passed.
  - Another session squash-merged it at 0514aabf (06:33 UTC) before this run's up-to-date push. CI on `main` is green there, and the local `make check` of the PR plus T-1728 passed.
  - The fidelity reviewer's MINOR gaps (an unprobed scalar TRUE beside an array bound, and error order per element) are filed as **T-1782** (functions-math, sonnet).
- Under 24 h: #912 (T-1626), #941 (T-1698), #980 (T-1631), #986 (T-1598), #987 (T-1767).
  - #912 renumbered its ASC follow-up to T-1691, which `main` has since given to a 3-D span task (M8). Merging `main` into it will need a fresh id from `make new-task`.
- Blocked drafts, each with a `status: blocked` task on its branch: #698 (T-1486), #553 (T-1284), #436 (T-1186), #409 (T-1155).
  - None of their branches moved since the last run.
  - #698's blocker, commented on the PR on 2026-10-15, still holds.
- No duplicates.

Queues at the start of the run:
- functions: 61 ready. The sonnet buffer is 55, so the gap-triager did not run.
- engine 0, api 1, harness 2, quality 0, research 0, perf 17.
  - engine and research are busy, not empty. Their branches moved in the last 2 h (T-1728, T-1767; T-1625), and engine holds 6 unclaimed M2 tasks (T-1717, T-1727, T-1758, T-1760, T-1762, T-1763).
  - The planner ran for M2:
    - It filed **T-1778** (quality, sonnet): resolve six code TODOs that name done tasks. It also filed **T-1779** (quality, opus, after T-1778): `make hygiene` rejects a TODO naming a done, dropped or missing task.
    - Six failing cells had no owner in any task's acceptance criteria: probes/legacy-n-t-reference-args S!D11, D20, M11, M20, P11 and P20. T-1717's Context already named them, and this PR adds them to its acceptance criteria.
    - research and api have no M2 work left. The api lane's ready task, T-1671, is M8.

Upstream IronCalc: no new engine commits. 7f992398 only touches the webapp, and 297553dc is the v0.9.0 version bump. UPSTREAM.md is now recorded as triaged through 297553dc (the last run triaged it but did not record it).

Oracle: excel-graph credentials are present.
- probes: 0 sidecars written. The 1 pending case is not fillable.
- ironcalc-calc: 0 written, 214 skipped.
- poi: 0 written, 292 skipped (the same 4 workbook failures as T-0019).
- corpus-bench's 907 pending workbooks are left for T-0427 (batched fill, todo). Its suite holds no Excel evidence yet.

Spot-audit: the last one was 2026-10-16, so none is due (next on or after 2026-10-23).

**Critical lane:** engine, quality

(Ordered list read by both tier workers, which then fall back to `functions`. The current milestone is M2.
- engine: T-1717, T-1727, T-1758, T-1760, T-1762 and T-1763 hold 59 of the 64 M0–M2 probe failures left after T-1728.
- quality: T-1778 and T-1779, hygiene from the planner.
- The other 5 failing cells are reached through `functions`: T-1716, T-0985 and, after T-1727, T-1713. So is the P0 T-1769.
- research, api and harness have no M2 work. perf stays out: Phase 2.)

## Needs owner

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
