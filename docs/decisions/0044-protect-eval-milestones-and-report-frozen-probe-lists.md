# 0044 — Protect eval/milestones and report frozen probe lists

- **Status:** accepted (2026-10-08, T-1146)
- **Context:** Decision 0043 froze M1's probe criterion as `eval/milestones/m1-probes.txt` and said
  a PR that edits `eval/milestones/` needs a decision record, but `scripts/check_governance.py`
  did not enforce it, and the steward counted the list's known failures with an `awk` one-liner.
- **Decision:** `eval/milestones/` is a protected path in `scripts/check_governance.py`. The eval
  harness parses every `eval/milestones/*.txt` list (comments, blank lines, no duplicates), fails
  `run` and `check` when a list names a case that its suite does not discover, and reports per list
  the cases, scored cells, bit-exact cells, deviations, known-failure cells (counted as the
  baseline counts F lines) and failing case ids, in report.md and report.json. The steward skill
  reads the M1 criterion from that report instead of the `awk` count.
- **Consequences:** Editing a frozen list fails the governance gate without a decision record. A
  list line that goes stale (a case renamed or deleted) fails `make eval-check` at once, naming the
  line. The lists stay an exit measure, not a pass-rate gate (decision 0043).
