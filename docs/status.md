# Status

The steward routine owns the snapshot and the critical-lane line and rewrites them daily; agents may
still add or remove "Needs owner" entries. PRs no longer append log lines (decision 0024): the record
of merged work is `git log --first-parent main` plus each task's Outcome, and the steward lists what
merged since its last run.

## Snapshot — 2026-10-25 (M5 is current)

| Suite | Cases | Cells | Bit-exact | Known failures |
|---|---|---|---|---|
| ironcalc-calc | 219 | 72,357 | 98.98% | 737 (was 831) |
| poi | 336 | 27,209 | 99.61% | 120 (was 130) |
| gen-m5 | 108 | 38,840 | 99.98% | 9 (= last run) |
| gen-m6 | 85 | 25,000 | 99.99% | 2 (was 155) |
| gen-m7 | 52 | 31,845 | **100%** | 0 (was 857) |
| gen-m9 | 261 | 95,262 | 94.05% | 5,664 (newly gated, T-2255) |
| gen-m2 | 30 | 12,000 | **100%** | 0 |
| gen-m1 | 15 | 3,750 | **100%** | 0 |
| gen-m10 | 20 | 1,690 | 99.11% | 15 (was 45); displayed text 406 (was 1,148) |
| probes | 1,425 | 92,498 | 97.25% (18 accepted deviations; 3 cases pending) | 2,570 (was 2,020; 219 new cases) |
| smoke | 6 | 46 | 100% | 0 |
| fixtures | 22 | 263 | **100%** | 0 |
| corpus-bench | 912 | 134,348 | 99.63% | 12 (was 14) |
| calipers | 454 | 1,354,921 | 99.79% | baseline 3,461; `gate = false` (new, T-0446) |
| gen-m8a | 12 | 3,577 | 100% | not gated |
| gen-m8b | 9 | 3,368 | 91.78% | not gated |
| probes-numeric | 189 | 134,384 | 73.71% (99.84% within 1e-9) | not gated |

`make eval` on `main` (2c523462) shows 0 regressions in every gated suite. poi, corpus-bench and calipers were fetched for this run. The first run shared the machine with twelve reviewer agents, and 6 corpus-bench workbooks hit the 60 s case timeout. The quiet rerun (`make eval-lower-visible-lock`, after decision 0110 below) had no timeouts, and it lowers the visible-parity lock from 8,632 to 6,039 (decision 0093).

**Fifteen-digit parity: not met (coverage 428/506, visibly-off cells 13047)**. The last recorded line (2026-10-22) also read "not met" (112/506, 9,288 cells; more Excel cells are scored now), so the checkpoint is not reached (decision 0094).

CI on `main` is green through 5ec25407; the runs for 5b51e9e0 and 0ab959f4 were still in progress at 00:50 UTC. None of the 228 `main` push runs since the last snapshot failed (199 green, 27 cancelled by the next push).

The nightly fuzz run failed once since the last snapshot (2026-10-24, run 40059742446): `formula_eval` on `DB(6912,0,1E+300,2,6)`, an i64 add overflow. T-2570 (P0) already fixed it, with the reproducer as a unit test. T-2351 (last run's crash) is done. The 2026-10-23 run was green.

**M2's exit is met: M0–M2 probes (decision 0051) have 0 known failures** (457 cases, 14,553 cells, 14,549 bit-exact, 4 deviations), so M5 is current again.
- The one M0–M2 failure on 2c523462 was `probes/xnpv-empty-values-cell` C8. T-2599 (#1690) tagged that XNPV probe `M2`, which its reviewers missed (decision 0058 says to catch this). **Decision 0110** (this PR) adds its `M9` override line to `eval/milestones/probe-milestones.tsv`; the cell keeps its owner T-2603 (M9) and its baseline line.
- T-2191 and T-2326 fixed last run's 5 failures. The frozen m1-probes list is still at 0.

Where M5's exit stands ("Milestone exit (decision 0085)" in the report):
- Catalog coverage: 135/135. **Met** (T-2353 added the INFO and SHEETS cells).
- gen-m5: 38,840 scored cells at 0.99977, 9 known failures against an allowance of 38. **Met.**
- ironcalc-calc M5 row: 0 failing. **Met.**
- M5 probes: 241 cases, 8 known failures. **Not met.**
  - Failing cases: text-255-units-from-cell (2), text-255-units-rept-error-count (2) and text-255-units-array-element (1), all T-2627; numbervalue-separator-edges (1), T-2441; roman-lift-forms (1), T-2523; text-format-digits-date-token (1), T-2388.
  - All are functions-lane work. `make exit-owners MILESTONE=M5` reports 0 unowned cells.
- M6 (next): coverage 38/39 is not met; gen-m6 (2/25) and M6 probes (0) are met.

Hygiene: lint-debt 91 allow entries (= last run) · duplicated lines 3,028 at c29fedf3 (was 3,121; ratcheted) · oversized files 9 (was 11; `stringify.rs` is no longer oversized; ratcheted).

Since the 2026-10-22 snapshot (steward PR #1509), 221 commits landed on `main` through 0ab959f4. None was reverted.
- All 221 were squash-merged through PRs on GitHub (committer GitHub). The 23 titles without `(#n)` are PR squashes too: for example T-2540 is #1729, merged at 00:33 UTC. No direct pushes to `main`.
- There were no steward snapshots on 2026-10-23 or 2026-10-24: no `agent/steward/*` branch or PR exists for those days.
- Task ids merged (217):
  - T-0063, T-0444–T-0446, T-0451, T-0973, T-0974, T-0977, T-1390, T-1538, T-1592, T-1634, T-1699, T-1726, T-1732, T-1733, T-1735, T-1741–T-1743, T-1835, T-1849, T-1899, T-1900, T-1927, T-1944, T-1946, T-1981
  - T-2005, T-2008, T-2018, T-2028, T-2033, T-2035, T-2038–T-2040, T-2044, T-2049, T-2050, T-2057, T-2066–T-2069, T-2082, T-2086, T-2093, T-2117, T-2121, T-2122, T-2124, T-2137, T-2139, T-2140, T-2143, T-2172, T-2189, T-2191, T-2192
  - T-2203, T-2207, T-2216, T-2218, T-2226, T-2231–T-2233, T-2238, T-2241, T-2242, T-2247–T-2249, T-2252–T-2258, T-2262, T-2270–T-2273, T-2277, T-2283, T-2286–T-2289, T-2291, T-2293, T-2294, T-2296
  - T-2301, T-2305, T-2310–T-2313, T-2315, T-2318, T-2319, T-2322, T-2326, T-2328, T-2329, T-2332, T-2334, T-2340, T-2348, T-2350–T-2355, T-2357, T-2359–T-2363, T-2365, T-2368, T-2373–T-2383, T-2386, T-2387, T-2389–T-2391, T-2393–T-2397, T-2399
  - T-2401, T-2405–T-2415, T-2417–T-2423, T-2426, T-2428, T-2436, T-2438, T-2440, T-2442, T-2450, T-2453–T-2455, T-2457, T-2459, T-2465, T-2475, T-2482–T-2485, T-2489, T-2497, T-2499
  - T-2514, T-2515, T-2520, T-2525, T-2526, T-2533, T-2539, T-2540, T-2547, T-2550, T-2553, T-2558, T-2564, T-2567, T-2568, T-2570, T-2578–T-2581, T-2583, T-2592–T-2594, T-2599, T-2600, T-2602, T-2625
  - Task-only PRs: #1659, #1679.

Blocked: 31 (31 last run, counted by front matter).
- New: T-2425 (FORECAST.ETS family, needs excel-desktop; decision 0039, M14-style wait).
- Unblocked: T-0441 (done in #1706) and T-1699 (done).
- T-2119 (#1620) and T-0971 (#1536) were blocked at the start of the run because the workers' permission policy refused their `rustsheet-eval update-baseline --allow-regressions` step (decision 0083's standing ulp allowance). Both merged during this run.
- The rest are unchanged: Phase 2 (T-0400, T-0401, T-0402, T-0423, T-1066), time-boxed fits with no new hypothesis (T-0968, T-0969, T-1528, T-1529, T-1592), T-1099/T-1284 (GEOMEAN), T-0431 (FUSE corpus), T-2037 (needs a routine change) and the M14 desktop-oracle tasks (decision 0039).

Open PRs: 9 at 00:55 UTC. Four agent PRs had been open longer than 24 h: #1536 (T-0971), #1583 (T-2398), #1620 (T-2119) and #1658 (T-2131, passes 24 h at 02:03 UTC).
- All four are drafts whose tasks are `status: blocked`, so none can be merged as an orphan (decision 0050).
  - #1536 and #1620: the `--allow-regressions` permission (above). Both merged during this run (#1620 at about 01:45 UTC, #1536 by 02:15 UTC).
  - #1583: waits on T-2459 (nested-array spec).
  - #1658: waits on T-2565 evidence.
- **Closed #1700 (T-2580) as a duplicate.** #1703 had already merged T-2580, and #1700's branch had been idle since 2026-10-24 15:15 UTC. I commented with a link to #1703 and deleted nothing.
- #1682 (T-2532, done, green) was opened 2026-10-24 09:45 UTC, so it is under 24 h. #1718, #1727, #1732 and #1733 are newer.
- No task is `Blocked because: review-round limit`.

Queues at the start of the run:
- functions: 48 ready (≥ 3), so the gap-triager did not run. The sonnet buffer is 51 (≥ 10).
- M5 ready by lane: engine 0, harness 0, research 0, quality 4, api 3.
  - Engine and harness have no open M5 tasks. Research has T-2517 and T-2529 unclaimed, but its lane is busy (T-2416, #1733).
- The planner ran for M5 and filed **T-2648** (research, opus, `campaign: visible`). It is a gen-m5 sweep for the 41 M5 functions with fewer than 200 generated outer-call cells (CELL, ERROR.TYPE, TEXTJOIN, MOD, AGGREGATE, IFS, SWITCH and others; 1,027 visibly-off cells).
  - It filed nothing for engine: all 8 M5 probe failures are fixable inside their functions.
  - It filed nothing for harness: every M5 exit measure is already reported and owned.

Upstream IronCalc: eight new commits under base/, xlsx/ or bindings/python since 7e271963 (34b8bb6e, eb9ca75b and 1fd0ff4f were triaged last run).
- f04cbbed and 9cfbdfce replace `ROUND.xlsx` with an Excel-authored `ROUND_ROUNDUP_ROUNDDOWN.xlsx`. Filed **T-2634** (functions-math, sonnet). It says not to take the `~$ROUND.xlsx` lock file that 9cfbdfce added.
- 4ef43bed, 026f6c47, f8f68e27 and 2aa0e653 are defined-name fixes:
  - formula bodies for names;
  - sheet-qualified names such as `Sheet1!ConstPi`;
  - local-name formulas resolving in their own sheet;
  - `Data!Q1_total` lexed as a name.
  - They also add the Mac-desktop-authored `named_ranges.xlsx`; ee4172e8 re-saves it.
  - Filed **T-2635** (engine, M8, opus). We already have sheet-qualified names (T-1641, T-1693), so the task checks each upstream case first.
- ff282eb1 is a clippy fix only.
- UPSTREAM.md is triaged through 9cfbdfce.

Oracle: excel-graph credentials are present (`rustsheet-oracle status`: excel-graph OK).
- probes: 0 written, 1 failure. `typed-if-one-argument` (T-1990) is still refused with 501 `FileCorruptTryRepair`. The other 2 pending cases are the unfillable M10 locale cases.
- ironcalc-calc: 0 written, 216 skipped (Excel-authored).
- poi's 205 pending cases are web-vs-desktop conflicts (decision 0019), so they were not refilled.
- corpus-bench and calipers pending cases are not oracle inputs.

Spot-audit (2026-10-25; the last was 2026-10-16). I sampled 3 code PRs merged since then (seeded random draw from 553) and re-reviewed each squash diff with every reviewer on `model: opus`.
- #1062 (T-1706, GROUPBY/PIVOTBY eta arity from the catalog minimum): 4/4 APPROVE.
  - **Miss** (architecture, which ran on Sonnet at merge time, and Excel fidelity, which ran on Opus): minimum-0 functions (ROW, COLUMN, SHEET, RANDARRAY, PI…) now make the whole result `#VALUE!`. That is extrapolated from NOW alone, and no probe task was filed. Filed **T-2644** (research).
  - Minor, not filed: the `table_matches_catalog` doc says "generated" (it is hand-maintained), and the table is re-scanned per call.
- #1121 (T-1926, MATCH error lookup_value over an array match_type): 4/4 APPROVE.
  - **Miss** (Excel fidelity on Opus; architecture on Sonnet): per-slot error order against a bad match_type element, and the shape against an error or 2-D lookup_array, are unprobed. lookup.md's `{0;"x"}` note can be misread. Filed **T-2637** (research).
- #1374 (T-2204, decision 0089, EXP_SIGN B44): security, quality and fidelity APPROVE; architecture REQUEST_CHANGES.
  - **Miss** (architecture on Sonnet): decision 0089 item 2 pre-authorised later exit-exception lines without their own decision record, against decision 0087. It is moot on `main`: decision 0090 (T-2210) replaced item 2, and the B44 line is gone.
  - **Miss** (Excel fidelity on Opus; architecture on Sonnet): T-2211's export rule covers negative subnormals and desktop saves without evidence. I recorded the evidence gap in T-2211.
  - Minor, not filed: older numbers.md passages still blame Graph for the 0, and `runner.rs` `graph_hides_subnormal` doc says the same.
- No real defect reached `main` in the sample: every miss is unprobed scope or a governance wording issue. Next audit on or after 2026-11-01.

**Critical lane:** research

(Ordered list read by both tier workers, which then fall back to `functions`. The current milestone is M5.
- research: T-2648 (gen-m5 sweep, opus), then the queued M5 probes T-2517 and T-2529. These are the only M5 tasks outside the functions lanes.
- The M5 exit work (8 probe cells) is in the functions lanes: T-2627 (functions-text), T-2441 (functions-text), T-2523 (functions-math) and T-2388 (functions-text, text-format-digits-date-token). The workers reach them through `functions`.
- engine and harness have no M5 work; quality and api have no M5 exit work. perf stays out: Phase 2.)

## Needs owner

- **Direct pushes to `main`** (2026-10-18, continuing 2026-10-19). Session `01Xbmds2…` has now pushed fifteen T-1778 commits straight to `main` with no PR (6f85cc9 … 892a53ca reported last run; fc3f993d, c00142bb, 0b5ce7a4, 36217c63, 756f849f, 37d303dd, 32ac084d and c763916b since), so the reviewer agents never saw them. CI passed on each and nothing needs reverting. New this run: **32ac084d edits a protected path, `.claude/skills/excel-probe/SKILL.md`**, without the decision record the governance gate demands of a PR. It is a two-line pointer to the new `docs/research/excel-kernel-playbook.md`, and both read as benign research guidance. Still, agent instructions changed with no review. Recommended fix: a branch ruleset on `main` that requires a pull request (the steward and workers already merge through PRs, so nothing changes for them). If these pushes were your own interactive work, remove this item. Update 2026-10-20: no direct push to `main` since the last run; all 109 commits came through PRs. Update 2026-10-25: still none; all 221 commits since 2026-10-22 came through PRs.
- **Missed steward runs** (2026-10-25). There is no steward snapshot or PR for 2026-10-23 or 2026-10-24, so this snapshot covers three days. Please check that the daily steward routine fired on those days.
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
