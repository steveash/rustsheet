# Status

The steward routine owns the snapshot and the critical-lane line and rewrites them daily; agents may
still add or remove "Needs owner" entries. PRs no longer append log lines (decision 0024): the record
of merged work is `git log --first-parent main` plus each task's Outcome, and the steward lists what
merged since its last run.

## Snapshot — 2026-10-10 (M1 in progress)

| Suite | Cases | Cells | Bit-exact | Known failures |
|---|---|---|---|---|
| ironcalc-calc | 217 | 72,203 | 97.86% | 1,547 |
| poi | 336 | 27,290 | 97.97% (204 cases pending: no formulas, unreadable, or web-vs-desktop conflicts) | 568 |
| gen-m2 | 30 | 12,000 | 99.62% | 46 |
| gen-m1 | 15 | 3,750 | 99.79% (M1 exit needs 100%; 8 cells left, all power-operator last-digit: 6 in T-1302, 2 in T-0221) | 8 |
| probes | 372 | 11,225 | 95.08% (4 accepted deviations, 11 cells passed by decision 0042) | 552 |
| smoke | 6 | 46 | 100% | 0 |
| fixtures | 6 | 67 | 100% | 0 |
| gen-m5 | 10 | 400 | 97.75% (not gated; the T-1197 timeout is fixed, so all 10 cases now run) | not gated |
| probes-numeric | 27 | 10,129 | 53.82% | not gated |

`make eval-check` on `main` (32dec3cb) shows 0 regressions and 0 baseline fixes in every suite. CI on `main` is green (last completed run, 74a6f3b; the run on 32dec3cb was still in progress).
Probes known failures went 558 → 552 with 19 new probe cases since the last snapshot (new coverage, not regressions).
Frozen M1 probe list (`eval/milestones/m1-probes.txt`): 89 cases, 1,858 cells, 1,854 bit-exact, 4 known failures, all in error-stored-constants (the stored #BUSY! kind, T-1222, open as #528). number-text-exponent-digits (T-0938), paren-number-text-forms (T-0282) and multiarea-printarea (T-1193) are fixed.

Hygiene: lint-debt 99 allow entries · duplicated lines 6,829 (ratchet file still 7,158) · oversized files 15 (ratchet file still lists 16).
`quality/` is protected, so the next PR that touches it records the drops.

Since the 2026-10-09 snapshot (89cb400), 33 PRs merged to `main` through 32dec3cb, all tasks. None was reverted.
- T-0282, T-0292, T-0295, T-0938
- T-1020, T-1090, T-1158, T-1177, T-1178, T-1193, T-1197, T-1198
- T-1200, T-1209, T-1210, T-1212, T-1213, T-1215, T-1216, T-1218, T-1220, T-1221, T-1224, T-1226
- T-1231–T-1234, T-1238, T-1240, T-1243, T-1245, T-1246

Blocked (13, unchanged): the Phase 2 tasks T-0400/T-0401/T-0402 and T-1066 (decision 0027); T-1099 (GEOMEAN last ulp);
the M14 desktop-oracle tasks T-0171, T-0198, T-1016, T-1051, T-1074, T-1075, T-1081, T-1174 (decision 0039).
T-1099's dependency T-1108 is done, but it found no reproducible model of Excel's root, so the task stays blocked; T-0288 (#508) owns the remaining one-ulp GEOMEAN misses.

**Nightly fuzz is red, with one new finding.**
- Run 38022063246 (2026-10-10, 3d755e8) first failed on all three targets within 2 s. No runner was ever assigned (runner id 0, no steps, no logs). I re-ran the failed jobs once.
- On the re-run, `formula_parse` and `xlsx_import` passed, so the T-1220 RSS fix holds.
- `formula_eval` panicked after 94k runs on `RANK(5,IF(A1:C1>0,A1:CWWW…W1))`. This is the same "Larger-than-1x1 array reached scalar-context cell" debug_assert as T-0278 and T-0296 (now `model.rs:988`), reached through RANK with a non-reference array `ref`.
- I reproduced it on 32dec3cb and narrowed it to `=RANK(5,IF(A1:C1>0,A1:XFE1))`. Filed as T-1303 (P0, functions-stat). T-0296's AGGREGATE fix (#501) does not cover it.

Open PRs: 32. Five are over 24 h old:
- #330 (T-0411, about 58 h old) is green and approved. Yesterday's blocker, the oversized-file ratchet on `functions/mod.rs`, is gone after `main` moved a test out of that file. I merged `main` into the branch (9dc3c83e) and `make check` passed locally. I commented on the PR and did not merge it, since it belongs to another session.
- #461 (T-0953, about 26 h old) is green and approved, but it was about 70 commits behind. I merged `main` into the branch (9884daf) and kept both sides of a conflict in a formatter test. `make check` passed locally. I commented on the PR and did not merge it.
- #314 (T-1100), #409 (T-1155) and #436 (T-1186) are drafts whose tasks are blocked on their branches, with reasons recorded. T-1186 waits on T-1187 (research). No action.
- The other 27 PRs are less than 24 h old. #528 (T-1222) carries the last 4 cells of the frozen M1 probe list.

Queues at the start of the run:
- functions: 21 ready (18 later in the run, after workers claimed tasks). The sonnet buffer was 13 and fell to 9 during the run.
- engine, harness, research and api: 0 ready. engine and research still have unclaimed M1 todo tasks, but their lanes are busy. quality 7, perf 7.
- The planner ran for M1. It filed T-1302 (engine, sonnet, P1): `^` and POWER with integer exponents use right-to-left square-and-multiply. That rule is already VERIFIED in numbers.md, and it clears 6 of the 8 gen-m1 cells. The planner also narrowed T-0221 to non-integer exponents, whose last 2 gen-m1 cells are A41 and A207, and confirmed T-1222 as the owner of the last 4 probe-list cells. It found no M1 scope left for harness, research or api.
- The gap-triager ran because the sonnet buffer fell below 10 (decision 0021). It filed five sonnet tasks, and the buffer is back to 12 ready:
  - T-1304 (functions-lookup): MATCH with a fractional match_type between -1 and 0 (2 gen-m2 cells).
  - T-1305 (research): MATCH cells that the documented search rule does not explain.
  - Three evidence-gap probes under decision 0031: T-1306 (CODE/UNICHAR edge code points), T-1307 (CLEAN/PROPER) and T-1308 (ISOWEEKNUM/WEEKNUM at year boundaries).
  - Every other M1/M2 failing-cell group already has a task.

Upstream IronCalc: 1 new commit past 6a1d35a6, now triaged through 8a82396a. 8a82396a (ECMA.CEILING evaluated like CEILING) became T-1301 (functions-math, sonnet, P3, probe first: the function is not in our catalog).

Oracle: excel-graph OK. The fill on ironcalc-calc wrote 0 sidecars (214 skipped). The fill on poi wrote 0 (292 skipped, the same 4 workbook failures as T-0019).

Spot-audit: none this run (not yet due). The last audit, 2026-10-09 (steward PR #466), re-reviewed #464, #448 and #453 on opus; its misses were filed as T-1223, T-1224, T-0298 and the T-1216 priority raise. The next audit is due 2026-10-16.

**Critical lane:** engine, research, harness

(Ordered list read by both tier workers, which then fall back to `functions`.
- M1 exit needs gen-m1 at 100% and the frozen M1 probe list at 100%.
  - gen-m1 has 8 last-digit cells left: 6 in T-1302 and 2 in T-0221.
  - The probe list has 4 cells left, in T-1222 (#528).
- All of that work is in engine. research holds the remaining M1 probes (T-1131, T-1181, T-1187, T-1250).
- harness has no M1 exit scope left (T-1096 is in progress), so it now comes after research.
- api is off the line: no M1 api scope is left.
- The fuzz P0s T-0296 (functions-math) and T-1303 (functions-stat) are reached through `functions`, P0 first. perf stays out: Phase 2.)

## Needs owner

None. T-0040 (function catalog) is done. excel-desktop needs are not owner items (decision 0039): T-1016, T-1051 and the other desktop tasks wait in M14 (Phase 4), tracked by T-1074 and T-1075.

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
