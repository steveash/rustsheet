---
id: T-1519
title: "Hygiene gate: a newly added probes case must carry exactly one milestone tag, never M0 or M1"
milestone: M2
lane: harness
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A PR that adds a `probes` case fails the governance check unless the new case.json has exactly one
milestone tag, `M2` or later. New probes then stop landing on M2's exit row under a stale `M1` or
`M0` tag (or no tag at all), so the "probe milestone M0-M2" count in `make eval` measures M2's own
subjects only. Today it has to be cleaned up by hand after the fact (T-1470).

## Context
- Decision 0051, rule 4: a probe case takes its case.json milestone tag. Rule 5: an untagged case is
  M2. Under decision 0043 a non-frozen `M1` or `M0` case counts toward M2. Decision 0051 item 2
  says "New probe cases carry the milestone tag of the function or feature they probe, so they need
  no line". Nothing enforces that, and T-1470 only adds the rule to the excel-probe skill text.
- Measured on this tree (2026-10-14): 104 `eval/suites/probes/cases/*.case.json` files were added
  since decision 0051 was accepted (2026-10-12). 47 are tagged `M1` and 5 have no milestone tag
  (51 outside `catalog/` when diffed against decision 0051's merge commit a9380cb7). They
  include `yield-solver-sweep` (66 known failures, YIELD, M9), `date-system-1900-odd-years` and
  `-1904-odd-years` (26, ODDF*/ODDL*, M9), `date-system-1900-financial-readers`, `-shifted` and
  `date-system-1904-financial-readers` (53, COUP*, M9), `coupdays-feb-month-end` (4),
  `text-format-unquoted-letters` (75, number-format codes, M10) and `text-format-length-limit-k`
  to `-m` (M10). `make eval` now reports `probe milestone M0-M2 ... 518 known failures`, against 170
  at the 2026-10-13 run. Most of the increase comes from these tags.
- A post-freeze probe whose subject is M0, M1 or M2 counts toward M2 either way (decisions 0043
  and 0051), so requiring `M2` instead of `M0`/`M1` loses no evidence. It only removes the stale
  tag as a default.
- Where: `scripts/check_governance.py` already diffs the PR against its merge base (`git diff
  --name-status`) and runs in CI (`.github/workflows/ci.yml`, job "hygiene", step "governance") and
  in `make governance`. Add the check there, on `A` (added) lines only, so existing cases (whose
  tags are never edited, decision 0043) are never flagged. `probes/catalog/*` cases are exempt:
  decision 0051 rule 3 assigns their milestone per cell from the functions they call, and their
  generator tags them `M0`.
- `scripts/` is a protected path (`PROTECTED` in check_governance.py), so the PR needs a decision
  record (`make new-decision`). The decision record states the rule, citing 0043 and 0051.
- Tag syntax: a milestone tag is a `tags` element that matches `^M\d+$`, the same rule as
  `parse_milestone` in crates/rustsheet-eval/src/probe_milestones.rs. Use the same regex and do
  not import the Rust code.

## Acceptance criteria
- [ ] `scripts/check_governance.py` fails, and names each offending file and its milestone tags,
      when the diff adds an `eval/suites/probes/cases/**/*.case.json` outside `cases/catalog/`
      that has zero milestone tags, more than one, or a single `M0` or `M1` tag. Added cases with
      one tag of `M2` or later pass. Modified and deleted case files are not checked.
- [ ] The rule is a pure function, for example `probe_tag_problems(name_status_lines, read_tags)`,
      that `main` calls with the real diff and file reader. Unit tests in
      `scripts/test_governance.py` call it directly, without git, as the existing tests call
      `is_protected`. They cover an added `M1` case (fail), an added untagged case (fail), an added
      case tagged both `M2` and `M9` (fail), an added `M2` case (pass), an added `M9` case (pass),
      an added `catalog/` case tagged `M0` (pass) and a modified `M1` case (pass).
- [ ] A new decision record in `docs/decisions/` states the rule and cites decisions 0043 and 0051.
      `scripts/check_decisions.py` passes.
- [ ] Running `python3 scripts/check_governance.py a9380cb7` (decision 0051's merge commit) on the
      branch lists the 51 non-catalog probe cases added since then that are tagged `M0`/`M1` or
      untagged, as counted on 2026-10-14. The PR description pastes the count, which may be higher
      if more land first. This shows the check would have caught them. `python3 scripts/check_governance.py origin/main` on the PR itself
      passes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Remapping the existing wrongly tagged cases in `eval/milestones/probe-milestones.tsv`. That is
  T-1470, whose case list predates the 2026-10-13 additions named above.
- Editing any case.json tag (oracle input hashes, decision 0043).
- Checking that a tag matches the case's subject. That is a reviewer judgment (decision 0051 item
  2) and is not mechanical.
- A `tasks.py validate` rule against new `milestone: M1` probe tasks (planner rule, decision 0043).
