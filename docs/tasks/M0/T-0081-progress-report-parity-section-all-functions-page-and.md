---
id: T-0081
title: "Progress report parity section, all-functions page and docs/parity.md pointer"
milestone: M0
lane: harness
status: done
priority: P1
size: M
tier: opus
depends_on: [T-0068, T-0070]
spec: docs/specs/SPEC-0022-daily-progress-dashboard.md
---

## Goal
docs/progress.md shows parity per function with the glue-aware tiers: functions per tier, tiers
by category, attributed cells by closeness, and the suites. docs/progress/functions.md lists
every function. docs/parity.md becomes a pointer, so the old parity page is retired and links
to it still work.

## Context
- SPEC-0022, the `parity` row of "Page layout", and "docs/parity.md".
- Inputs: report.json's T-0068 fields (`tier`, `attribution`, `attributed`, `excel`,
  `workbooks`) and T-0080's `v15_checkpoint`.
- This folds in T-0066 (dropped). The old generator recomputed the Phase 1 share
  (`within15_share`, `PHASE1_SHARE`); this task reads `v15_checkpoint` instead.
- Module: `progress_report/parity.py` in the package from T-0070.

## Acceptance criteria
- [ ] The `parity` block shows:
  - functions per `tier`;
  - tiers by category, as a table and a stacked SVG;
  - attributed cells by closeness;
  - the suites table, with cases, cells, bit-exact, ≤ 4 ulp, visibly off, `workbooks`,
    timeouts, and known failures (non-comment lines of `eval/baseline/<suite>.txt`).
  Python derives no verdict, bucket, tier or share threshold. A report without the T-0068 fields
  or `v15_checkpoint` fails loudly.
- [ ] Phase 1 numbers come from `v15_checkpoint` (`at_rate`, `in_scope`, per-function rate and
      `meets_rate`), with the strict count labelled as information. `within15_share` and
      `PHASE1_SHARE` are gone (T-0066's criterion).
- [ ] docs/progress/functions.md is written: every in-scope function with category, milestone,
      tier, attribution, attributed and Excel cells, and its within-15-digit share.
- [ ] docs/parity.md and docs/parity/functions.md become short static pointers to
      docs/progress.md. The three docs/parity/*.svg files are deleted. The generator writes
      neither file. docs/README.md links docs/progress.md.
- [ ] Tests on a synthetic report cover the tier table, the category table and a missing T-0068
      field.
- [ ] This PR commits docs/progress.md and docs/progress/ from a fresh `make eval` (poi,
      corpus-bench and calipers fetched). The Outcome compares the tier counts with the
      assessment's (308 never visibly off, 127 below 99%).
- [ ] `make py-check` green; `make check` green; reviewers approve.

## Out of scope
- History and stale handling (T-0071).
- Goals and campaign (T-0072).
- Top-40 and evidence (T-0073).

## Outcome
- `progress_report/parity.py` now renders the `parity` block of docs/progress.md (between the
  bottom line and readiness) and `docs/progress/functions.md` plus three SVGs (`tiers.svg`,
  `tiers-by-category.svg`, `cells-by-closeness.svg`). The block shows the Phase 1 line and the
  strict information line from `v15_checkpoint`, functions per tier (report.json `tiers`, with
  the fallbacks), tiers by category (table and stacked SVG), attributed cells by closeness, and
  the suites with bit-exact, ≤ 4 ulp, visibly off, `workbooks` (scored / all exact / all within
  15 digits / any visibly off), timeouts and known failures.
- Known failures are the `F` lines of eval/baseline/<suite>.txt (`TF` displayed-text lines shown
  after "text"). The criterion said "non-comment lines", but the `C`/`TC` lines list every case,
  not failures; `F` lines match the counts docs/status.md prints.
- model.py reads `tier`, `attribution`, `attributed`, `excel`, `workbooks`, `tiers` and
  `v15_checkpoint` (`at_rate`, `in_scope`, `phase1_met`, `at_strict_rate`, per-function
  `rate_cells`, `rate_visible`, `meets_rate`) through small checked accessors, so a missing key is
  a `ProgressReportError` naming the task that added it (T-0068 or T-0080); `__init__.py` no
  longer catches KeyError/TypeError. The per-tier counts derived from scope.json are checked
  against report.json `tiers`, so a catalog that is not the run's fails loudly. `PHASE1_SHARE`,
  `PHASE1_BANDS`, `NEAR_SHARE` and `within15_share` are gone (T-0066's criterion).
- docs/parity.md and docs/parity/functions.md are static pointers; the three docs/parity/*.svg are
  deleted; the generator writes neither. docs/README.md links docs/progress.md.
- svg.py gained a `Palette` (the bucket palette is unchanged) and a tier palette validated with
  the dataviz validator in both themes (light worst adjacent CVD ΔE 15.3, dark 10.0; unscored in
  the neutral slate).
- T-0070 review findings folded in: `REGENERATE` is defined once (markdown.py); the judgment block
  order and headings live only in page.py `LAYOUT` (blocks.parse takes both name sets); `md_text`
  writes `&` as an entity first and escapes `\`; the Makefile target removes `tasks.json.tmp`
  when `tasks.py stats` fails; `CODE_READ_PATHS` file entries now match only the exact file
  (directories keep prefix matching; check_docs_readers.py uses the same rule), recorded in
  decision 0120 items 1, 2 and 4; decision 0079's status says "Amended by 0120".
- First committed page: a fresh `rustsheet-eval run` (all 17 suites, poi, corpus-bench and
  calipers fetched; 0 skipped) at `05703c6c` (this branch's claim commit, T-0070 head plus the
  claim), then `make progress-report`. The judgment blocks hold their placeholders.
  - Phase 1: not met, 324 of 506 at rate (attributed); information: 81 of 506 (strict).
  - Tiers: exact 301, ≤ 4 ulp 25, 15 digits 4 (**330 never visibly off**), ≥ 99.9% 7, ≥ 99% 72,
    **< 99% 97**, unscored 0; 3 fallbacks.
  - Against the assessment's 308 never visibly off and 127 below 99%: T-0068's Outcome measured
    the same code at the assessment's commit 2c52346 as 301 and 129, so about 2 to 7 functions of
    the gap are the assessment's one-off counting method. The rest is the 119 commits since
    2c52346 (T-0068 lists the functions that moved, mostly out of `below99`). Today's 330 / 97
    is within 1 of T-0068's 330 / 98 at 22588cf.
  - 1,433,656 attributed Excel cells (76.41% bit-exact, 10.56% visibly off), against the
    assessment's "969k". The assessment's script is not in the repo, so this gap cannot be
    split; this count includes every Excel-evidence suite (probes-numeric's 611,860 cells among
    them) and leaves out corpus-bench.
  - calipers workbooks: 171 scored, 137 all exact, 153 within 15 digits, 18 with a visible cell.
- Filed T-0082 (harness, P3) from the T-0080 review: unowned, unattributed visibly-off cells
  (decision 0118 item 6) and a `run.dirty` flag.
