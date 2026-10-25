# 0102 — Pin CI runners to ubuntu-24.04

- **Status:** accepted (2026-10-23, T-2520)
- **Context:** Every CI job ran on `ubuntu-latest`. Around 2026-10-23 16:15 UTC GitHub moved that label to
  the Ubuntu 26.04 image (`ubuntu-26.04`, version 20261012.163.1 in the job logs). Its glibc libm returns
  different last digits for some transcendental functions. Since then every PR has gone red on code it does
  not touch. The eval gate reports 17 regressions in probes/eng-impower-angle-cut and eng-impower-past-cut
  (IMPOWER's angle and power), and the fitlab test `test_norm_s_dist_scores` counts 1087 glibc `erfc` hits
  instead of 1103 (PRs #1630-#1633). One of them is a re-run of a T-2291 head that was green on 24.04 at
  15:31. The engine's results, and the fitted Excel kernels it relies on, depend on the host libm. The
  baselines and the fitlab scores were recorded on Ubuntu 24.04 (glibc 2.39), which is also what the agent
  sandboxes run.
- **Decision:** Every job in `.github/workflows/` runs on `ubuntu-24.04`, not `ubuntu-latest`. Moving to a
  newer image is its own task. That task runs `make check` on the new image, then either explains each
  changed cell (and re-records the baseline through `make eval-baseline` under a decision record) or makes the
  affected kernels independent of the host libm.
- **Consequences:** CI goes back to judging PRs on the environment the baselines were recorded on. A later
  image move no longer breaks every open PR silently. The libm dependence itself stays: an
  engine built on another glibc may differ in the last digit for IMPOWER, NORM.S.DIST and others. Making
  those kernels self-contained (no host `pow`/`atan2`/`erfc` in Excel-fitted paths) is follow-up work, not
  part of this pin.
