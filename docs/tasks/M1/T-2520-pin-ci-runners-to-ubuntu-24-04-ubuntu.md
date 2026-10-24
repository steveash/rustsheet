---
id: T-2520
title: "Pin CI runners to ubuntu-24.04 (ubuntu-latest moved to 26.04, libm changed)"
milestone: M1
lane: harness
status: done
priority: P0
size: S
tier: opus
depends_on: []
---

## Goal
CI judges PRs on the Ubuntu 24.04 image the baselines were recorded on, so a runner-image change no longer turns every PR red.

## Context
`ubuntu-latest` moved to Ubuntu 26.04 on 2026-10-23. Its libm changed the last digit of 17 IMPOWER probe cells (probes/eng-impower-angle-cut, eng-impower-past-cut) and the fitlab glibc-erfc NORM.S.DIST score (1087 vs 1103), and every open PR went red (#1630-#1633). Decision 0102.

## Acceptance criteria
- [x] Every job in .github/workflows/ runs on `ubuntu-24.04`.
- [x] Decision 0102 records why, and how a later image move must be done.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Making IMPOWER, NORM.S.DIST and other kernels independent of the host libm.

## Outcome
Pinned every workflow job to `ubuntu-24.04` (decision 0102). No engine or eval change, so the eval delta is 0.
