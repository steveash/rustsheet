# 0177 — mark-ready removes merge-train-ejected; the skills drop the manual step

- **Status:** accepted (2026-10-30, T-3155). It implements SPEC-0027 section 6, "After an ejection".
- **Context:** SPEC-0027 says the `merge-train-ejected` label "is removed when the PR is marked again", but
  `scripts/merge_ready.py mark` only added `ready-to-merge` and kept every other label. T-2989 bridged the gap with
  a manual step in the work-loop (step 6.3) and steward ("Merge train") skills: remove the label by hand after
  marking. A forgotten manual step leaves a ready PR labelled as ejected, which misleads the resuming run and the
  steward.
- **Decision:** `mark` removes `merge-train-ejected` itself. Direct mode deletes the label after adding
  `ready-to-merge`; step mode prints an `unlabel` action (the existing `merge_step.py` schema, decision 0169) after
  the `label` action, and both actions' `labels` set is the final one (`ready-to-merge` in, `merge-train-ejected`
  out). A PR already marked and labelled at its head that still carries the label gets the `unlabel` action alone.
  The manual-removal sentences go from both skills.
- **Consequences:** workers and the steward mark again after an ejection with `make mark-ready` and nothing else;
  a session in step mode performs one more printed action. No other label and none of the merger's writes change.
