# Status

The steward routine owns the snapshot and the critical-lane line and rewrites them daily; agents may
still add or remove "Needs owner" entries. PRs no longer append log lines (decision 0024): the record
of merged work is `git log --first-parent main` plus each task's Outcome, and the steward lists what
merged since its last run.

## Snapshot — 2026-10-21 (M5 is current)

| Suite | Cases | Cells | Bit-exact | Known failures |
|---|---|---|---|---|
| ironcalc-calc | 219 | 72,223 | 98.77% | 890 (was 1,056) |
| poi | 336 | 27,205 | 99.57% | 131 (was 462; this run recorded 1 unrecorded fix, `EverythingTests!I1360`) |
| gen-m5 | 65 | 22,400 | 99.26% | 165 (new: T-2114 gated it) |
| gen-m2 | 30 | 12,000 | **100%** | 0 |
| gen-m1 | 15 | 3,750 | **100%** | 0 |
| gen-m10 | 20 | 1,690 | 96.80% | 54 (was 78); displayed text 1,347 |
| probes | 1,101 | 71,315 | 97.77% (8 accepted deviations; 3 cases pending) | 1,599 (was 1,666) |
| smoke | 6 | 46 | 100% | 0 |
| fixtures | 17 | 248 | 96.37% | 9 |
| corpus-bench | 912 | 117,121 | 99.58% | 14 (see the timeout note below) |
| gen-m6 | 25 | 1,000 | 100% | not gated |
| gen-m7 | 25 | 994 | 99.90% | not gated |
| gen-m8a | 12 | 3,577 | 100% | not gated |
| gen-m8b | 9 | 3,368 | 91.78% | not gated |
| gen-m9 | 25 | 998 | 99.10% | not gated |
| probes-numeric | 167 | 115,809 | 75.58% (99.72% within 1e-9) | not gated |

`make eval` on `main` (7dab5361) shows 0 regressions in every gated suite. poi and corpus-bench were fetched for this run. After `main` was merged into this branch, `make eval-lower-visible-lock` counted 1,809 visibly-off cells, so this PR lowers the lock from 1,842 to 1,809 (decision 0093).

The run shared a 4-core machine with four `make check` runs (the orphan-PR updates below), at a load average of about 15. Under that load, 9 corpus-bench workbooks and `probes/union-walk-repeated-in-lambda` hit the 60 s case timeout. `make eval-baseline` wanted to record them as `!Timeout` known failures. I did not commit those lines because they are load, not engine changes. A later rerun on a quiet machine (`make eval-lower-visible-lock`) recorded no timeouts. The probe case passes on its own (15/15) but takes about 36 s of CPU, so a contended CI runner could time it out.

CI on `main` is green at 7dab5361. No CI run on `main` failed since the last snapshot; about 15 were cancelled by the next push. The nightly fuzz run (2026-10-21) passed.

**M2's exit is no longer fully met: M0–M2 probes (decision 0051) have 12 known failures** (436 cases, 12,555 cells, 12,539 bit-exact, 4 deviations).
- All 12 are in `probes/subnormal-cell-constants`, a case T-2016 (#1397) added today. It is new coverage, not a regression. A subnormal number constant in a loaded workbook reads as 0 in Excel.
- T-2228 (engine, sonnet) owns it, and #1422 for it opened during this run. This run raised it from P3 to P1, and engine leads the Critical lane until it lands.
- The frozen m1-probes list is still at 0. ironcalc-calc's M2 row is unchanged: 1 failing cell, the decision 0074 exception.

Where M5's exit stands ("Milestone exit (decision 0085)" in the report):
- Catalog coverage: 135/135. Met.
- gen-m5: 22,400 scored cells at 0.99263, 165 known failures against an allowance of 22. **Not met.** The planner checked every failing cell this run. All 165 have an open M5 owner:
  - largest groups: T-2169 (SINH/COSH/EXP overflow, 37 cells), T-2151 and T-2152 (NUMBERVALUE), T-2240 (expm1 ulps);
  - also T-2150, T-2153, T-2155–T-2162, T-2166, T-2167, T-2173, T-2147, T-2067, T-2199 and T-1735.
- ironcalc-calc cumulative M5 row: 15 failing, 10 after exit exceptions. **Not met.** All 10 are SINH/TANH/COTH/CSCH below 1, owned by T-2240 (Excel's expm1, the T-2134 follow-up).
- M5 probes: 208 cases, 8,977 cells, 54 known failures. **Not met** (it was 0 last run; new probes since). Owners:
  - T-2167: MROUND, 38 cells;
  - T-2190: INFO, 14 cells;
  - T-2137: INDEX reference arguments, 2 cells.

Hygiene: lint-debt 91 allow entries (= last run) · duplicated lines 3,147 (was 3,268) · oversized files 11 (= last run) · evidence floor: 3 in-scope functions under 20 Excel cells (= lock).

Since the 2026-10-20 snapshot (steward PR #1287), 109 commits landed on `main` through 7dab5361, touching 102 task ids. None was reverted.
- All 109 were squash-merged through PRs on GitHub (committer GitHub). Every task commit matches a merged PR, including the 16 whose titles carry no `(#n)`. No direct pushes to `main`.
- Task ids merged:
  - T-0060, T-0061, T-0442, T-0443, T-0908, T-0915, T-0917, T-0929, T-0934, T-0936, T-0937, T-0961, T-0969 (attempts 4–10), T-0970, T-0980, T-0982
  - T-1333, T-1592, T-1639, T-1640, T-1711, T-1730, T-1740, T-1828, T-1838, T-1843, T-1844, T-1846, T-1855, T-1869, T-1870, T-1941, T-1962, T-1964, T-1975, T-1991, T-1993
  - T-2015, T-2016, T-2032, T-2034, T-2048, T-2056, T-2059, T-2062, T-2077, T-2089, T-2092, T-2098, T-2100, T-2108–T-2116, T-2120, T-2123, T-2127–T-2129, T-2132–T-2134, T-2136, T-2138, T-2142, T-2148, T-2149
  - T-2154, T-2163–T-2165, T-2167, T-2168, T-2170, T-2171, T-2174–T-2179, T-2181, T-2183, T-2187, T-2193, T-2201, T-2204–T-2208, T-2210, T-2219, T-2223–T-2225, T-2229
  - Nine task-only queue refills: #1320, #1325, #1329, #1340, #1360, #1365, #1388, #1389, #1409.
- Every M5 harness task filed last run (T-2108–T-2113) and the research tasks T-2114–T-2116 are done. gen-m5 now has 22,400 Excel-backed cells and is gated.

Blocked: 29 (28 last run). None can be unblocked.
- New: T-2182 (desktop evidence for INFO type_text, M14, decision 0039).
- The rest are unchanged: Phase 2 (T-0400, T-0401, T-0402, T-0423, T-1066, T-0441), time-boxed fits with no new hypothesis (T-0968, T-1528, T-1529, T-1592; T-2187 is the oracle sweep T-1592 needs), T-1099/T-1284 (GEOMEAN), T-0431 (FUSE corpus), T-2037 (needs a routine change) and the M14 desktop-oracle tasks (decision 0039).

Open PRs: 20 at 08:10 UTC. Seven agent PRs were open longer than 24 h at the start of the run: #1209 (T-1996), #1210 (T-1943), #1212 (T-1878), #1230 (T-1835), #1284 (T-2101), and the drafts #1226 (T-0439) and #1245 (T-1484).
- #1209, #1212 and #1284: this run merged `origin/main` into each, taking `main`'s side of baseline conflicts as decision 0093 says, and then ran all four reviewers on each diff. Every reviewer said APPROVE, and CI went green.
- Session `01HP2RnX…` held the merge lock and merged all three between 07:49 and 08:02 UTC (#1284, #1209, #1212), so this run merged none itself.
- #1230 (T-1835) conflicted with `main` in the lift code. Once merged, it raised duplication by 4 lines, from repeated WRAPCOLS/WRAPROWS argument reads. I deduplicated those reads, but a live session pushed its own merge at 07:17 UTC, so my version was not pushed. Whoever lands #1230 must still pass the duplication ratchet.
- Duplicates: T-1943 is in both #1210 (older) and #1347 (newer, closer to `main`). #1347 survives. Close #1210 with a link once #1347 has merged (decision 0050).
- Decision 0093 (T-2245, #1414) landed during this run. Fix PRs no longer write the visible-parity lock, and this steward PR lowers it once a day. Measured on this branch after merging `main`, the lock goes from 1,842 to 1,809.

Queues at the start of the run:
- functions: 8 ready (≥ 3), so the gap-triager did not run (a worker already triaged today, #1388/#1409). The sonnet buffer is 29 (≥ 10).
- M5 ready by lane: engine 1, harness 0, research 0, quality 0, api 0.
- The planner ran for M5:
  - It filed T-2259 (harness, opus): `make exit-owners`, which lists failing exit cells that no open task owns. The planner found 13 by hand.
  - It linked those 13 cells to T-2240.
  - Research shows 0 ready only because T-2199 holds the lane. Seven M5 research tasks are queued. Engine M5 work is all in T-2137, so it filed nothing there.

Upstream IronCalc: three new commits under base/, xlsx/ or bindings/python.
- abd55b64 moves xlsx to zip 9.0.5. We are already on zip 9 (9.0.3).
- 4e6db1e6 adds a malformed-ZIP error-kind test.
- 179d0221 adds license files only.
- Filed T-2244 (quality, sonnet): port the test and take zip 9.0.5. UPSTREAM.md is triaged through 179d0221.

Oracle: excel-graph credentials are present (`rustsheet-oracle status`: excel-graph OK).
- probes: 0 written, 1 failure. `typed-if-one-argument` (T-1990) is still refused with 501 `FileCorruptTryRepair`. The other 2 pending cases are the unfillable M10 locale cases.
- ironcalc-calc: 0 written, 216 skipped (Excel-authored).
- poi's 205 pending cases are web-vs-desktop conflicts (decision 0019), so they were not refilled.

Spot-audit: the last one was 2026-10-16, so none is due (next on or after 2026-10-23).

**Critical lane:** engine, harness, research

(Ordered list read by both tier workers, which then fall back to `functions`. The current milestone is M5.
- engine: T-2228 puts the M0–M2 probes back at 0 (sonnet, P1). T-2137 owns the M5 INDEX reference-argument probe cells.
- harness: T-2259 (exit-owner report) keeps every M5 exit cell owned.
- research: T-2199, T-2198, then the queued M5 probes (T-2067, T-2143, T-2172, T-2189, T-2200, T-2202, T-2203).
- Most of the M5 exit gap sits in the functions lanes (T-2169, T-2151/T-2152, T-2240, T-2167 and the rest above), which the workers reach through `functions`.
- quality (T-2244) and api have no M5 exit work. perf stays out: Phase 2.)

## Needs owner

- **Direct pushes to `main`** (2026-10-18, continuing 2026-10-19). Session `01Xbmds2…` has now pushed fifteen T-1778 commits straight to `main` with no PR (6f85cc9 … 892a53ca reported last run; fc3f993d, c00142bb, 0b5ce7a4, 36217c63, 756f849f, 37d303dd, 32ac084d and c763916b since), so the reviewer agents never saw them. CI passed on each and nothing needs reverting. New this run: **32ac084d edits a protected path, `.claude/skills/excel-probe/SKILL.md`**, without the decision record the governance gate demands of a PR. It is a two-line pointer to the new `docs/research/excel-kernel-playbook.md`, and both read as benign research guidance. Still, agent instructions changed with no review. Recommended fix: a branch ruleset on `main` that requires a pull request (the steward and workers already merge through PRs, so nothing changes for them). If these pushes were your own interactive work, remove this item. Update 2026-10-20: no direct push to `main` since the last run; all 109 commits came through PRs.
- **Decision 0081 says the owner asked for it** (2026-10-20, T-2107, #1283, merged 06:13 UTC as 4d9b7754). It turns the performance reviewer off and lets a bench criterion such as "within 5%" mean up to 50% slower than `main`. That loosens a gate, and the only evidence that you asked is the PR text ("the owner, who asked on 2026-10-20"). If you asked, remove this item. If not, revert 4d9b7754.
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
