---
id: T-2631
title: "Probe the IM* log-space #NUM! edge: IMSIN/IMCOS 709<|y|<709.78 with sin/cos x<0, IMSECH/IMCSCH past |x|=709, IMCOSH past 709 with a large y"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
excel-graph evidence on whether Excel's log-space `#NUM!` (IMCOSH/IMSINH past |x| = 709 are `#NUM!` when cos y or
sin y is negative, T-2231) reaches the neighbouring kernels, recorded in docs/excel-semantics/engineering.md.

## Context
T-2231 (docs/excel-semantics/engineering.md "IMCOSH and IMSINH past |x| = 709 work in log space") left these unprobed:
- IMSIN/IMCOS with 709 < |y| < 709.78 and sin x or cos x negative: `IMSIN("2+709.5i")`, `IMCOS("2+709.5i")`,
  `IMSIN("-1+709.5i")`. The existing probes only use x = 1.5, where both are positive.
- IMSECH/IMCSCH past |x| = 709 with cos y or sin y negative: `IMSECH("709.5+2i")`, `IMCSCH("709.5-0.5i")`.
- IMCOSH/IMSINH past |x| = 709 with a large y, where SIN's x87 reduction could change digits: `IMCOSH("709.5+1000.5i")`,
  `IMSINH("709.5+100000.3i")`.
Write one probe case (probes/eng-im-log-space-edge, tag M9), fill it with `rustsheet-oracle fill -b excel-graph`,
and record the results. Any mismatch with ours becomes an implementation task.

## Acceptance criteria
- [ ] The probe case and its oracle sidecar are committed, and each question has a VERIFIED bullet in engineering.md.
- [ ] Mismatches with ours are baselined (`make eval-baseline`) and filed as tasks.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
