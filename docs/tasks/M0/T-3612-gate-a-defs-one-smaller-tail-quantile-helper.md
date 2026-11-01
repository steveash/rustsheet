---
id: T-3612
title: "Gate A defs: one smaller-tail quantile helper and a shared DF_CUT"
milestone: M0
lane: quality
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The Gate A definitions (`tools/src/rustsheet_tools/accuracy/defs/`) solve a quantile from its smaller tail in one
shared helper, take the CHISQ.INV df limit from one shared place, and compute the series tolerance once.

## Context
- Review minors on T-3529 (PR #2668), deferred so the reviewed head did not move:
  - `fisher.quantile` (fisher.py), `gamma.quantile` (gamma.py) and `beta.quantile` (beta.py) repeat the same
    skeleton: pick the smaller tail, then solve ln tail(e^s) = ln q with `solve_log_bracket`. Only the tail functions
    and the bracket differ.
  - gamma.py imports `DF_CUT` from student_t.py, so the chi-squared family depends on the Student-t family for a
    statistics.md constant.
  - gamma.py's `_series` and `_continued_fraction` each compute `eps = 2 ** (-prec + 4)` with the same comment.
- Pure refactor: every generated table and sidecar must stay byte-identical.

## Acceptance criteria
- [ ] One helper in `accuracy/model.py` (next to `solve_log_bracket`) takes the lower and upper tail functions, p and
      the bracket; fisher, gamma and beta call it.
- [ ] `DF_CUT` lives in a shared module, and student_t.py and gamma.py import it from there.
- [ ] The series tolerance is computed by one helper in gamma.py.
- [ ] `rustsheet-accuracy generate` for an existing table reproduces the committed `s1.math.json` byte for byte (for
      F.INV, GAMMA.INV and BETA.DIST at least), and `tools/tests` pass unchanged.
- [ ] `make check` green; reviewers approve.

## Out of scope
- New definitions or changing any value.
