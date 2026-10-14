# 0057 — New probe cases carry one milestone tag, M2 or later, enforced by the governance check

- **Status:** accepted (2026-10-14, T-1519). It adds a gate and weakens none.
- **Context:** Decision 0051 makes a probe case count toward the milestone in its case.json tag
  (rule 4; an untagged case is M2, rule 5), and its item 2 says new probes carry the tag of the
  function or feature they probe. Under decision 0043 a post-freeze `M0` or `M1` case counts
  toward M2. Nothing enforced item 2. Since 0051 was merged (a9380cb7), 53 non-catalog probe
  cases have been added tagged `M0`/`M1` or untagged, as `scripts/check_governance.py a9380cb7`
  now lists. They include `yield-solver-sweep` (YIELD, M9) and `text-format-unquoted-letters`
  (number-format codes, M10). `make eval`'s "probe milestone M0-M2" count rose from 170 to 518
  known failures, mostly from these tags, and T-1470 has to remap them by hand.
- **Decision:** `scripts/check_governance.py` (CI job "hygiene", `make governance`) fails a PR
  that adds an `eval/suites/probes/cases/**/*.case.json` outside `cases/catalog/` unless the case
  has exactly one milestone tag (a `tags` element matching `^M\d+$`, as `parse_milestone` reads
  one) and that tag is `M2` or later. A probe of an M0–M2 subject is tagged `M2`; it counts
  toward M2 either way (decisions 0043, 0051), so no evidence is lost. Catalog cases are exempt:
  rule 3 of 0051 assigns their milestones per cell. Modified, renamed and deleted cases are not
  checked, because their tags are never edited (decision 0043).
- **Consequences:** A worker that writes a probe tags it with the milestone of its subject, or
  CI fails and names the file and its tags. The rule is a pure function,
  `probe_tag_problems`, unit-tested in `scripts/test_governance.py`. Whether the tag matches
  the subject stays a reviewer judgment (0051 item 2). The existing wrongly tagged cases are
  remapped by T-1470, not by this check.
