# 0212 — The merge train reads a CI run as red at its first failed job

- **Status:** accepted (2026-10-31, T-3531; the owner asked for it)
- **Context:** the train waited for a train CI run to complete before reading it red. On
  2026-10-31 the first live train after the re-flip (decision 0208) ran four red runs while it
  bisected #2581. In each, the eval job failed after about 5 min while `rust (test 2/2)` ran on
  for another 3-4 min, and the train ran out of its 40 min hold before it could merge the four
  green PRs.
- **Decision:** `merge_ready.ci_state(..., early=True)`, which the train uses, reads a run still in
  progress as RED as soon as one job of that run's check suite has concluded `failure`. Only
  `failure` counts; `cancelled` and `timed_out` keep their infrastructure meaning and are read
  when the run completes. A PR head's readiness check (`mark-ready`, V2) is unchanged.
- **Consequences:** a red train run costs the time to its first failure, not to its slowest job,
  which saves 3-4 min per red run and more in a bisection. The run is not cancelled; it finishes
  on its own while the train moves on.
