# 0052 — Deviations from Excel only for directive 5; decisions 0033 and 0038 reopened

- **Status:** accepted (2026-10-13, T-1030). Supersedes decision 0033 and decision 0038 as
  deviations, and states decision 0036's scope.
- **Context:**
  - The governing documents disagreed. CLAUDE.md directive 1 says the only exception to "Excel is
    always right" is what directive 5 requires (resource, recursion and allocation limits, no I/O,
    no panics). Decision 0014, made at the owner's request, says the same, and so do check 6 of
    `reviewer-excel-fidelity` and docs/process/quality-gates.md. But docs/excel-semantics/README.md
    ("Deliberate deviations") added: "Any other kind of deviation needs its own decision record".
    Decisions 0033 and 0038 took that route.
  - Decision 0033 (lone surrogates) deviates because engine text is a Rust `String`, and moving
    to UTF-16 or WTF-8 text is a large change. That is a cost argument. Decision 0014 and the
    fidelity reviewer reject cost as a reason ("never a convenience").
  - Decision 0038 (WEEKNUM stale return_type) deviates because Excel's value depends on its calc
    order. But that order is not arbitrary. On a fresh load the probes fit "bottom to top within a
    sheet", and after edits it follows `calcChain.xml`, which is part of the file. Matching it is
    hard and needs work on calc order, but it is not proven impossible.
  - T-0287 (dropped, 2026-10-08) already refused to list the 0038 cells in
    eval/suites/probes/deviations: decision 0036 lets that file cite only directive-5 limits.
  - The cost: decision 0051 assigns probes/text-surrogate-positions (A6, A11, A12, A29, A38, B10)
    and probes/concatenate-limit-spill (E8–E11) to M2. These 10 cells are known failures, so M2
    cannot exit until the engine keeps lone surrogates. The 0038 cells are M6 scope.
- **Options considered:**
  - *(a) Allow deviations by decision record, with criteria* (for example "Excel's value is not a
    function of the workbook", or "the fix is an architecture change"). It would unblock M2 at
    once. But every hard quirk would become a candidate, and agents with no human review would
    judge their own exceptions. The first criterion does not even cover 0038 cleanly (the calc
    chain is in the file), and the second is the cost argument directive 1 forbids. Rejected.
  - *(b) Directive 5 is the only exception, everywhere.* CLAUDE.md, decision 0014 and the
    fidelity reviewer already say this, so only the semantics README and the two records change.
    Chosen.
- **Decision:**
  1. RustSheet deviates from Excel only where CLAUDE.md directive 5 requires it. Decision 0014 is
     the standing authority for those deviations, and their semantics entries read
     `Status: DEVIATION (decision 0014)`. No other decision record can authorize a deviation. A
     behavior RustSheet does not match for any other reason is a known gap: its entry is
     `VERIFIED` (Excel's behavior is known) and names the task that will close it.
  2. Decisions 0033 and 0038 stop being deviations. Their entries in docs/excel-semantics
     (text.md, coercion.md, dates.md) become `VERIFIED` with an "Ours" note and a task. The
     current behavior stays until those tasks land: dropping a straddling surrogate pair whole,
     and `#NUM!` for a WEEKNUM return_type of 22 or more or negative. It is a known gap, not a
     choice.
  3. Decision 0036's `deviations` files list only cells that differ because of a directive-5
     limit and cite decision 0014. A cell RustSheet gets wrong for any other reason stays an F
     line in eval/baseline and counts against its milestone's exit.
  4. Follow-ups:
     - T-1483 (engine, M2, P1): a spec for engine text that can hold lone surrogates. It covers
       the 10 M2 cells.
     - T-1484 (engine, M6, P3): a spec for reproducing WEEKNUM's stale return_type state in
       Excel's calc order.
- **Consequences:**
  - docs/excel-semantics/README.md "Deliberate deviations" now matches CLAUDE.md directive 1 and
    decision 0014. CLAUDE.md and decision 0014 already said this and do not change.
  - The 10 lone-surrogate cells stay F lines in eval/baseline/probes.txt. M2 exit waits for
    T-1483 and the implementation tasks it files. The five WEEKNUM cells stay F lines for M6.
  - Code comments that cited 0033 or 0038 now cite the follow-up tasks
    (`TODO(T-1483)`, `TODO(T-1484)`).
  - If an Excel behavior ever turns out to be truly impossible to reproduce from the file, the
    route is the owner, who changes CLAUDE.md directive 1. An agent decision record is not
    enough.
