# Status

The steward routine owns the snapshot and the critical-lane line and rewrites them daily; agents may
still add or remove "Needs owner" entries. PRs no longer append log lines (decision 0024): the record
of merged work is `git log --first-parent main` plus each task's Outcome, and the steward lists what
merged since its last run.

## Snapshot — 2026-10-16 (M2 is current)

| Suite | Cases | Cells | Bit-exact | Known failures |
|---|---|---|---|---|
| ironcalc-calc | 217 | 72,203 | 98.07% | 1,396 |
| poi | 336 | 27,290 | 98.23% (204 cases pending: no formulas, unreadable, or web-vs-desktop conflicts) | 499 (was 512; this PR records the 13 fixed cells) |
| gen-m2 | 30 | 12,000 | **100%** | 0 |
| gen-m1 | 15 | 3,750 | **100%** | 0 |
| probes | 696 | 29,856 | 81.64% (4 accepted deviations, 11 cells passed by decision 0042) | 5,485 |
| smoke | 6 | 46 | 100% | 0 |
| fixtures | 16 | 237 | 96.20% | 9 |
| gen-m5 | 10 | 400 | 99.25% | not gated |
| gen-m6 | 25 | 1,000 | 99.50% | not gated |
| gen-m7 | 25 | 994 | 96.18% | not gated |
| gen-m8a | 12 | 3,577 | 83.20% | not gated |
| gen-m9 | 25 | 998 | 92.18% | not gated |
| probes-numeric | 133 | 17,119 | 55.34% | not gated |

`make eval-check` on `main` (a18a7329) shows 0 regressions in every gated suite.
- poi has 13 baseline fixes. CI does not fetch poi, so nobody had recorded them; this PR does, with `make eval-baseline`.
- poi did not run on the first `make eval` because the container had not fetched it (`rustsheet-eval fetch --suite poi`). The numbers above are from the second run.

CI on `main` is green at a18a7329.
- One run failed since the last snapshot: 1aebd11c, in the hygiene job's `cargo deny`, because zip 0.6.6 was yanked.
- The next commit, T-1668 (de25e70c, zip 9.0.3), fixed it, and every later run passed.

Where M2's exit stands:
- gen-m2: **100%, criterion met** (T-1495 fixed the last cell).
- ironcalc-calc cells using only M1/M2 functions: 3,977 of 3,977, criterion met.
- The frozen m1-probes list is at 0 failures.
- M0–M2 probes: 329 cases, 9,435 cells, 9,402 bit-exact, 4 deviations, **29 known failures** (was 170). Every failing cell has an open M2 task:
  - `text-collation-longer` (9): T-1669 (functions-text).
  - `lotus-dot-range` (6): T-1659 (engine).
  - `r1c1-let-name-r0c1`, `-r1c1`, `-r5c7` (1 each): T-1658 (engine).
  - `text-lone-surrogates` (6): T-1656 (functions-text).
  - `vara-single-ref-picked` (3): T-1291 (functions-stat).
  - `legacy-index-array-index` (2): T-1616 (functions-logic).
  - The planner found that the last three tasks were filed in M10, M9 and M5, although their cases count toward M2 (decision 0051). This PR moves T-1656, T-1291 and T-1616 to M2.
- Criteria matching has one implementation: met.
- Aggregate-argument iteration is not fully met. `logical_nary` (AND, OR, XOR) still has its own range walk, array loop and first-error rule; the planner filed **T-1677** (quality, opus).
- Duplication is at 6,328 lines, 30.9% below the fork's 9,153 (target ≥ 15%), so the number is met.

Hygiene: lint-debt 91 allow entries (= last run) · duplicated lines 6,328 (ratchet 6,360) · oversized files 14 (ratchet 15).
Both ratchets can tighten. The steward left them as they were, because `quality/` is a protected path; the next quality PR should run `--update`.

Since the 2026-10-15 snapshot (steward PR #821, de2fedc8), 76 PRs merged to `main` through a18a7329: 75 task PRs and 1 task-filing PR (#834). None was reverted.
- T-0056, T-0265, T-0294, T-0428, T-0429, T-0958
- T-1049, T-1190, T-1191, T-1219, T-1249, T-1259, T-1262, T-1265, T-1359
- T-1438, T-1476, T-1495, T-1516, T-1543, T-1545, T-1552, T-1563, T-1567
- T-1570–T-1574, T-1578–T-1581, T-1584–T-1587, T-1589, T-1593, T-1595–T-1597
- T-1601–T-1604, T-1606, T-1609, T-1612, T-1615, T-1617–T-1619, T-1627, T-1629
- T-1633, T-1635, T-1636, T-1638, T-1641–T-1643, T-1647–T-1650, T-1652, T-1653, T-1655
- T-1660–T-1662, T-1664, T-1666, T-1668

Blocked: 24 (= last run, the same tasks). None can be unblocked.
- The Phase 2 tasks T-0400, T-0401, T-0402, T-0423 and T-1066 (decision 0027).
- T-1099 (GEOMEAN last ulp).
- Time-boxed last-digit fits (decision 0056) with no new hypothesis: T-0968, T-1528, T-1529, T-1592.
- The M14 desktop-oracle tasks T-0171, T-0198, T-1016, T-1051, T-1074, T-1075, T-1081, T-1174, T-1204, T-1277, T-1299, T-1310, T-1367 and T-1553 (decision 0039).

**Nightly fuzz:** run 38847367745 (2026-10-16, 1aebd11c) failed on `formula_eval` with an out-of-memory abort.
- The input is `PERCENTRANK.INC(SE:RCH,"İİx",2)`.
- I reproduced it on a18a7329 with a release facade test: `memory allocation of 3019898896 bytes failed`.
- `PERCENTILE.INC(SE:RCH,…)` and `QUARTILE(SE:RCH,…)` abort the same way. MEDIAN and SUM do not.
- Root cause: `values_from_range` (`statistical/chisq.rs`) allocates one entry per cell of a whole-column range, unclipped.
- Filed **T-1681** (functions-stat, P0, opus).
- `formula_parse` and `xlsx_import` passed.

Open PRs: 5, the same as at the end of the last run, apart from #819 (merged).
- Under 24 h: #891 (T-1316).
- Blocked drafts, each with a `status: blocked` task on its branch: #698 (T-1486), #553 (T-1284), #436 (T-1186), #409 (T-1155).
  - None of their branches moved since the last run.
  - #698's blocker, commented on the PR on 2026-10-15, still holds: `functions/text/common.rs` is over the 1,000-line limit after a `main` merge.
- No orphan PR met every decision 0050 condition, and none is a duplicate, so none was merged or closed.

Queues at the start of the run:
- functions: 104 ready. The sonnet buffer is 63, so the gap-triager did not run.
- engine 0, api 0, harness 0, quality 0, research 0, perf 16. The planner ran for M2:
  - engine and research are busy, not empty. Their branches moved in the last 2 h (T-1657, T-1665), and they hold unclaimed M2 tasks (engine T-1439, T-1535, T-1554, T-1658, T-1659; research T-1670).
  - The planner filed **T-1677** (quality, opus), the AND/OR/XOR walker described above.
  - api and harness have no M2 work.

Upstream IronCalc: 2 new commits through 052c619a.
- 297553dc only bumps the version to v0.9.0.
- 052c619a adds Python-binding API for themes, colors and merged cells.
- Neither is an engine fix, and our bindings sit on API v0, so I filed no task. UPSTREAM.md is triaged through 052c619a.

Oracle: excel-graph credentials are present.
- The fill on ironcalc-calc wrote 0 sidecars (214 skipped).
- The fill on poi wrote 0 sidecars (292 skipped, the same 4 workbook failures as T-0019).
- No other suite has pending cases.

Spot-audit 2026-10-16 (last was 2026-10-09). 408 task PRs merged since then. A seeded shuffle picked #871 (T-1609), #500 (T-1252) and #606 (T-1415), and every applicable reviewer re-ran on `model: opus`. Every verdict was APPROVE, with no blocking findings and no real defects. The original reviews missed these:
- **#606 (T-1415, CI docs-only skip, protected path):**
  - The PR body has no agent-review table. I found no record that any reviewer ran before the merge, although decision-0049 governance needs reviewer-architecture and reviewer-security to approve.
  - The merge came from an owner-directed session (`claude/` branch), not a work-loop worker.
  - The opus re-review (security, architecture, quality) approves it now. It found the same MINOR from all three: the docs-only classifier is untested inline shell, and nothing enforces registering new `docs/` readers.
  - Filed **T-1680** (harness, opus).
- **#500 (T-1252):**
  - The original quality reviewer (sonnet) returned REQUEST_CHANGES. Its fixes landed in a later commit that was never re-reviewed, and the PR merged anyway, against the merge policy (every reviewer APPROVE).
  - The opus quality re-review approves. It found two MINOR issues the sonnet quality reviewer missed:
    - T-1264 was filed in the wrong lane (moot: T-1264 and T-0270 are both done);
    - the H21 quirk is written out twice, in references.md and lookup.md.
  - Original architecture (sonnet) had noted the hard-coded picked-argument index, and no task was filed. Filed **T-1678** (quality, sonnet).
  - The opus fidelity reviewer also notes that lookup.md's "one result per element" is inferred from a SUM, not observed. That is MINOR, and T-1264's probe has since covered it.
- **#871 (T-1609):**
  - Architecture and quality (sonnet) and excel-fidelity (opus) all missed one point, MINOR, docs only: a references.md sentence still calls uncached names and sheets UNVERIFIED after this PR verified them, and "reversed span the same" overstates one SUM cell.
  - Filed **T-1679** (research, sonnet).

**Critical lane:** engine, quality, research, harness

(Ordered list read by both tier workers, which then fall back to `functions`. The current milestone is M2.
- engine: T-1658 and T-1659 hold 9 of the 29 M0–M2 probe failures.
- quality: T-1677 closes "one implementation of aggregate-argument iteration". T-1678 is an audit follow-up.
- research: T-1670 and T-1665 (M2 probes).
- harness: T-1680 (audit follow-up).
- The other 20 failing cells are reached through `functions`: T-1669, T-1656, T-1291 and T-1616. So is the P0 T-1681.
- api has no M2 work. perf stays out: Phase 2.)

## Needs owner

- **Sync the live steward routine prompt with docs/process/routines.md.** The live prompt (still so on 2026-10-16) opens with "On an up-to-date `main`". routines.md says instead to work on a branch cut from a freshly fetched `origin/main` and never to check out, pull into or reset a local `main` (decision 0055). With the old wording, the 2026-10-15 run first tried to reset a stale local `main`, and the 2026-10-16 run checked out and fast-forwarded local `main` before branching. Agents never edit routines (decision 0008).

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
