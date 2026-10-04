# 0014 — Excel is always right; every quirk is recorded

- **Status:** accepted (2026-10-04, owner request)
- **Context:**
  - Parity with Microsoft 365 Excel is the project's bar, but the instructions did not say
    outright that behavior which looks like an Excel bug must be matched too, nor that every
    quirk found must be recorded. Quirks surfaced in passing (eval triage, other tasks) could be
    implemented without an entry, or "tidied up".
  - The owner wants a canonical list of the Excel quirks we know about and accept, and RustSheet
    to match Excel exactly even where Excel is wrong, arbitrary or inconsistent.
- **Decision:**
  - Excel is always right. RustSheet matches the observed behavior exactly; agents never
    "correct" Excel or implement a nicer variant.
  - `docs/excel-semantics/` is the canonical list of known, accepted quirks. Every quirk found is
    recorded there, with evidence, in the PR that found it, whether or not anything implements it yet.
  - The only allowed deviations are those CLAUDE.md directive 5 requires (resource, recursion
    and allocation limits, no I/O from formulas, no panics); directive 5 wins over directive 1.
    This record is the standing authority for all of them; each needs a `DEVIATION (decision
    0014)` entry in docs/excel-semantics/. Existing ones are being catalogued (T-0012); the
    BESSELJ/BESSELY order cap is the first.
  - Text: CLAUDE.md directives 1 and 6, `docs/excel-semantics/README.md`, the `excel-probe` and
    `work-loop` skills, the gap-triager agent, and check 6 of `reviewer-excel-fidelity`.
- **Consequences:**
  - The Excel-fidelity reviewer blocks changes that correct Excel or deviate for any reason other
    than directive 5; a directive-5 deviation without its entry is only MAJOR, so adding a safety
    limit is never penalized more than omitting one. Implemented or uncovered behavior with no
    entry is MAJOR.
  - The semantics docs grow with every quirk found, including ones no task implements yet.
