---
id: T-0064
title: "File the fifteen-digit parity checkpoint, its coverage sweeps and the remaining gap tasks (decision 0094)"
milestone: M0
lane: harness
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
The owner can see what is left before RustSheet has fifteen-digit parity, and the work to get
there is filed and ordered.

## Context
On the eval of 2026-10-21 (`main` 7dab5361):
- the decision 0088 lock (1,825 cells) counts the gated suites only; all vendored suites together
  have 5,804 visibly-off cells in 231 functions;
- 358 of 506 in-scope functions have fewer than 400 Excel cells, and the M7 and M9 params have no
  generator edge pools;
- 89 visibly-off functions had no open campaign task;
- 12 open probes and sweeps blocked campaign fixes but were ordered by milestone.

## Acceptance criteria
- [x] Decision 0094 defines the checkpoint (coverage, zero visibly-off cells in every suite, two
      steward runs), its exemptions and its measurement.
- [x] Filed, all `campaign: visible`: T-0062 (report section), T-0063 (generator `--functions`
      and `--category`), T-2247 to T-2250 (edge pools), T-2251 (INFO and SHEETS catalog) and
      T-2252 to T-2258 (oracle sweeps, which depend on the generator and pool tasks).
- [x] The gap-triager traced the 89 untasked visibly-off functions. Every one now has a campaign
      task (2 new: T-2259, T-2260 a probe without the key; 36 extended) or a stated reason not
      to have one (Outcome).
- [x] `scripts/tasks.py next` promotes the open dependencies of campaign tasks (`in_campaign`),
      with a test.
- [x] `make check` green; reviewers approve.

## Out of scope
- Implementing the report section, the generator options, the pools or the sweeps.

## Outcome
Docs, task files, and the `scripts/tasks.py` ordering and its test; no eval delta.

Visibly-off functions left without a campaign task, with the reason:
- **FORECAST.ETS family:** desktop-only evidence; T-1553 is blocked in M14. Only T-2028's 7 lift
  cells are tagged.
- **WEEKNUM:** spec task T-1484.
- **COUPNUM:** the 1904 date system, an exotic calendar under decision 0083 (T-2125).
- **BESSELJ:** the order-cap DEVIATION.
- **IMCOS:** desktop only (T-2043).
- **ISNUMBER, COUNT and ISFORMULA:** circular references and data tables, both Phase 2 (T-0401,
  T-0441).
- **CORREL, MONTH, QUARTILE and MINVERSE:** owned by probe tasks (T-2051, T-2066, T-2067).
- **HSTACK:** a directive-5 bound.

Exemptions from the 400-cell bar (decision 0094): TRUE, FALSE, RAND, RANDBETWEEN, INFO, NOW,
TODAY and RANDARRAY. PI and NA already have more than 400 cells.
