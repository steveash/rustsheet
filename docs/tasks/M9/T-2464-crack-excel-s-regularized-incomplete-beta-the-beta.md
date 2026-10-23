---
id: T-2464
title: "Crack Excel's regularized incomplete beta (the BETA.DIST family's kernel) from the gen-m9 sweeps, with Excel's own EXP, LN and GAMMALN"
milestone: M9
lane: research
status: todo
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
Identify Excel's regularized incomplete beta I_x(a, b), the kernel under BETA.DIST, BETADIST, F.DIST, F.DIST.RT,
F.TEST, T.DIST.RT, T.DIST.2T, T.TEST, BETA.INV and the F/T inverses (about 5,600 visibly-off cells, 31% of all on
2026-10-23), well enough that a model is bit-exact or within 15 digits on the sweeps.

## Context
- Prior work: statistics.md "Incomplete beta: Excel's closed forms and the best candidate (T-1744)" (direct branch
  unexplained, best 15 of 107 points), the T-0969 entries (Student t is this kernel; TOMS 708 betaln structure for
  B(a,b)), T-1745 (BETA.INV inverts Excel's own CDF), T-1759 (LibreOffice and published algorithms rejected).
- New since then: Excel's EXP (model V6), LN (FLDLN2/FYL2X) and GAMMALN/GAMMA (T-1941, T-2128, T-2129) are
  modelled to the bit, and the gen-m9 sweeps (T-2255) hold thousands of oracle-filled BETA.DIST, F.DIST and
  BINOM.DIST cells. The earlier candidates were scored with libm elementary functions.
- Method: docs/research/excel-kernel-playbook.md. Work and data live in docs/research/t2464/.

## Acceptance criteria
- [ ] A consolidated dataset of every oracle-filled incomplete-beta cell (BETA.DIST cdf and pdf, F.DIST family,
      T.DIST.RT) with its numeric arguments, in docs/research/t2464/.
- [ ] Every hypothesis tried is recorded with its bit-exact, within-4-ulp and within-15-digit counts, rejected ones
      included (decision 0056), and the findings go into statistics.md.
- [ ] If a model reaches the visible-parity stage on the sweeps, file the engine task (or land it here if small).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
